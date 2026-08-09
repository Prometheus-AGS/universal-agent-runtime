//! Transport-free knowledge-base administration over `PersistenceLayer`.

use std::sync::Arc;

use crate::uar::domain::knowledge::KnowledgeBase;
use crate::uar::persistence::PersistenceLayer;

pub async fn list(store: &Arc<dyn PersistenceLayer>) -> anyhow::Result<Vec<KnowledgeBase>> {
    store.list_knowledge_bases().await
}

pub async fn get(
    store: &Arc<dyn PersistenceLayer>,
    id: &str,
) -> anyhow::Result<Option<KnowledgeBase>> {
    store.get_knowledge_base(id).await
}

pub async fn save(store: &Arc<dyn PersistenceLayer>, kb: &KnowledgeBase) -> anyhow::Result<()> {
    store.save_knowledge_base(kb).await
}

pub async fn delete(store: &Arc<dyn PersistenceLayer>, id: &str) -> anyhow::Result<()> {
    store.delete_knowledge_base(id).await
}
