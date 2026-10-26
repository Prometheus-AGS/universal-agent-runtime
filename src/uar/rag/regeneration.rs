//! Durable regeneration rules shared by host ingestion and storage transactions.
//! Scheduling remains the existing supervised retained-source ingestion pool.

use crate::uar::domain::knowledge::{EmbeddingFingerprint, KbConfig, KnowledgeBase,
    KnowledgeRegenerationState};
use crate::uar::persistence::ingestion_records::{IngestionJob, IngestionState};

#[derive(Debug, thiserror::Error)]
pub enum RegenerationError {
    #[error("knowledge regeneration is already running or the cohort changed")]
    Conflict,
    #[error("retained source bytes are missing; reupload is required")]
    SourceMissing,
}

/// Admit only the exact current cohort/target, or a normal compatible upload.
pub fn validate_preparation(kb: &KnowledgeBase, job: &IngestionJob,
    active: &EmbeddingFingerprint) -> anyhow::Result<()> {
    validate_config(&kb.config, job, active)
}

pub fn validate_config(config: &KbConfig, job: &IngestionJob,
    active: &EmbeddingFingerprint) -> anyhow::Result<()> {
    if let Some(generation) = job.regeneration.as_deref() {
        let cohort = config.regeneration.as_ref()
            .ok_or_else(|| anyhow::anyhow!("regeneration cohort was revoked"))?;
        anyhow::ensure!(cohort.generation == generation
            && cohort.state == KnowledgeRegenerationState::Running
            && cohort.document_ids.contains(&job.document_id), "regeneration cohort was revoked");
        anyhow::ensure!(&cohort.target == active, "active embedding backend differs from durable regeneration target");
    } else {
        anyhow::ensure!(!config.regeneration.as_ref().is_some_and(|row|
            row.state != KnowledgeRegenerationState::Complete), "knowledge regeneration is incomplete");
        anyhow::ensure!(config.indexed_embedding.as_ref().is_none_or(|indexed| indexed == active),
            "embedding space changed before ingestion publication");
    }
    Ok(())
}

/// Recompute durable progress from the transaction's exact member job records.
/// The caller commits this alongside job/chunk changes under the KB write fence.
pub fn settle_progress(config: &mut KbConfig, jobs: &[IngestionJob]) -> anyhow::Result<()> {
    let Some(cohort) = config.regeneration.as_mut() else { return Ok(()); };
    if cohort.state != KnowledgeRegenerationState::Running { return Ok(()); }
    let mut completed = 0;
    let mut terminal_error = None;
    let mut last_error = None;
    for document in &cohort.document_ids {
        let job = jobs.iter().find(|job| job.document_id == *document
            && job.regeneration.as_deref() == Some(cohort.generation.as_str()))
            .ok_or_else(|| anyhow::anyhow!("regeneration member attempt is missing or revoked"))?;
        if job.state == IngestionState::Indexed { completed += 1; }
        if job.last_error.is_some() { last_error = job.last_error.clone(); }
        if job.state == IngestionState::Failed { terminal_error = job.last_error.clone(); }
    }
    cohort.completed_documents = completed;
    cohort.last_error = terminal_error.clone().or(last_error);
    cohort.updated_at = chrono::Utc::now().to_rfc3339();
    if terminal_error.is_some() || jobs.iter().any(|job| job.regeneration.as_deref()
        == Some(cohort.generation.as_str()) && job.state == IngestionState::Failed) {
        cohort.state = KnowledgeRegenerationState::Failed;
    } else if completed == cohort.document_ids.len() {
        cohort.state = KnowledgeRegenerationState::Complete;
        cohort.last_error = None;
        config.indexed_embedding = Some(cohort.target.clone());
        config.embedding_provider.clone_from(&cohort.target.backend);
        config.embedding_model.clone_from(&cohort.target.model);
        config.vector_dimensions = Some(cohort.target.dimension);
    }
    Ok(())
}
