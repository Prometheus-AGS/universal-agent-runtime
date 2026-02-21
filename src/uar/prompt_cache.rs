use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// Cached prompt metadata for Anthropic-compatible prompt caching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedEntry {
    /// Number of input tokens represented by this cached block.
    pub token_count: u32,
    /// Opaque compiled representation of the block.
    pub compiled_representation: Vec<u8>,
    /// Creation timestamp in UTC.
    pub created_at: DateTime<Utc>,
}

/// Abstract cache provider for prompt-caching backends.
#[async_trait]
pub trait PromptCacheProvider: std::fmt::Debug + Send + Sync {
    /// Lookup a cached entry by normalized content hash.
    async fn get(&self, hash: &str) -> Option<CachedEntry>;
    /// Store or overwrite a cached entry.
    async fn set(&self, hash: &str, entry: CachedEntry) -> Result<()>;
    /// Delete one cached entry.
    async fn delete(&self, hash: &str) -> Result<()>;
    /// Clear all cached entries.
    async fn clear(&self) -> Result<()>;
}

/// Lock-free in-memory prompt cache backed by a `HashMap`.
///
/// Used as the default provider when no external cache backend is configured.
/// Suitable for development, testing, and single-process deployments where
/// cross-process cache sharing is not required.
#[derive(Debug, Clone)]
pub struct SurrealMemPromptCacheProvider {
    store: Arc<RwLock<HashMap<String, CachedEntry>>>,
}

impl SurrealMemPromptCacheProvider {
    /// Create a new empty in-memory prompt cache.
    pub async fn new() -> Result<Self> {
        Ok(Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl PromptCacheProvider for SurrealMemPromptCacheProvider {
    async fn get(&self, hash: &str) -> Option<CachedEntry> {
        self.store.read().await.get(hash).cloned()
    }

    async fn set(&self, hash: &str, entry: CachedEntry) -> Result<()> {
        self.store.write().await.insert(hash.to_owned(), entry);
        Ok(())
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        self.store.write().await.remove(hash);
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.store.write().await.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn surreal_mem_cache_round_trip() {
        let cache = SurrealMemPromptCacheProvider::new()
            .await
            .expect("cache should initialize");
        let entry = CachedEntry {
            token_count: 42,
            compiled_representation: b"compiled".to_vec(),
            created_at: Utc::now(),
        };

        cache
            .set("abc123", entry.clone())
            .await
            .expect("set should succeed");
        let fetched = cache.get("abc123").await.expect("entry should exist");

        assert_eq!(fetched.token_count, entry.token_count);
        assert_eq!(
            fetched.compiled_representation,
            entry.compiled_representation
        );
    }

    #[tokio::test]
    async fn surreal_mem_cache_delete_and_clear() {
        let cache = SurrealMemPromptCacheProvider::new()
            .await
            .expect("cache should initialize");

        cache
            .set(
                "one",
                CachedEntry {
                    token_count: 1,
                    compiled_representation: vec![1],
                    created_at: Utc::now(),
                },
            )
            .await
            .expect("set should succeed");
        cache
            .set(
                "two",
                CachedEntry {
                    token_count: 2,
                    compiled_representation: vec![2],
                    created_at: Utc::now(),
                },
            )
            .await
            .expect("set should succeed");

        cache.delete("one").await.expect("delete should succeed");
        assert!(cache.get("one").await.is_none());
        assert!(cache.get("two").await.is_some());

        cache.clear().await.expect("clear should succeed");
        assert!(cache.get("two").await.is_none());
    }
}
