use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::any::{self, Any};

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

/// In-memory embedded SurrealDB cache provider (`mem://`).
#[derive(Debug, Clone)]
pub struct SurrealMemPromptCacheProvider {
    db: Surreal<Any>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptCacheRecord {
    token_count: u32,
    compiled_representation: Vec<u8>,
    created_at: DateTime<Utc>,
}

impl SurrealMemPromptCacheProvider {
    /// Create a new in-memory prompt cache.
    pub async fn new() -> Result<Self> {
        let db = any::connect("mem://").await?;
        db.use_ns("uar").use_db("prompt_cache").await?;
        Ok(Self { db })
    }
}

#[async_trait]
impl PromptCacheProvider for SurrealMemPromptCacheProvider {
    async fn get(&self, hash: &str) -> Option<CachedEntry> {
        let result: anyhow::Result<Option<serde_json::Value>> = self
            .db
            .select(("prompt_cache", hash))
            .await
            .map_err(Into::into);

        match result {
            Ok(Some(record)) => match serde_json::from_value::<PromptCacheRecord>(record) {
                Ok(parsed) => Some(CachedEntry {
                    token_count: parsed.token_count,
                    compiled_representation: parsed.compiled_representation,
                    created_at: parsed.created_at,
                }),
                Err(err) => {
                    tracing::warn!(error = %err, hash, "Prompt cache decode failed");
                    None
                }
            },
            Ok(None) => None,
            Err(err) => {
                tracing::warn!(error = %err, hash, "Prompt cache get failed");
                None
            }
        }
    }

    async fn set(&self, hash: &str, entry: CachedEntry) -> Result<()> {
        let record = PromptCacheRecord {
            token_count: entry.token_count,
            compiled_representation: entry.compiled_representation,
            created_at: entry.created_at,
        };
        let payload = serde_json::to_value(record)?;
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("prompt_cache", hash))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        let _: Option<serde_json::Value> = self.db.delete(("prompt_cache", hash)).await?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let _ = self.db.query("DELETE prompt_cache").await?;
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
