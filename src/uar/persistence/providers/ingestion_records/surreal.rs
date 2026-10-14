use crate::uar::persistence::knowledge_namespace::KnowledgeNamespace;
use super::{SurrealDbProvider, tenant_record_payload};
use crate::uar::domain::knowledge::{DocumentStatus, KnowledgeDocument};
use crate::uar::persistence::ingestion_records::{IngestionJob, IngestionPublication, IngestionState, IngestionStoreError};
use anyhow::Result;

pub(super) async fn initialize(db: &surrealdb::Surreal<surrealdb::engine::any::Any>) -> Result<()> {
    db.query(include_str!("../../../../../migrations/surrealdb/durable_ingestion.surql")).await?.check()?;
    db.query(include_str!("../../../../../migrations/surrealdb/principal_knowledge.surql")).await?.check()?;
    Ok(())
}

fn require_durable(store: &SurrealDbProvider) -> Result<()> {
    if !store.durable_instances { return Err(IngestionStoreError::Unsupported.into()); }
    Ok(())
}

fn state(job: &IngestionJob) -> Result<String> {
    Ok(serde_json::to_value(job.state)?.as_str().unwrap_or_default().to_owned())
}

fn check(mut response: surrealdb::IndexedResults) -> Result<()> {
    let errors = response.take_errors();
    // Match the existing provider's transaction handling: cancellation wrappers
    // may precede the deliberate domain refusal in the statement results.
    for error in errors.values() {
        let message = error.to_string();
        if message.contains("uar_ingestion_revoked") { return Err(IngestionStoreError::Revoked.into()); }
        if message.contains("uar_ingestion_kb_missing") { return Err(IngestionStoreError::MissingKnowledgeBase.into()); }
    }
    if let Some((_, error)) = errors.into_iter().min_by_key(|(index, _)| *index) { return Err(error.into()); }
    Ok(())
}

pub(super) async fn accept(store: &SurrealDbProvider, document: &KnowledgeDocument, source: &[u8]) -> Result<IngestionJob> {
    require_durable(store)?;
    anyhow::ensure!(document.status == DocumentStatus::Pending && document.chunk_count == 0, "new upload must be pending");
    let job = IngestionJob::new(document)?;
    let namespace = job.namespace;
    check(store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $kb = (SELECT * FROM knowledge_bases WHERE owner_id=$owner AND (logical_id=$kb_id OR id=type::record('knowledge_bases',$kb_id)))[0];
        IF $kb = NONE { THROW 'uar_ingestion_kb_missing'; };
        UPDATE $kb.id SET updated_at=$now;
        CREATE type::record('knowledge_documents',$key) CONTENT $document;
        CREATE type::record('knowledge_ingestion',$key) CONTENT $payload;
        COMMIT TRANSACTION;"))
        .bind(("owner", job.owner_id.clone())).bind(("kb_id", job.kb_id.clone()))
        .bind(("key", job.storage_key())).bind(("now", chrono::Utc::now().to_rfc3339()))
        .bind(("document", tenant_record_payload(document, &document.id)?))
        .bind(("payload", serde_json::json!({"owner_id": job.owner_id, "document_id": job.document_id,
            "kb_id": job.kb_id, "state": "pending", "next_attempt_at": job.next_attempt_at,
            "data": serde_json::to_string(&job)?, "source": source})))
        .await?)?;
    Ok(job)
}

pub(super) async fn discover(store: &SurrealDbProvider, limit: usize) -> Result<Vec<IngestionJob>> {
    require_durable(store)?;
    let mut jobs = Vec::new();
    for namespace in [KnowledgeNamespace::Legacy, KnowledgeNamespace::PrincipalV1] {
        let mut response = store.db.query(namespace.query("SELECT data,next_attempt_at,owner_id,document_id FROM knowledge_ingestion WHERE ($principal OR owner_id='anonymous') AND state IN ['pending','processing','retryable'] AND next_attempt_at <= time::unix() ORDER BY next_attempt_at,owner_id,document_id LIMIT $limit"))
            .bind(("principal", namespace == KnowledgeNamespace::PrincipalV1))
            .bind(("limit", limit as u64)).await?.check()?;
        let rows: Vec<surrealdb::types::Value> = response.take(0)?;
        for row in rows {
            let value = super::surreal_to_json(row)?;
            let job: IngestionJob = serde_json::from_str(value["data"].as_str().ok_or_else(|| anyhow::anyhow!("ingestion metadata missing"))?)?;
            job.validate_namespace()?;
            anyhow::ensure!(job.namespace == namespace, "ingestion record namespace differs from its table");
            jobs.push(job);
        }
    }
    jobs.sort_by(|a, b| (a.next_attempt_at, &a.owner_id, &a.document_id).cmp(&(b.next_attempt_at, &b.owner_id, &b.document_id)));
    jobs.truncate(limit);
    Ok(jobs)
}

