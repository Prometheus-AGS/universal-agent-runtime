//! Auth-free real embedded-store migration boundary for #421/#465.
//! Source authoring only: execution belongs to the approved whole-set batch.
#![cfg(feature = "surreal-backend")]

use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::Value;
use universal_agent_runtime::uar::persistence::{
    PersistenceLayer,
    providers::surreal::{HostSurrealBackend, SurrealDbProvider},
};

#[tokio::test]
async fn concurrent_real_migration_facades_preserve_owned_data_and_reopen() -> Result<()> {
    let directory = tempfile::tempdir().context("owned migration directory")?;
    let endpoint = format!(
        "surrealkv://{}",
        directory.path().join("migration.db").display()
    );
    let host = surrealdb::engine::any::connect(&endpoint)
        .await
        .context("owned embedded engine")?;
    let outcome: Result<()> = async {
        host.use_ns("production-readiness")
            .use_db("concurrent-migrations")
            .await?;
        host.query("CREATE migration_canary:owned SET marker = 'PR476-MIGRATION-7319';")
            .await?
            .check()?;

        // Both production constructors issue their actual migration sequence
        // against the selected real host engine. No mock migration or source
        // text assertion substitutes for successful schema initialization.
        let migrations = tokio::time::timeout(Duration::from_secs(90), async {
            tokio::join!(
                SurrealDbProvider::from_existing_client(
                    host.clone(),
                    HostSurrealBackend::SurrealKv
                ),
                SurrealDbProvider::from_existing_client(
                    host.clone(),
                    HostSurrealBackend::SurrealKv
                ),
            )
        })
        .await
        .context("concurrent migration constructors did not complete")?;
        let first = migrations.0.context("first actual production migration")?;
        let second = migrations.1.context("second actual production migration")?;
        first
            .list_settings_types()
            .await
            .context("first migrated facade works")?;
        second
            .list_settings_types()
            .await
            .context("second migrated facade works")?;
        let mut schema = host.query("INFO FOR DB;").await?.check()?;
        let schema = schema.take::<surrealdb::types::Value>(0)?.into_json_value();
        for table in [
            "agent_threads",
            "canonical_tool_receipts",
            "agui_runs",
            "durable_run_records",
            "durable_run_events",
        ] {
            anyhow::ensure!(
                schema["tables"].get(table).is_some(),
                "production migration omitted table {table}"
            );
        }
        let mut readback = host
            .query("SELECT marker FROM migration_canary:owned;")
            .await?
            .check()?;
        let rows: Vec<Value> = readback.take(0)?;
        assert_eq!(
            rows,
            vec![serde_json::json!({"marker":"PR476-MIGRATION-7319"})]
        );
        for provider in [first, second] {
            let close = provider.close_scope().await;
            anyhow::ensure!(
                close.failure.is_none() && close.scope_released,
                "borrowed migration facade did not close: {close:?}"
            );
            anyhow::ensure!(
                !close.engine_close_acknowledged,
                "borrower must not close the host engine"
            );
        }
        // Reapplying the production migration after both facades release is
        // distinct from opening an empty replacement store.
        let reapplied =
            SurrealDbProvider::from_existing_client(host.clone(), HostSurrealBackend::SurrealKv)
                .await?;
        let close = reapplied.close_scope().await;
        anyhow::ensure!(
            close.failure.is_none() && close.scope_released,
            "reapplied facade did not release"
        );
        Ok(())
    }
    .await;
    let closed = host
        .shutdown()
        .await
        .context("acknowledged owned engine shutdown");
    outcome?;
    closed?;

    let reopened = surrealdb::engine::any::connect(&endpoint)
        .await
        .context("one immediate reopen")?;
    let retained: Result<()> = async {
        reopened
            .use_ns("production-readiness")
            .use_db("concurrent-migrations")
            .await?;
        let mut response = reopened
            .query("SELECT marker FROM migration_canary:owned;")
            .await?
            .check()?;
        let rows: Vec<Value> = response.take(0)?;
        assert_eq!(
            rows,
            vec![serde_json::json!({"marker":"PR476-MIGRATION-7319"})]
        );
        Ok(())
    }
    .await;
    let closed = reopened.shutdown().await.context("close reopened engine");
    retained?;
    closed?;
    Ok(())
}

