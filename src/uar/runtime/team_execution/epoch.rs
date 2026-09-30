use crate::uar::{
    compiler::collaboration::CollaborationCatalogService,
    domain::{collaboration::EffectiveBindingReceipt, team_execution::TeamExecutionAttempt},
};

/// Recompute current member resources as well as its durable task/membership fence.
pub(crate) async fn revalidate_member_binding(
    catalog: &CollaborationCatalogService,
    attempt: &TeamExecutionAttempt,
    receipt: &EffectiveBindingReceipt,
) -> anyhow::Result<()> {
    let current = catalog.resolve_team_member_run(attempt).await?;
    let stable = |receipt: &EffectiveBindingReceipt| -> anyhow::Result<serde_json::Value> {
        let mut value = serde_json::to_value(receipt)?;
        if let Some(object) = value.as_object_mut() {
            object.remove("createdAt");
            object.remove("contentDigest");
        }
        Ok(value)
    };
    anyhow::ensure!(
        stable(receipt)? == stable(&current.effective_binding_receipt)?,
        "Member executable resources changed after admission"
    );
    Ok(())
}