pub(super) async fn get(store: &SurrealDbProvider, owner: &str, document: &str) -> Result<Option<IngestionJob>> {
    let namespace = KnowledgeNamespace::for_owner(owner)?;
    require_durable(store)?;
    let mut response = store.db.query(namespace.query("SELECT VALUE data FROM type::record('knowledge_ingestion',$key)"))
        .bind(("key", crate::uar::persistence::tenant_storage_key(owner, document))).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.into_iter().next().map(|row| {
        let job: IngestionJob = serde_json::from_str(&row)?;
        job.validate_namespace()?;
        anyhow::ensure!(job.namespace == namespace && job.owner_id == owner && job.document_id == document, "ingestion record identity differs from storage scope");
        Ok(job)
    }).transpose()
}

pub(super) async fn source(store: &SurrealDbProvider, job: &IngestionJob) -> Result<Option<Vec<u8>>> {
    let namespace = job.namespace;
    job.validate_namespace()?;
    require_durable(store)?;
    let mut response = store.db.query(namespace.query("SELECT VALUE source FROM type::record('knowledge_ingestion',$key) WHERE data=$before"))
        .bind(("key", job.storage_key())).bind(("before", serde_json::to_string(job)?)).await?.check()?;
    let rows: Vec<Vec<u8>> = response.take(0)?;
    Ok(rows.into_iter().next())
}

pub(super) async fn transition(store: &SurrealDbProvider, before: &IngestionJob, after: &IngestionJob) -> Result<bool> {
    let namespace = before.namespace;
    require_durable(store)?;
    before.validate_transition(after)?;
    let status = match after.state {
        IngestionState::Processing => DocumentStatus::Processing,
        IngestionState::Pending => DocumentStatus::Pending,
        IngestionState::Retryable | IngestionState::Failed => DocumentStatus::Failed { error: after.last_error.clone().unwrap_or_default() },
        IngestionState::Indexed => DocumentStatus::Indexed,
    };
    let response = store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $updated = UPDATE type::record('knowledge_ingestion',$key) SET state=$state,next_attempt_at=$next,data=$after WHERE data=$before RETURN AFTER;
        IF array::len($updated) != 1 { THROW 'uar_ingestion_revoked'; };
        LET $document = UPDATE type::record('knowledge_documents',$key) SET status=$status,updated_at=$now WHERE owner_id=$owner RETURN AFTER;
        IF array::len($document) != 1 { THROW 'uar_ingestion_revoked'; };
        COMMIT TRANSACTION;"))
        .bind(("key", before.storage_key())).bind(("owner", before.owner_id.clone()))
        .bind(("state", state(after)?)).bind(("next", after.next_attempt_at))
        .bind(("before", serde_json::to_string(before)?)).bind(("after", serde_json::to_string(after)?))
        .bind(("status", serde_json::to_value(status)?)).bind(("now", chrono::Utc::now().to_rfc3339())).await?;
    match check(response) {
        Ok(()) => Ok(true),
        Err(error) if error.downcast_ref::<IngestionStoreError>().is_some_and(|error| matches!(error, IngestionStoreError::Revoked)) => Ok(false),
        Err(error) => Err(error),
    }
}

