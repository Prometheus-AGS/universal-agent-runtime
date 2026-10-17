//! Trusted embedded-host composition. Not an HTTP-configurable durability assertion.
use super::{DurableStoreEligibility, SurrealDbProvider, Any, Surreal};
use anyhow::Result;

#[derive(Debug, Clone, Copy)]
pub enum HostSurrealBackend { RocksDb, SurrealKv }

impl SurrealDbProvider {
    /// Reuse the host's already selected namespace/database and engine handle.
    /// The caller owns its lifecycle; this never reconnects, changes its selection,
    /// or substitutes another storage engine for existing user data.
    pub async fn from_existing_client(db: Surreal<Any>, backend: HostSurrealBackend) -> Result<Self> {
        migrate(&db).await?;
        Ok(Self { db, durable_instances: true,
            durable_store_eligibility: DurableStoreEligibility::LocalDurable,
            catalog_storage_backend: match backend { HostSurrealBackend::RocksDb => "rocksdb", HostSurrealBackend::SurrealKv => "surrealkv" },
            remote_requires_durability_attestation: false })
    }
}
pub(super) async fn migrate(db: &Surreal<Any>) -> Result<()> {
    db.query(include_str!(
        "../../../../migrations/surrealdb/durable_api_keys.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/principal_credentials.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/credential_revisions.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/agent_threads.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/agent_instances.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/observers.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/channel_observers.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/canonical_tool_receipts.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!("../../../../migrations/surrealdb/approval_records.surql")).await?.check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/tool_admission_evidence.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/presentations.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/principal_conversation_policies.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!(
        "../../../../migrations/surrealdb/collaboration_catalog.surql"
    ))
    .await?
    .check()?;

    db.query(include_str!("../../../../migrations/surrealdb/supervisor_fencing.surql")).await?.check()?;
    super::checkpoint_records::initialize(db).await?;
    super::ingestion_records::initialize(db).await?;
    super::accounting_records::initialize(db).await?;
    db.query(include_str!("../../../../migrations/surrealdb/a2a_bindings.surql")).await?.check()?;
    db.query(include_str!("../../../../migrations/surrealdb/agui_runs.surql")).await?.check()?;
    db.query(include_str!("../../../../migrations/surrealdb/mcp_elicitations.surql")).await?.check()?;

    Ok(())
}