#[tokio::test]
async fn agui_context_assertion_rolls_back_staged_run_and_preserves_owner_scope_after_reopen(
) -> Result<()> {
    let directory = tempfile::tempdir().context("create disposable SurrealKV directory")?;
    let endpoint = format!("surrealkv://{}", directory.path().display());
    let namespace = "production-readiness";
    let database = "agui-atomicity";
    let rejected = agui_run_binding("owner-rejected");
    let accepted = agui_run_binding("owner-accepted");
    let rejected_key = rejected.storage_key();

    anyhow::ensure!(
        accepted.storage_key() != rejected_key,
        "fixture owners must produce distinct durable run keys"
    );
    anyhow::ensure!(
        accepted.context_key() != rejected.context_key(),
        "fixture owners must produce distinct context slots"
    );

    let host = surrealdb::engine::any::connect(&endpoint)
        .await
        .context("connect disposable SurrealKV host")?;
    let first_open: Result<()> = async {
        host.use_ns(namespace)
            .use_db(database)
            .await
            .context("select disposable AG-UI namespace and database")?;
        let provider = SurrealDbProvider::from_existing_client(
            host.clone(),
            HostSurrealBackend::SurrealKv,
        )
        .await
        .context("construct borrowed SurrealDB provider")?;

        host.query(format!(
            "DEFINE FIELD OVERWRITE run_key ON agui_contexts TYPE string ASSERT $value != '{rejected_key}';"
        ))
        .await
        .context("install AG-UI context fault assertion")?
        .check()
        .context("apply AG-UI context fault assertion")?;

        let failure = provider
            .create_agui_run(&rejected, None)
            .await
            .expect_err("context assertion must reject the synthetic run key");
        anyhow::ensure!(
            failure.downcast_ref::<surrealdb::Error>().is_some(),
            "context assertion must surface the typed SurrealDB error, got {failure:#}"
        );
        anyhow::ensure!(
            provider
                .load_agui_run(
                    &rejected.owner,
                    &rejected.agent_id,
                    &rejected.thread_id,
                    &rejected.wire_run_id,
                )
                .await
                .context("load rejected run before reopen")?
                .is_none(),
            "rejected run was visible before reopen"
        );

        anyhow::ensure!(
            provider
                .create_agui_run(&accepted, None)
                .await
                .context("create accepted run under the same assertion")?,
            "accepted run unexpectedly conflicted"
        );
        anyhow::ensure!(
            provider
                .load_agui_run(
                    &rejected.owner,
                    &accepted.agent_id,
                    &accepted.thread_id,
                    &accepted.wire_run_id,
                )
                .await
                .context("load accepted selectors as rejected owner")?
                .is_none(),
            "owner-scoped lookup exposed the accepted owner's run"
        );
        anyhow::ensure!(
            provider
                .load_agui_run(
                    &accepted.owner,
                    &accepted.agent_id,
                    &accepted.thread_id,
                    &accepted.wire_run_id,
                )
                .await
                .context("load accepted run before reopen")?
                .as_ref()
                == Some(&accepted),
            "accepted run did not round-trip before reopen"
        );

        let close = provider.close_scope().await;
        anyhow::ensure!(
            close.failure.is_none() && close.scope_released,
            "borrowed AG-UI provider did not release: {close:?}"
        );
        anyhow::ensure!(
            !close.engine_close_acknowledged,
            "borrowed AG-UI provider must not close the host engine"
        );
        Ok(())
    }
    .await;
    let host_close = host.shutdown().await.context("close initial SurrealKV host");
    first_open?;
    host_close?;

    let reopened = surrealdb::engine::any::connect(&endpoint)
        .await
        .context("reopen disposable SurrealKV host")?;
    let reopened_result: Result<()> = async {
        reopened
            .use_ns(namespace)
            .use_db(database)
            .await
            .context("select reopened AG-UI namespace and database")?;
        let provider = SurrealDbProvider::from_existing_client(
            reopened.clone(),
            HostSurrealBackend::SurrealKv,
        )
        .await
        .context("construct reopened borrowed SurrealDB provider")?;

        let mut records = reopened
            .query(
                "SELECT * FROM type::record('agui_runs', $run_key); \
                 SELECT * FROM type::record('agui_contexts', $context_key);",
            )
            .bind(("run_key", rejected.storage_key()))
            .bind(("context_key", rejected.context_key()))
            .await
            .context("query rejected durable run and context after reopen")?
            .check()
            .context("check rejected durable run and context query")?;
        let rejected_runs: Vec<Value> = records
            .take(0)
            .context("decode rejected durable run query")?;
        let rejected_contexts: Vec<Value> = records
            .take(1)
            .context("decode rejected durable context query")?;
        anyhow::ensure!(
            rejected_runs.is_empty(),
            "rejected run survived transaction rollback: {rejected_runs:?}"
        );
        anyhow::ensure!(
            rejected_contexts.is_empty(),
            "rejected context slot survived transaction rollback: {rejected_contexts:?}"
        );
        anyhow::ensure!(
            provider
                .load_agui_run(
                    &rejected.owner,
                    &accepted.agent_id,
                    &accepted.thread_id,
                    &accepted.wire_run_id,
                )
                .await
                .context("load accepted selectors as rejected owner after reopen")?
                .is_none(),
            "owner-scoped lookup exposed the accepted owner's run after reopen"
        );
        anyhow::ensure!(
            provider
                .load_agui_run(
                    &accepted.owner,
                    &accepted.agent_id,
                    &accepted.thread_id,
                    &accepted.wire_run_id,
                )
                .await
                .context("load accepted run after reopen")?
                .as_ref()
                == Some(&accepted),
            "accepted run did not round-trip after reopen"
        );

        let close = provider.close_scope().await;
        anyhow::ensure!(
            close.failure.is_none() && close.scope_released,
            "reopened borrowed AG-UI provider did not release: {close:?}"
        );
        anyhow::ensure!(
            !close.engine_close_acknowledged,
            "reopened borrowed AG-UI provider must not close the host engine"
        );
        Ok(())
    }
    .await;
    let reopened_close = reopened
        .shutdown()
        .await
        .context("close reopened SurrealKV host");
    reopened_result?;
    reopened_close?;

    Ok(())
}

fn agui_run_binding(owner: &str) -> universal_agent_runtime::uar::persistence::agui_runs::AguiRunBinding {
    universal_agent_runtime::uar::persistence::agui_runs::AguiRunBinding {
        owner: owner.to_owned(),
        agent_id: "agent-atomicity".to_owned(),
        native_agent_id: None,
        thread_id: "thread-atomicity".to_owned(),
        wire_run_id: "run-atomicity".to_owned(),
        native_run_id: format!("native-{owner}"),
        session_id: format!("session-{owner}"),
        client_message_id: Some("message-atomicity".to_owned()),
        action_interaction_id: Some("interaction-atomicity".to_owned()),
        workspace_id: None,
        input_digest: "input-digest-atomicity".to_owned(),
        ordinal: 1,
        revision: 1,
        start_cursor: 0,
        previous_wire_run_id: None,
        resumed_tool_ids: Vec::new(),
        terminal: None,
    }
}
