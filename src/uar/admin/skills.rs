//! Transport-free skill administration over `PersistenceLayer`.
//!
//! The persistence methods (`save_skill`, `list_skills`, `delete_skill`,
//! `search_skills`) already exist and are implemented by `SurrealDbProvider`,
//! so skills were ALWAYS database-backed — they were simply unreachable from an
//! embedded container because only the HTTP handlers called them.

use std::sync::Arc;

use crate::uar::domain::skills::Skill;
use crate::uar::persistence::PersistenceLayer;

pub async fn list(store: &Arc<dyn PersistenceLayer>) -> anyhow::Result<Vec<Skill>> {
    store.list_skills().await
}

/// Persist a skill with its embedding so semantic matching works immediately.
///
/// The embedding is supplied by the caller rather than computed here: the
/// embedding backend is a runtime concern the admin layer has no business
/// owning, and an embedded device may use a different one than a server.
pub async fn save(
    store: &Arc<dyn PersistenceLayer>,
    skill: &Skill,
    embedding: &[f32],
) -> anyhow::Result<()> {
    store.save_skill(skill, embedding).await
}

pub async fn delete(store: &Arc<dyn PersistenceLayer>, id: &str) -> anyhow::Result<()> {
    store.delete_skill(id).await
}
