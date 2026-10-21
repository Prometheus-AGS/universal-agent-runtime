use crate::uar::persistence::knowledge_namespace::KnowledgeNamespace;
use super::PostgresProvider;
use crate::uar::domain::knowledge::{DocumentStatus, KbConfig, KnowledgeDocument};
use crate::uar::persistence::ingestion_records::{IngestionJob, IngestionPublication, IngestionState, IngestionStoreError};
use anyhow::Result;
use sqlx::Row;

fn state(job: &IngestionJob) -> Result<String> {
    Ok(serde_json::to_value(job.state)?.as_str().unwrap_or_default().to_owned())
}

pub(super) async fn accept(store: &PostgresProvider, document: &KnowledgeDocument, source: &[u8]) -> Result<IngestionJob> {
    anyhow::ensure!(document.status == DocumentStatus::Pending && document.chunk_count == 0, "new upload must be pending");
    let job = IngestionJob::new(document)?;
    let namespace = job.namespace;
    let mut tx = store.pool.begin().await?;
    let kb = sqlx::query(&namespace.query("SELECT id FROM knowledge_bases WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(&document.owner_id).bind(&document.kb_id).fetch_optional(&mut *tx).await?;
    if kb.is_none() { return Err(IngestionStoreError::MissingKnowledgeBase.into()); }
    sqlx::query(&namespace.query("INSERT INTO knowledge_documents (owner_id,id,kb_id,filename,mime_type,chunk_count,status,created_at,updated_at) VALUES ($1,$2,$3,$4,$5,0,'pending',NOW(),NOW())"))
        .bind(&document.owner_id).bind(&document.id).bind(&document.kb_id).bind(&document.filename)
        .bind(document.mime_type.as_deref().unwrap_or("application/octet-stream")).execute(&mut *tx).await?;
    sqlx::query(&namespace.query("INSERT INTO knowledge_ingestion (owner_id,document_id,kb_id,state,next_attempt_at,data,source) VALUES ($1,$2,$3,'pending',$4,$5,$6)"))
        .bind(&job.owner_id).bind(&job.document_id).bind(&job.kb_id).bind(job.next_attempt_at)
        .bind(serde_json::to_value(&job)?).bind(source).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(job)
}

pub(super) async fn discover(store: &PostgresProvider, limit: usize) -> Result<Vec<IngestionJob>> {
    let rows = sqlx::query("SELECT data,principal FROM (SELECT data,next_attempt_at,owner_id,document_id,true AS principal FROM principal_knowledge_ingestion WHERE state IN ('pending','processing','retryable') UNION ALL SELECT data,next_attempt_at,owner_id,document_id,false AS principal FROM knowledge_ingestion WHERE owner_id='anonymous' AND state IN ('pending','processing','retryable')) jobs WHERE next_attempt_at <= EXTRACT(EPOCH FROM clock_timestamp()) ORDER BY next_attempt_at,owner_id,document_id LIMIT $1")
        .bind(i64::try_from(limit)?).fetch_all(&store.pool).await?;
    rows.into_iter().map(|row| {
        let job: IngestionJob = serde_json::from_value(row.try_get("data")?)?;
        job.validate_namespace()?;
        anyhow::ensure!((job.namespace == KnowledgeNamespace::PrincipalV1) == row.try_get::<bool, _>("principal")?, "ingestion record namespace differs from its table");
        Ok(job)
    }).collect()
}

pub(super) async fn next_due_at(store: &PostgresProvider) -> Result<Option<i64>> {
    let row = sqlx::query("SELECT MIN(next_attempt_at) AS due_at FROM (SELECT next_attempt_at FROM principal_knowledge_ingestion WHERE state IN ('pending','processing','retryable') AND next_attempt_at > EXTRACT(EPOCH FROM clock_timestamp()) UNION ALL SELECT next_attempt_at FROM knowledge_ingestion WHERE owner_id='anonymous' AND state IN ('pending','processing','retryable') AND next_attempt_at > EXTRACT(EPOCH FROM clock_timestamp())) jobs")
        .fetch_one(&store.pool)
        .await?;
    Ok(row.try_get("due_at")?)
}

pub(super) async fn get(store: &PostgresProvider, owner: &str, document: &str) -> Result<Option<IngestionJob>> {
    let namespace = KnowledgeNamespace::for_owner(owner)?;
    sqlx::query(&namespace.query("SELECT data FROM knowledge_ingestion WHERE owner_id=$1 AND document_id=$2"))
        .bind(owner).bind(document).fetch_optional(&store.pool).await?
        .map(|row| {
            let job: IngestionJob = serde_json::from_value(row.try_get("data")?)?;
            job.validate_namespace()?;
            anyhow::ensure!(job.namespace == namespace && job.owner_id == owner && job.document_id == document, "ingestion record identity differs from storage scope");
            Ok(job)
        }).transpose()
}

pub(super) async fn source(store: &PostgresProvider, job: &IngestionJob) -> Result<Option<Vec<u8>>> {
    let namespace = job.namespace;
    job.validate_namespace()?;
    sqlx::query(&namespace.query("SELECT source FROM knowledge_ingestion WHERE owner_id=$1 AND document_id=$2 AND data=$3"))
        .bind(&job.owner_id).bind(&job.document_id).bind(serde_json::to_value(job)?)
        .fetch_optional(&store.pool).await?.map(|row| Ok(row.try_get("source")?)).transpose()
}

pub(super) async fn transition(store: &PostgresProvider, before: &IngestionJob, after: &IngestionJob) -> Result<bool> {
    let namespace = before.namespace;
    before.validate_transition(after)?;
    let mut tx = store.pool.begin().await?;
    let doc = sqlx::query(&namespace.query("SELECT id FROM knowledge_documents WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(&before.owner_id).bind(&before.document_id).fetch_optional(&mut *tx).await?;
    if doc.is_none() { return Ok(false); }
    let updated = sqlx::query(&namespace.query("UPDATE knowledge_ingestion SET state=$4,next_attempt_at=$5,data=$6 WHERE owner_id=$1 AND document_id=$2 AND data=$3"))
        .bind(&before.owner_id).bind(&before.document_id).bind(serde_json::to_value(before)?)
        .bind(state(after)?).bind(after.next_attempt_at).bind(serde_json::to_value(after)?)
        .execute(&mut *tx).await?;
    if updated.rows_affected() != 1 { return Ok(false); }
    let document_state = match after.state {
        IngestionState::Processing => "processing",
        IngestionState::Retryable | IngestionState::Failed => "failed",
        IngestionState::Pending => "pending",
        IngestionState::Indexed => "indexed",
    };
    sqlx::query(&namespace.query("UPDATE knowledge_documents SET status=$3,error_message=$4,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
        .bind(&after.owner_id).bind(&after.document_id).bind(document_state).bind(&after.last_error).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(true)
}

pub(super) async fn publish(store: &PostgresProvider, job: &IngestionJob, publication: &IngestionPublication) -> Result<()> {
    let namespace = job.namespace;
    job.validate_namespace()?;
    publication.validate(job)?;
    let mut tx = store.pool.begin().await?;
    let kb = sqlx::query(&namespace.query("SELECT config FROM knowledge_bases WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(&job.owner_id).bind(&job.kb_id).fetch_optional(&mut *tx).await?
        .ok_or(IngestionStoreError::Revoked)?;
    let mut config: KbConfig = serde_json::from_value(kb.try_get("config")?)?;
    anyhow::ensure!(config.indexed_embedding.as_ref().is_none_or(|indexed| indexed == &publication.embedding), "embedding space changed before ingestion publication");
    config.indexed_embedding = Some(publication.embedding.clone());
    let doc = sqlx::query(&namespace.query("SELECT id FROM knowledge_documents WHERE owner_id=$1 AND id=$2 AND kb_id=$3 FOR UPDATE"))
        .bind(&job.owner_id).bind(&job.document_id).bind(&job.kb_id).fetch_optional(&mut *tx).await?;
    if doc.is_none() { return Err(IngestionStoreError::Revoked.into()); }
    let locked = sqlx::query(&namespace.query("SELECT data FROM knowledge_ingestion WHERE owner_id=$1 AND document_id=$2 AND data=$3 FOR UPDATE"))
        .bind(&job.owner_id).bind(&job.document_id).bind(serde_json::to_value(job)?).fetch_optional(&mut *tx).await?;
    if locked.is_none() { return Err(IngestionStoreError::Revoked.into()); }
    sqlx::query(&namespace.query("DELETE FROM knowledge_chunks WHERE owner_id=$1 AND document_id=$2"))
        .bind(&job.owner_id).bind(&job.document_id).execute(&mut *tx).await?;
    for chunk in &publication.chunks {
        sqlx::query(&namespace.query("INSERT INTO knowledge_chunks (id,owner_id,kb_id,document_id,content,metadata,embedding,created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,NOW())"))
            .bind(chunk.id).bind(&job.owner_id).bind(&job.kb_id).bind(&job.document_id)
            .bind(&chunk.content).bind(&chunk.metadata).bind(pgvector::Vector::from(chunk.embedding.clone()))
            .execute(&mut *tx).await?;
    }
    sqlx::query(&namespace.query("UPDATE knowledge_bases SET config=$3,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
        .bind(&job.owner_id).bind(&job.kb_id).bind(serde_json::to_value(config)?).execute(&mut *tx).await?;
    sqlx::query(&namespace.query("UPDATE knowledge_documents SET status='indexed',chunk_count=$3,error_message=NULL,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
        .bind(&job.owner_id).bind(&job.document_id).bind(i32::try_from(publication.chunks.len())?).execute(&mut *tx).await?;
    let mut indexed = job.clone();
    indexed.state = IngestionState::Indexed;
    indexed.last_error = None;
    sqlx::query(&namespace.query("UPDATE knowledge_ingestion SET state='indexed',data=$3 WHERE owner_id=$1 AND document_id=$2"))
        .bind(&job.owner_id).bind(&job.document_id).bind(serde_json::to_value(indexed)?).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

pub(super) async fn reconcile(store: &PostgresProvider) -> Result<()> {
    for namespace in [KnowledgeNamespace::Legacy, KnowledgeNamespace::PrincipalV1] {
        sqlx::query(&namespace.query("UPDATE knowledge_documents d SET status='failed',error_message='source_bytes_missing: reupload required',updated_at=NOW() WHERE ($1 OR owner_id='anonymous') AND status IN ('pending','processing') AND NOT EXISTS (SELECT 1 FROM knowledge_ingestion j WHERE j.owner_id=d.owner_id AND j.document_id=d.id)"))
            .bind(namespace == KnowledgeNamespace::PrincipalV1).execute(&store.pool).await?;
    }
    Ok(())
}
