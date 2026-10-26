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
    let kb = sqlx::query(&namespace.query("SELECT config FROM knowledge_bases WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(&document.owner_id).bind(&document.kb_id).fetch_optional(&mut *tx).await?;
    if kb.is_none() { return Err(IngestionStoreError::MissingKnowledgeBase.into()); }
    let config: KbConfig = serde_json::from_value(kb.ok_or(IngestionStoreError::MissingKnowledgeBase)?.try_get("config")?)?;
    anyhow::ensure!(!config.regeneration.as_ref().is_some_and(|row| row.state != crate::uar::domain::knowledge::KnowledgeRegenerationState::Complete),
        crate::uar::rag::regeneration::RegenerationError::Conflict);
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
    let rows = sqlx::query("SELECT data,principal FROM (SELECT j.data,j.next_attempt_at,j.owner_id,j.document_id,true AS principal FROM principal_knowledge_ingestion j JOIN principal_knowledge_bases b ON b.owner_id=j.owner_id AND b.id=j.kb_id WHERE j.state IN ('pending','processing','retryable') AND COALESCE(b.config->'regeneration'->>'state','complete')!='failed' UNION ALL SELECT j.data,j.next_attempt_at,j.owner_id,j.document_id,false AS principal FROM knowledge_ingestion j JOIN knowledge_bases b ON b.owner_id=j.owner_id AND b.id=j.kb_id WHERE j.owner_id='anonymous' AND j.state IN ('pending','processing','retryable') AND COALESCE(b.config->'regeneration'->>'state','complete')!='failed') jobs WHERE next_attempt_at <= EXTRACT(EPOCH FROM clock_timestamp()) ORDER BY next_attempt_at,owner_id,document_id LIMIT $1")
        .bind(i64::try_from(limit)?).fetch_all(&store.pool).await?;
    rows.into_iter().map(|row| {
        let job: IngestionJob = serde_json::from_value(row.try_get("data")?)?;
        job.validate_namespace()?;
        anyhow::ensure!((job.namespace == KnowledgeNamespace::PrincipalV1) == row.try_get::<bool, _>("principal")?, "ingestion record namespace differs from its table");
        Ok(job)
    }).collect()
}

pub(super) async fn next_due_at(store: &PostgresProvider) -> Result<Option<i64>> {
    let row = sqlx::query("SELECT MIN(next_attempt_at) AS due_at FROM (SELECT j.next_attempt_at FROM principal_knowledge_ingestion j JOIN principal_knowledge_bases b ON b.owner_id=j.owner_id AND b.id=j.kb_id WHERE j.state IN ('pending','processing','retryable') AND COALESCE(b.config->'regeneration'->>'state','complete')!='failed' AND j.next_attempt_at > EXTRACT(EPOCH FROM clock_timestamp()) UNION ALL SELECT j.next_attempt_at FROM knowledge_ingestion j JOIN knowledge_bases b ON b.owner_id=j.owner_id AND b.id=j.kb_id WHERE j.owner_id='anonymous' AND j.state IN ('pending','processing','retryable') AND COALESCE(b.config->'regeneration'->>'state','complete')!='failed' AND j.next_attempt_at > EXTRACT(EPOCH FROM clock_timestamp())) jobs")
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
    let kb = sqlx::query(&namespace.query("SELECT config FROM knowledge_bases WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(&before.owner_id).bind(&before.kb_id).fetch_optional(&mut *tx).await?;
    let Some(kb) = kb else { return Ok(false); };
    let mut config: KbConfig = serde_json::from_value(kb.try_get("config")?)?;
    if let Some(generation) = &before.regeneration {
        if !config.regeneration.as_ref().is_some_and(|row| &row.generation == generation
            && row.state == crate::uar::domain::knowledge::KnowledgeRegenerationState::Running) { return Ok(false); }
    } else if config.regeneration.as_ref().is_some_and(|row| row.state != crate::uar::domain::knowledge::KnowledgeRegenerationState::Complete) {
        return Ok(false);
    }
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
    progress(&mut tx, namespace, &before.owner_id, &before.kb_id, &mut config).await?;
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
    crate::uar::rag::regeneration::validate_config(&config, job, &publication.embedding)?;
    if job.regeneration.is_none() { config.indexed_embedding = Some(publication.embedding.clone()); }
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
    sqlx::query(&namespace.query("UPDATE knowledge_documents SET status='indexed',chunk_count=$3,error_message=NULL,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
        .bind(&job.owner_id).bind(&job.document_id).bind(i32::try_from(publication.chunks.len())?).execute(&mut *tx).await?;
    let mut indexed = job.clone();
    indexed.state = IngestionState::Indexed;
    indexed.last_error = None;
    sqlx::query(&namespace.query("UPDATE knowledge_ingestion SET state='indexed',data=$3 WHERE owner_id=$1 AND document_id=$2"))
        .bind(&job.owner_id).bind(&job.document_id).bind(serde_json::to_value(indexed)?).execute(&mut *tx).await?;
    progress(&mut tx, namespace, &job.owner_id, &job.kb_id, &mut config).await?;
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

pub(super) async fn begin_regeneration(store: &PostgresProvider, owner: &str, kb_id: &str,
    target: &crate::uar::domain::knowledge::EmbeddingFingerprint)
    -> Result<crate::uar::domain::knowledge::KnowledgeRegeneration> {
    use crate::uar::domain::knowledge::{KnowledgeRegeneration, KnowledgeRegenerationState};
    use crate::uar::rag::regeneration::RegenerationError;
    let namespace = KnowledgeNamespace::for_owner(owner)?;
    let mut tx = store.pool.begin().await?;
    let row = sqlx::query(&namespace.query("SELECT config FROM knowledge_bases WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(owner).bind(kb_id).fetch_optional(&mut *tx).await?.ok_or(IngestionStoreError::MissingKnowledgeBase)?;
    let mut config: KbConfig = serde_json::from_value(row.try_get("config")?)?;
    if config.regeneration.as_ref().is_some_and(|row| row.state == KnowledgeRegenerationState::Running) {
        return Err(RegenerationError::Conflict.into());
    }
    let documents = sqlx::query(&namespace.query("SELECT id FROM knowledge_documents WHERE owner_id=$1 AND kb_id=$2 ORDER BY id FOR UPDATE"))
        .bind(owner).bind(kb_id).fetch_all(&mut *tx).await?;
    let ids = documents.iter().map(|row| row.try_get::<String,_>("id")).collect::<std::result::Result<Vec<_>,_>>()?;
    let orphan = sqlx::query(&namespace.query("SELECT c.id FROM knowledge_chunks c LEFT JOIN knowledge_documents d ON d.owner_id=c.owner_id AND d.id=c.document_id WHERE c.owner_id=$1 AND c.kb_id=$2 AND d.id IS NULL LIMIT 1"))
        .bind(owner).bind(kb_id).fetch_optional(&mut *tx).await?;
    anyhow::ensure!(orphan.is_none(), RegenerationError::SourceMissing);
    let cohort = KnowledgeRegeneration::new(config.indexed_embedding.clone(), target.clone(), ids.clone());
    for document in ids {
        let row = sqlx::query(&namespace.query("SELECT data,source IS NOT NULL AS retained FROM knowledge_ingestion WHERE owner_id=$1 AND document_id=$2 FOR UPDATE"))
            .bind(owner).bind(&document).fetch_optional(&mut *tx).await?.ok_or(RegenerationError::SourceMissing)?;
        anyhow::ensure!(row.try_get::<bool,_>("retained")?, RegenerationError::SourceMissing);
        let old: IngestionJob = serde_json::from_value(row.try_get("data")?)?;
        old.validate_namespace()?;
        anyhow::ensure!(old.owner_id == owner && old.kb_id == kb_id && old.document_id == document, "retained source ownership mismatch");
        let mut pending = old.retry();
        pending.regeneration = Some(cohort.generation.clone());
        sqlx::query(&namespace.query("UPDATE knowledge_ingestion SET state='pending',next_attempt_at=$3,data=$4 WHERE owner_id=$1 AND document_id=$2"))
            .bind(owner).bind(&document).bind(pending.next_attempt_at).bind(serde_json::to_value(pending)?).execute(&mut *tx).await?;
        sqlx::query(&namespace.query("UPDATE knowledge_documents SET status='pending',error_message=NULL,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
            .bind(owner).bind(&document).execute(&mut *tx).await?;
    }
    if cohort.state == KnowledgeRegenerationState::Complete {
        config.indexed_embedding = Some(target.clone());
        config.embedding_provider.clone_from(&target.backend);
        config.embedding_model.clone_from(&target.model);
        config.vector_dimensions = Some(target.dimension);
    }
    config.regeneration = Some(cohort.clone());
    sqlx::query(&namespace.query("UPDATE knowledge_bases SET config=$3,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
        .bind(owner).bind(kb_id).bind(serde_json::to_value(config)?).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(cohort)
}

async fn progress(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, namespace: KnowledgeNamespace,
    owner: &str, kb: &str, config: &mut KbConfig) -> Result<()> {
    let rows = sqlx::query(&namespace.query("SELECT data FROM knowledge_ingestion WHERE owner_id=$1 AND kb_id=$2"))
        .bind(owner).bind(kb).fetch_all(&mut **tx).await?;
    let jobs = rows.into_iter().map(|row| Ok(serde_json::from_value::<IngestionJob>(row.try_get("data")?)?))
        .collect::<Result<Vec<_>>>()?;
    crate::uar::rag::regeneration::settle_progress(config, &jobs)?;
    sqlx::query(&namespace.query("UPDATE knowledge_bases SET config=$3,updated_at=NOW() WHERE owner_id=$1 AND id=$2"))
        .bind(owner).bind(kb).bind(serde_json::to_value(config)?).execute(&mut **tx).await?;
    Ok(())
}

pub(super) async fn search_in_space(store: &PostgresProvider, owner: &str, ids: Option<&[&str]>,
    query: &[f32], limit: usize, min_score: f32,
    space: Option<&crate::uar::domain::knowledge::EmbeddingFingerprint>)
    -> Result<Vec<crate::uar::domain::knowledge::KnowledgeMatch>> {
    let namespace = KnowledgeNamespace::for_owner(owner)?;
    let selected: Vec<String> = ids.unwrap_or_default().iter().map(|id| (*id).to_owned()).collect();
    let rows = sqlx::query(&namespace.query("WITH eligible AS MATERIALIZED (
        SELECT c.* FROM knowledge_chunks c JOIN knowledge_bases b ON b.owner_id=c.owner_id AND b.id=c.kb_id
        WHERE c.owner_id=$2 AND ($3 OR c.kb_id=ANY($4))
        AND COALESCE(b.config->'regeneration'->>'state','complete')='complete'
        AND ($5::jsonb IS NULL OR b.config->'indexed_embedding'=$5)
        AND vector_dims(c.embedding)=vector_dims($1))
        SELECT id,owner_id,kb_id,document_id,content,metadata,created_at,1-(embedding <=> $1) AS score
        FROM eligible WHERE 1-(embedding <=> $1)>=$6 ORDER BY embedding <=> $1 LIMIT $7"))
        .bind(pgvector::Vector::from(query.to_vec())).bind(owner).bind(ids.is_none()).bind(selected)
        .bind(space.map(serde_json::to_value).transpose()?).bind(f64::from(min_score)).bind(i64::try_from(limit)?)
        .fetch_all(&store.pool).await?;
    rows.into_iter().map(|row| {
        let created: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
        Ok(crate::uar::domain::knowledge::KnowledgeMatch { chunk: crate::uar::domain::knowledge::KnowledgeChunk {
            id: row.try_get("id")?, owner_id: row.try_get("owner_id")?, kb_id: row.try_get("kb_id")?,
            document_id: row.try_get("document_id")?, content: row.try_get("content")?, metadata: row.try_get("metadata")?,
            embedding: Vec::new(), created_at: created.map(|at| at.to_rfc3339()).unwrap_or_default(),
        }, score: row.try_get::<f64,_>("score")? as f32 })
    }).collect()
}

pub(super) async fn lock_writable_kb(tx: &mut sqlx::Transaction<'_,sqlx::Postgres>,
    namespace: KnowledgeNamespace, owner: &str, kb: &str) -> Result<KbConfig> {
    let row = sqlx::query(&namespace.query("SELECT config FROM knowledge_bases WHERE owner_id=$1 AND id=$2 FOR UPDATE"))
        .bind(owner).bind(kb).fetch_optional(&mut **tx).await?.ok_or(IngestionStoreError::MissingKnowledgeBase)?;
    let config: KbConfig = serde_json::from_value(row.try_get("config")?)?;
    anyhow::ensure!(!config.regeneration.as_ref().is_some_and(|job| job.state != crate::uar::domain::knowledge::KnowledgeRegenerationState::Complete),
        crate::uar::rag::regeneration::RegenerationError::Conflict);
    Ok(config)
}
