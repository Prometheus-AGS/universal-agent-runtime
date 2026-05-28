use anyhow::{Context, Result};
use std::sync::Arc;
use universal_agent_runtime::config::AppConfig;
use universal_agent_runtime::uar::defaults::ensure_default_knowledge_base;
use universal_agent_runtime::uar::persistence::PersistenceLayer;
#[cfg(feature = "postgres-backend")]
use universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider;
use universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load().context("failed to load app configuration")?;

    if !config.file_processing.upload_dir.is_empty() {
        tokio::fs::create_dir_all(&config.file_processing.upload_dir)
            .await
            .context("failed to create upload directory")?;
    }

    let persistence: Arc<dyn PersistenceLayer> = if matches!(
        config.persistence.provider.as_str(),
        "surreal" | "surrealdb"
    ) {
        Arc::new(
            SurrealDbProvider::new(&config.persistence.database_url, None, None, None, None)
                .await
                .context("failed to initialize SurrealDB")?,
        )
    } else {
        #[cfg(feature = "postgres-backend")]
        {
            Arc::new(
                PostgresProvider::new(&config.persistence.database_url)
                    .await
                    .context("failed to initialize Postgres")?,
            )
        }
        #[cfg(not(feature = "postgres-backend"))]
        {
            anyhow::bail!(
                "Postgres persistence requested but the `postgres-backend` Cargo \
                 feature is disabled. Either rebuild with --features postgres-backend \
                 or set persistence.provider = \"surreal\" in your config."
            );
        }
    };

    ensure_default_knowledge_base(&*persistence, config.knowledge_bases.default.as_ref())
        .await
        .context("failed to ensure default knowledge base")?;

    Ok(())
}
