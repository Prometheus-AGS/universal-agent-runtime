use super::SurrealDbProvider;
use crate::uar::persistence::a2ui_actions::A2uiActionReceipt;

pub(super) async fn create(
    store: &SurrealDbProvider,
    row: &A2uiActionReceipt,
) -> anyhow::Result<bool> {
    anyhow::ensure!(store.durable_instances, "durable_a2ui_actions_unsupported");
    row.validate()?;
    let response = store.db.query("BEGIN TRANSACTION;
        IF (SELECT * FROM type::record('a2ui_action_receipts',$key))[0] != NONE { THROW 'a2ui_interaction_exists'; };
        CREATE type::record('a2ui_action_receipts',$key) CONTENT $payload;
        COMMIT TRANSACTION;")
        .bind(("key", row.storage_key()))
        .bind(("payload", serde_json::json!({"owner_key":row.owner,"agent_id":row.agent_id,"thread_id":row.thread_id,"data":serde_json::to_string(row)?}))).await?;
    match response.check() {
        Ok(_) => Ok(true),
        Err(error) if error.to_string().contains("a2ui_interaction_exists") => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) async fn load(
    store: &SurrealDbProvider,
    owner: &str,
    agent: &str,
    thread: &str,
    interaction: &str,
) -> anyhow::Result<Option<A2uiActionReceipt>> {
    anyhow::ensure!(store.durable_instances, "durable_a2ui_actions_unsupported");
    let mut response = store.db.query("SELECT VALUE data FROM type::record('a2ui_action_receipts',$key) WHERE owner_key=$owner AND agent_id=$agent AND thread_id=$thread")
        .bind(("key", A2uiActionReceipt::key(owner, agent, thread, interaction)))
        .bind(("owner", owner.to_owned())).bind(("agent", agent.to_owned())).bind(("thread", thread.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    rows.first()
        .map(|data| {
            let row: A2uiActionReceipt = serde_json::from_str(data)?;
            row.validate()?;
            anyhow::ensure!(
                row.owner == owner
                    && row.agent_id == agent
                    && row.thread_id == thread
                    && row.interaction_id == interaction,
                "a2ui_interaction_identity_conflict"
            );
            Ok(row)
        })
        .transpose()
}
