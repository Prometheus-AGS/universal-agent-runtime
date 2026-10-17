//! Explicit local metadata initialization. Never invoked by ordinary startup.
use super::PrincipalCredentialStore;
pub use super::migration::RevisionInitializationSummary;

/// Connect only the selected credential backend after the operator acknowledges
/// quiescing every writer and consumer. This starts no listener or provider.
pub async fn initialize_revisions(config: &crate::config::AppConfig, writers_quiesced: bool) -> anyhow::Result<RevisionInitializationSummary> {
    if !writers_quiesced { anyhow::bail!("credential revision initialization requires --writers-quiesced"); }
    initialize_backend(config).await.map_err(|_| anyhow::anyhow!("credential revision initialization unavailable"))
}
async fn initialize_backend(config: &crate::config::AppConfig) -> anyhow::Result<RevisionInitializationSummary> {
    let settings = &config.persistence;
    match settings.provider.as_str() {
        "surreal" | "surrealdb" => {
            #[cfg(feature = "surreal-backend")]
            {
                use surrealdb::opt::auth::{Root, Namespace, Database};
                let endpoint = if let Some(rest) = settings.database_url.strip_prefix("https://") { format!("wss://{rest}") }
                    else if let Some(rest) = settings.database_url.strip_prefix("http://") { format!("ws://{rest}") } else { settings.database_url.clone() };
                let db = surrealdb::engine::any::connect(&endpoint).await?;
                let namespace = settings.surreal_ns.as_deref().unwrap_or("uar");
                let database = settings.surreal_db.as_deref().unwrap_or("uar");
                if endpoint.starts_with("ws://") || endpoint.starts_with("wss://") {
                    let username = settings.surreal_user.clone().unwrap_or_else(|| "root".to_owned());
                    let password = settings.surreal_pass.clone().unwrap_or_else(|| "root".to_owned());
                    match settings.surreal_auth_level.as_deref().unwrap_or("root") {
                        "root" => { db.signin(Root { username, password }).await?; }
                        "namespace" => { db.signin(Namespace { namespace: namespace.to_owned(), username, password }).await?; }
                        "database" => { db.signin(Database { namespace: namespace.to_owned(), database: database.to_owned(), username, password }).await?; }
                        _ => anyhow::bail!("invalid credential backend authentication"),
                    }
                }
                db.use_ns(namespace).use_db(database).await?;
                db.query(include_str!("../../../../migrations/surrealdb/credential_revisions.surql")).await?.check()?;
                return super::SurrealCredentialStore::new(db).initialize_revisions().await;
            }
            #[cfg(not(feature = "surreal-backend"))]
            anyhow::bail!("credential backend unavailable")
        }
        "postgres" | "postgresql" => {
            #[cfg(feature = "sqlx")]
            {
                let pool = sqlx::PgPool::connect(&settings.database_url).await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                let result = super::PostgresCredentialStore::new(pool.clone()).initialize_revisions().await;
                pool.close().await;
                return result;
            }
            #[cfg(not(feature = "sqlx"))]
            anyhow::bail!("credential backend unavailable")
        }
        // Process-local memory has no historical state to upgrade across invocations.
        _ => anyhow::bail!("credential initialization requires a durable backend"),
    }
}
