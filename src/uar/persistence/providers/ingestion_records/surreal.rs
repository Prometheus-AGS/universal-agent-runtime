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
        if message.contains("uar_ingestion_regeneration_conflict") { return Err(crate::uar::rag::regeneration::RegenerationError::Conflict.into()); }
        if message.contains("uar_ingestion_source_missing") { return Err(crate::uar::rag::regeneration::RegenerationError::SourceMissing.into()); }
        if message.contains("uar_ingestion_revoked") { return Err(IngestionStoreError::Revoked.into()); }
        if message.contains("uar_ingestion_kb_missing") { return Err(IngestionStoreError::MissingKnowledgeBase.into()); }
    }
    // A failed COMMIT marks earlier statement slots NotExecuted. Preserve the
    // actual cause rather than returning one of those cancellation wrappers.
    if let Some((_, error)) = errors.into_iter().min_by_key(|(index, error)| (
        matches!(error.query_details(), Some(surrealdb::types::QueryError::NotExecuted | surrealdb::types::QueryError::Cancelled)),
        *index,
    )) { return Err(error.into()); }
    Ok(())
}

pub(super) async fn accept(store: &SurrealDbProvider, document: &KnowledgeDocument, source: &[u8]) -> Result<IngestionJob> {
    require_durable(store)?;
    anyhow::ensure!(document.status == DocumentStatus::Pending && document.chunk_count == 0, "new upload must be pending");
    let job = IngestionJob::new(document)?;
    let namespace = job.namespace;
    let mut attempt = 0;
    loop {
    attempt += 1;
    let result = check(store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $kb = (SELECT * FROM knowledge_bases WHERE owner_id=$owner AND (logical_id=$kb_id OR id=type::record('knowledge_bases',$kb_id)))[0];
        IF $kb = NONE { THROW 'uar_ingestion_kb_missing'; };
        IF $kb.config.regeneration.state IN ['running','failed'] { THROW 'uar_ingestion_regeneration_conflict'; };
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
        .await?);
    match result {
        Ok(()) => return Ok(job),
        Err(error) if attempt < 3 && error.chain().any(|cause| {
            cause.downcast_ref::<surrealdb::Error>().is_some_and(|error|
                matches!(error.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
        }) => {
            // Concurrent uploads touch the same KB fence. Retry only a typed
            // rolled-back conflict, reusing this job/document and source bytes.
            // Unknown commits, transport failures and other errors are not retried.
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        Err(error) => return Err(error),
    }
    }
}

pub(super) async fn discover(store: &SurrealDbProvider, limit: usize) -> Result<Vec<IngestionJob>> {
    require_durable(store)?;
    let mut jobs = Vec::new();
    for namespace in [KnowledgeNamespace::Legacy, KnowledgeNamespace::PrincipalV1] {
        let mut response = store.db.query(namespace.query("SELECT data,next_attempt_at,owner_id,document_id FROM knowledge_ingestion WHERE ($principal OR owner_id='anonymous') AND [owner_id,kb_id] IN (SELECT VALUE [owner_id,logical_id] FROM knowledge_bases WHERE config.regeneration.state=NONE OR config.regeneration.state=NULL OR config.regeneration.state IN ['running','complete']) AND state IN ['pending','processing','retryable'] AND next_attempt_at <= time::unix() ORDER BY next_attempt_at,owner_id,document_id LIMIT $limit"))
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

pub(super) async fn next_due_at(store: &SurrealDbProvider) -> Result<Option<i64>> {
    require_durable(store)?;
    let mut due_at = None;
    for namespace in [KnowledgeNamespace::Legacy, KnowledgeNamespace::PrincipalV1] {
        let mut response = store.db.query(namespace.query("SELECT VALUE next_attempt_at FROM knowledge_ingestion WHERE ($principal OR owner_id='anonymous') AND [owner_id,kb_id] IN (SELECT VALUE [owner_id,logical_id] FROM knowledge_bases WHERE config.regeneration.state=NONE OR config.regeneration.state=NULL OR config.regeneration.state IN ['running','complete']) AND state IN ['pending','processing','retryable'] AND next_attempt_at > time::unix() ORDER BY next_attempt_at,owner_id,document_id LIMIT 1"))
            .bind(("principal", namespace == KnowledgeNamespace::PrincipalV1))
            .await?
            .check()?;
        let rows: Vec<i64> = response.take(0)?;
        if let Some(next) = rows.into_iter().next() {
            due_at = Some(due_at.map_or(next, |current: i64| current.min(next)));
        }
    }
    Ok(due_at)
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
        LET $kb=(SELECT * FROM knowledge_bases WHERE owner_id=$owner AND logical_id=$kb_id)[0];
        IF $kb=NONE { THROW 'uar_ingestion_revoked'; };
        IF $regen != NONE AND $regen != NULL {
            IF $kb.config.regeneration.generation != $regen OR $kb.config.regeneration.state != 'running' { THROW 'uar_ingestion_revoked'; };
        } ELSE {
            IF $kb.config.regeneration.state IN ['running','failed'] { THROW 'uar_ingestion_revoked'; };
        };
        UPDATE $kb.id SET updated_at=$now;
        LET $updated = UPDATE type::record('knowledge_ingestion',$key) SET state=$state,next_attempt_at=$next,data=$after WHERE data=$before RETURN AFTER;
        IF array::len($updated) != 1 { THROW 'uar_ingestion_revoked'; };
        LET $document = UPDATE type::record('knowledge_documents',$key) SET status=$status,updated_at=$now WHERE owner_id=$owner RETURN AFTER;
        IF array::len($document) != 1 { THROW 'uar_ingestion_revoked'; };
        IF $regen != NONE AND $regen != NULL {
            LET $done=array::len((SELECT id FROM knowledge_ingestion WHERE owner_id=$owner AND kb_id=$kb_id AND regeneration=$regen AND state='indexed' AND document_id IN $kb.config.regeneration.document_ids));
            UPDATE $kb.id SET config.regeneration.completed_documents=$done,config.regeneration.last_error=$error,config.regeneration.updated_at=$now;
            IF $state='failed' { UPDATE $kb.id SET config.regeneration.state='failed'; };
        };
        COMMIT TRANSACTION;"))
        .bind(("key", before.storage_key())).bind(("owner", before.owner_id.clone()))
        .bind(("kb_id",before.kb_id.clone())).bind(("regen",before.regeneration.clone())).bind(("error",after.last_error.clone()))
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
        IF $regen != NONE AND $regen != NULL {
            IF $kb.config.regeneration.generation != $regen OR $kb.config.regeneration.state != 'running' OR $kb.config.regeneration.target != $embedding OR $document_id NOT IN $kb.config.regeneration.document_ids { THROW 'uar_ingestion_revoked'; };
        } ELSE {
            IF $kb.config.regeneration.state IN ['running','failed'] { THROW 'uar_ingestion_revoked'; };
            IF $kb.config.indexed_embedding != NONE AND $kb.config.indexed_embedding != NULL AND $kb.config.indexed_embedding != $embedding { THROW 'ingestion embedding space changed'; };
        };
        LET $updated = UPDATE type::record('knowledge_ingestion',$key) SET state='indexed',data=$after WHERE data=$before RETURN AFTER;
        IF array::len($updated) != 1 { THROW 'uar_ingestion_revoked'; };
        LET $document = UPDATE type::record('knowledge_documents',$key) SET status=$status,chunk_count=$count,updated_at=$now WHERE owner_id=$owner AND kb_id=$kb_id RETURN AFTER;
        IF array::len($document) != 1 { THROW 'uar_ingestion_revoked'; };
        DELETE knowledge_chunks WHERE owner_id=$owner AND document_id=$document_id;
        FOR $chunk IN $chunks { CREATE type::record('knowledge_chunks',$chunk.key) CONTENT $chunk.payload; };
        UPDATE $kb.id SET updated_at=$now;
        IF $regen != NONE AND $regen != NULL {
            LET $done=array::len((SELECT id FROM knowledge_ingestion WHERE owner_id=$owner AND kb_id=$kb_id AND regeneration=$regen AND state='indexed' AND document_id IN $kb.config.regeneration.document_ids));
            UPDATE $kb.id SET config.regeneration.completed_documents=$done,config.regeneration.updated_at=$now;
            IF $done=array::len($kb.config.regeneration.document_ids) {
                UPDATE $kb.id SET config.indexed_embedding=$embedding,config.embedding_provider=$embedding.backend,config.embedding_model=$embedding.model,config.vector_dimensions=$embedding.dimension,config.regeneration.state='complete',config.regeneration.last_error=NULL;
            };
        } ELSE { UPDATE $kb.id SET config.indexed_embedding=$embedding; };
        COMMIT TRANSACTION;"))
        .bind(("key", job.storage_key())).bind(("owner", job.owner_id.clone())).bind(("kb_id", job.kb_id.clone()))
        .bind(("document_id", job.document_id.clone())).bind(("before", serde_json::to_string(job)?))
        .bind(("regen",job.regeneration.clone()))
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
         LET $doc=(SELECT * FROM type::record('knowledge_documents',$key))[0];
         LET $kb=(SELECT * FROM knowledge_bases WHERE owner_id=$owner AND logical_id=$doc.kb_id)[0];
         IF $kb.config.regeneration.state IN ['running','failed'] { THROW 'uar_ingestion_regeneration_conflict'; };
         IF $kb != NONE { UPDATE $kb.id SET updated_at=<string>time::now(); };
         DELETE type::record('knowledge_ingestion',$key);
         DELETE knowledge_chunks WHERE owner_id=$owner AND document_id=$id;
         DELETE knowledge_documents WHERE owner_id=$owner AND (logical_id=$id OR id=type::record('knowledge_documents',$id));
         COMMIT TRANSACTION;"
    };
    check(store.db.query(namespace.query(sql)).bind(("owner", owner.to_owned())).bind(("id", id.to_owned()))
        .bind(("key", crate::uar::persistence::tenant_storage_key(owner, id))).await?)
}

pub(super) async fn begin_regeneration(store: &SurrealDbProvider, owner: &str, kb_id: &str,
    target: &crate::uar::domain::knowledge::EmbeddingFingerprint)
    -> Result<crate::uar::domain::knowledge::KnowledgeRegeneration> {
    use crate::uar::persistence::PersistenceLayer;
    use crate::uar::domain::knowledge::{KnowledgeRegeneration, KnowledgeRegenerationState};
    use crate::uar::rag::regeneration::RegenerationError;
    require_durable(store)?;
    let namespace = KnowledgeNamespace::for_owner(owner)?;
    let kb = store.get_knowledge_base(owner,kb_id).await?.ok_or(IngestionStoreError::MissingKnowledgeBase)?;
    if kb.regeneration_unavailable() && kb.config.regeneration.as_ref().is_some_and(|job| job.state == KnowledgeRegenerationState::Running) {
        return Err(RegenerationError::Conflict.into());
    }
    let mut documents = store.list_documents(owner,kb_id).await?;
    documents.sort_by(|a,b| a.id.cmp(&b.id));
    let cohort = KnowledgeRegeneration::new(kb.config.indexed_embedding.clone(),target.clone(),documents.iter().map(|doc|doc.id.clone()).collect());
    let mut updates = Vec::new();
    for doc in documents {
        let before = get(store,owner,&doc.id).await?.ok_or(RegenerationError::SourceMissing)?;
        let mut after = before.retry();
        after.regeneration=Some(cohort.generation.clone());
        updates.push(serde_json::json!({"key":before.storage_key(),"document_id":doc.id,
            "before":serde_json::to_string(&before)?,"after":serde_json::to_string(&after)?,"next":after.next_attempt_at}));
    }
    let mut config = kb.config.clone();
    config.regeneration=Some(cohort.clone());
    if cohort.state == KnowledgeRegenerationState::Complete {
        config.indexed_embedding=Some(target.clone());
        config.embedding_provider.clone_from(&target.backend);
        config.embedding_model.clone_from(&target.model);
        config.vector_dimensions=Some(target.dimension);
    }
    check(store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $kb=(SELECT * FROM knowledge_bases WHERE owner_id=$owner AND logical_id=$kb_id)[0];
        IF $kb=NONE { THROW 'uar_ingestion_kb_missing'; };
        IF $kb.config != $before_config { THROW 'uar_ingestion_regeneration_conflict'; };
        LET $documents=(SELECT id FROM knowledge_documents WHERE owner_id=$owner AND kb_id=$kb_id);
        IF array::len($documents) != array::len($updates) { THROW 'uar_ingestion_regeneration_conflict'; };
        LET $orphan=(SELECT id FROM knowledge_chunks WHERE owner_id=$owner AND kb_id=$kb_id AND (document_id=NONE OR document_id=NULL OR document_id NOT IN $config.regeneration.document_ids));
        IF array::len($orphan)>0 { THROW 'uar_ingestion_source_missing'; };
        UPDATE $kb.id SET config=$config,updated_at=$now;
        FOR $item IN $updates {
            LET $source=(SELECT * FROM type::record('knowledge_ingestion',$item.key))[0];
            IF $source=NONE OR $source.source=NONE OR $source.source=NULL { THROW 'uar_ingestion_source_missing'; };
            LET $updated=UPDATE type::record('knowledge_ingestion',$item.key) SET data=$item.after,state='pending',regeneration=$generation,next_attempt_at=$item.next WHERE data=$item.before AND owner_id=$owner AND kb_id=$kb_id RETURN AFTER;
            IF array::len($updated)!=1 { THROW 'uar_ingestion_regeneration_conflict'; };
            LET $doc=UPDATE type::record('knowledge_documents',$item.key) SET status={status:'pending'},updated_at=$now WHERE owner_id=$owner AND kb_id=$kb_id RETURN AFTER;
            IF array::len($doc)!=1 { THROW 'uar_ingestion_regeneration_conflict'; };
        };
        COMMIT TRANSACTION;"))
        .bind(("owner",owner.to_owned())).bind(("kb_id",kb_id.to_owned())).bind(("before_config",serde_json::to_value(kb.config)?))
        .bind(("config",serde_json::to_value(config)?)).bind(("generation",cohort.generation.clone()))
        .bind(("updates",updates)).bind(("now",chrono::Utc::now().to_rfc3339())).await?)?;
    Ok(cohort)
}

pub(super) async fn save_kb(store: &SurrealDbProvider, kb: &crate::uar::domain::knowledge::KnowledgeBase) -> Result<()> {
    let namespace=KnowledgeNamespace::for_owner(&kb.owner_id)?;
    check(store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $existing=(SELECT * FROM type::record('knowledge_bases',$key))[0];
        IF $existing != NONE {
            IF $existing.config.regeneration.state IN ['running','failed'] OR $existing.config.regeneration != $payload.config.regeneration { THROW 'uar_ingestion_regeneration_conflict'; };
        };
        UPSERT type::record('knowledge_bases',$key) CONTENT $payload;
        IF $existing != NONE AND $existing.config.indexed_embedding != NONE AND $existing.config.indexed_embedding != NULL {
            UPDATE type::record('knowledge_bases',$key) SET config.indexed_embedding=$existing.config.indexed_embedding;
        };
        COMMIT TRANSACTION;"))
        .bind(("key",crate::uar::persistence::tenant_storage_key(&kb.owner_id,&kb.id)))
        .bind(("payload",tenant_record_payload(kb,&kb.id)?)).await?)
}

pub(super) async fn save_member<T: serde::Serialize>(store: &SurrealDbProvider, table: &'static str,
    owner: &str, kb: &str, id: &str, value: &T) -> Result<()> {
    let namespace=KnowledgeNamespace::for_owner(owner)?;
    check(store.db.query(namespace.query("BEGIN TRANSACTION;
        LET $kb=(SELECT * FROM knowledge_bases WHERE owner_id=$owner AND logical_id=$kb_id)[0];
        IF $kb=NONE { THROW 'uar_ingestion_kb_missing'; };
        IF $kb.config.regeneration.state IN ['running','failed'] { THROW 'uar_ingestion_regeneration_conflict'; };
        UPDATE $kb.id SET updated_at=$now;
        IF $chunk AND $kb.config.indexed_embedding != NONE AND $kb.config.indexed_embedding != NULL {
            IF array::len($payload.embedding) != $kb.config.indexed_embedding.dimension { THROW 'chunk embedding dimension differs from indexed space'; };
            IF $payload.metadata.embedding_space != NONE AND $payload.metadata.embedding_space != NULL AND $payload.metadata.embedding_space != $kb.config.indexed_embedding { THROW 'chunk embedding space was superseded'; };
        };
        UPSERT type::record($table,$key) CONTENT $payload;
        LET $legacy=(SELECT * FROM type::record($table,$logical))[0];
        IF $legacy != NONE AND $legacy.owner_id=$owner { DELETE $legacy.id; };
        COMMIT TRANSACTION;"))
        .bind(("table",namespace.table(table).to_owned())).bind(("key",crate::uar::persistence::tenant_storage_key(owner,id)))
        .bind(("chunk",table == "knowledge_chunks"))
        .bind(("owner",owner.to_owned())).bind(("kb_id",kb.to_owned())).bind(("logical",id.to_owned()))
        .bind(("payload",tenant_record_payload(value,id)?)).bind(("now",chrono::Utc::now().to_rfc3339())).await?)
}

pub(super) async fn search_in_space(store: &SurrealDbProvider, owner: &str, ids: Option<&[&str]>, query: &[f32],
    limit: usize, min_score: f32, space: Option<&crate::uar::domain::knowledge::EmbeddingFingerprint>)
    -> Result<Vec<crate::uar::domain::knowledge::KnowledgeMatch>> {
    use crate::uar::domain::knowledge::{KnowledgeChunk,KnowledgeMatch};
    let namespace=KnowledgeNamespace::for_owner(owner)?;
    let ids: Option<Vec<String>>=ids.map(|ids|ids.iter().map(|id|(*id).to_owned()).collect());
    let mut response=store.db.query(namespace.query("SELECT * FROM knowledge_chunks WHERE owner_id=$owner
        AND kb_id IN (SELECT VALUE logical_id FROM knowledge_bases WHERE owner_id=$owner
            AND ($ids=NONE OR $ids=NULL OR logical_id IN $ids)
            AND (config.regeneration=NONE OR config.regeneration=NULL OR config.regeneration.state='complete')
            AND ($space=NONE OR $space=NULL OR config.indexed_embedding=$space));"))
        .bind(("owner",owner.to_owned())).bind(("ids",ids)).bind(("space",space.map(serde_json::to_value).transpose()?)).await?.check()?;
    let rows:Vec<surrealdb::types::Value>=response.take(0)?;
    let chunks=rows.into_iter().map(|row|super::from_db_value::<KnowledgeChunk>(
        super::restore_tenant_record_id(super::surreal_to_json(row)?))).collect::<Result<Vec<_>>>()?;
    super::warn_zero_norm_chunks(&chunks);
    let mut result=Vec::new();
    for chunk in chunks {
        if chunk.embedding.len()!=query.len() { continue; }
        let score=super::cosine_similarity(&chunk.embedding,query);
        if score>=min_score { result.push(KnowledgeMatch {chunk,score}); }
    }
    result.sort_by(|a,b|b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    result.truncate(limit);
    Ok(result)
}