pub(super) async fn publish(store: &SurrealDbProvider, job: &IngestionJob, publication: &IngestionPublication) -> Result<()> {
    let namespace = job.namespace;
    job.validate_namespace()?;
    require_durable(store)?;
    publication.validate(job)?;
    let chunks = publication.chunks.iter().map(|chunk| Ok(serde_json::json!({
        "key": crate::uar::persistence::tenant_storage_key(&job.owner_id, &chunk.id.to_string()),
        "payload": tenant_record_payload(chunk, &chunk.id.to_string())?
    }))).collect::<Result<Vec<_>>>()?;
    let mut indexed = job.clone();
    indexed.state = IngestionState::Indexed;
    indexed.last_error = None;
    check(store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $kb = (SELECT * FROM knowledge_bases WHERE owner_id=$owner AND (logical_id=$kb_id OR id=type::record('knowledge_bases',$kb_id)))[0];
        IF $kb = NONE { THROW 'uar_ingestion_revoked'; };
        IF $kb.config.indexed_embedding != NONE AND $kb.config.indexed_embedding != NULL AND $kb.config.indexed_embedding != $embedding { THROW 'ingestion embedding space changed'; };
        LET $updated = UPDATE type::record('knowledge_ingestion',$key) SET state='indexed',data=$after WHERE data=$before RETURN AFTER;
        IF array::len($updated) != 1 { THROW 'uar_ingestion_revoked'; };
        LET $document = UPDATE type::record('knowledge_documents',$key) SET status=$status,chunk_count=$count,updated_at=$now WHERE owner_id=$owner AND kb_id=$kb_id RETURN AFTER;
        IF array::len($document) != 1 { THROW 'uar_ingestion_revoked'; };
        DELETE knowledge_chunks WHERE owner_id=$owner AND document_id=$document_id;
        FOR $chunk IN $chunks { CREATE type::record('knowledge_chunks',$chunk.key) CONTENT $chunk.payload; };
        UPDATE $kb.id SET config.indexed_embedding=$embedding,updated_at=$now;
        COMMIT TRANSACTION;"))
        .bind(("key", job.storage_key())).bind(("owner", job.owner_id.clone())).bind(("kb_id", job.kb_id.clone()))
        .bind(("document_id", job.document_id.clone())).bind(("before", serde_json::to_string(job)?))
        .bind(("after", serde_json::to_string(&indexed)?)).bind(("status", serde_json::to_value(DocumentStatus::Indexed)?))
        .bind(("count", publication.chunks.len() as u64)).bind(("chunks", chunks))
        .bind(("embedding", serde_json::to_value(&publication.embedding)?)).bind(("now", chrono::Utc::now().to_rfc3339())).await?)
}

pub(super) async fn reconcile(store: &SurrealDbProvider) -> Result<()> {
    require_durable(store)?;
    for namespace in [KnowledgeNamespace::Legacy, KnowledgeNamespace::PrincipalV1] {
    store.db.query(namespace.query("BEGIN TRANSACTION;
        FOR $document IN (SELECT * FROM knowledge_documents WHERE ($principal OR owner_id='anonymous') AND (status.status IN ['pending','processing'] OR status IN ['pending','processing'])) {
            LET $job = (SELECT * FROM knowledge_ingestion WHERE owner_id=$document.owner_id AND document_id=$document.logical_id)[0];
            IF $job = NONE { UPDATE $document.id SET status={status:'failed',error:'source_bytes_missing: reupload required'},updated_at=$now; };
        };
        COMMIT TRANSACTION;"))
        .bind(("principal", namespace == KnowledgeNamespace::PrincipalV1))
        .bind(("now", chrono::Utc::now().to_rfc3339())).await?.check()?;
    }
    Ok(())
}

pub(super) async fn delete(store: &SurrealDbProvider, owner: &str, id: &str, entire_kb: bool) -> Result<()> {
    let namespace = KnowledgeNamespace::for_owner(owner)?;
    // Deletion participates in the same record-write conflicts as claim/publication.
    let sql = if entire_kb {
        "BEGIN TRANSACTION;
         DELETE knowledge_bases WHERE owner_id=$owner AND (logical_id=$id OR id=type::record('knowledge_bases',$id));
         DELETE knowledge_ingestion WHERE owner_id=$owner AND kb_id=$id;
         DELETE knowledge_chunks WHERE owner_id=$owner AND kb_id=$id;
         DELETE knowledge_documents WHERE owner_id=$owner AND kb_id=$id;
         COMMIT TRANSACTION;"
    } else {
        "BEGIN TRANSACTION;
         DELETE type::record('knowledge_ingestion',$key);
         DELETE knowledge_chunks WHERE owner_id=$owner AND document_id=$id;
         DELETE knowledge_documents WHERE owner_id=$owner AND (logical_id=$id OR id=type::record('knowledge_documents',$id));
         COMMIT TRANSACTION;"
    };
    store.db.query(namespace.query(sql)).bind(("owner", owner.to_owned())).bind(("id", id.to_owned()))
        .bind(("key", crate::uar::persistence::tenant_storage_key(owner, id))).await?.check()?;
    Ok(())
}
