//! Native durable supervisor transactions. Exclusion and challenge consumption
//! share one transaction; timeout/retry never turns a pending row into proof.
use super::SurrealDbProvider;
use crate::uar::runtime::supervisor::{RecoveryTarget, SupervisorChallenge,
    SupervisorError, SupervisorRegistration, VerifiedSupervisorExclusion};

fn supported(store: &SurrealDbProvider) -> anyhow::Result<()> {
    if !store.durable_instances { return Err(SupervisorError::Unsupported.into()); }
    Ok(())
}
async fn read<T: serde::de::DeserializeOwned>(store: &SurrealDbProvider, table: &str, id: &str) -> anyhow::Result<Option<T>> {
    supported(store)?;
    let mut response = store.db.query("SELECT VALUE data FROM type::record($table,$id)")
        .bind(("table", table.to_owned())).bind(("id", id.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    if rows.len() > 1 { return Err(SupervisorError::Invalid.into()); }
    rows.first().map(|data| Ok(serde_json::from_str(data)?)).transpose()
}
pub(super) async fn load_domain(store: &SurrealDbProvider, epoch: &str) -> anyhow::Result<Option<SupervisorRegistration>> {
    let registration: Option<SupervisorRegistration> = read(store,"supervisor_domains",epoch).await?;
    if registration.as_ref().is_some_and(|row| row.runtime_epoch != epoch) { return Err(SupervisorError::Invalid.into()); }
    Ok(registration)
}
pub(super) async fn register(store: &SurrealDbProvider, registration: &SupervisorRegistration) -> anyhow::Result<()> {
    supported(store)?;
    let data = serde_json::to_string(registration)?;
    store.db.query("BEGIN TRANSACTION;
        LET $epoch_row = (SELECT * FROM type::record('checkpoint_runtime_epochs',$epoch))[0];
        IF $epoch_row != NONE AND $epoch_row.state != 'active' { THROW 'supervisor_conflict'; };
        IF $epoch_row = NONE { CREATE type::record('checkpoint_runtime_epochs',$epoch)
            SET state='active',epoch=$epoch,checkpoint_fence=0,claim_fence=0; };
        LET $domain = (SELECT * FROM type::record('supervisor_domains',$epoch))[0];
        IF $domain = NONE { CREATE type::record('supervisor_domains',$epoch) SET data=$data; }
        ELSE IF $domain.data != $data { THROW 'supervisor_conflict'; };
        COMMIT TRANSACTION;")
        .bind(("epoch", registration.runtime_epoch.clone())).bind(("data",data)).await?.check()
        .map_err(|_| SupervisorError::Conflict)?;
    Ok(())
}
pub(super) async fn create_challenge(store: &SurrealDbProvider, challenge: &SupervisorChallenge) -> anyhow::Result<()> {
    supported(store)?;
    if uuid::Uuid::parse_str(&challenge.id).is_err() || challenge.generation != 1 { return Err(SupervisorError::Invalid.into()); }
    // Bind against the immutable registration in the same transaction. An
    // operator cannot turn a copied domain from a different store into proof.
    store.db.query("BEGIN TRANSACTION;
        LET $domain = (SELECT * FROM type::record('supervisor_domains',$epoch))[0];
        IF $domain = NONE OR $domain.data != $registration { THROW 'supervisor_conflict'; };
        CREATE type::record('supervisor_challenges',$id) SET data=$data,state='pending',generation=1;
        COMMIT TRANSACTION;")
        .bind(("epoch",challenge.registration.runtime_epoch.clone()))
        .bind(("registration",serde_json::to_string(&challenge.registration)?))
        .bind(("id",challenge.id.clone())).bind(("data",serde_json::to_string(challenge)?)).await?.check()?;
    Ok(())
}
pub(super) async fn load_challenge(store: &SurrealDbProvider, id: &str) -> anyhow::Result<Option<SupervisorChallenge>> {
    supported(store)?;
    let mut response = store.db.query("SELECT VALUE data FROM type::record('supervisor_challenges',$id) WHERE state='pending' AND generation=1")
        .bind(("id",id.to_owned())).await?.check()?;
    let rows: Vec<String> = response.take(0)?;
    let value: Option<SupervisorChallenge> = rows.first().map(|data| serde_json::from_str(data)).transpose()?;
    if value.as_ref().is_some_and(|row| row.id != id || row.generation != 1) { return Err(SupervisorError::Invalid.into()); }
    Ok(value)
}
pub(super) async fn checkpoint_exclusion(store: &SurrealDbProvider, proof: &VerifiedSupervisorExclusion) -> anyhow::Result<()> {
    supported(store)?;
    let challenge = proof.challenge();
    let runtime_epoch = &challenge.registration.runtime_epoch;
    match &challenge.target {
        RecoveryTarget::CheckpointEpoch { runtime_epoch: target } if target == runtime_epoch => {},
        RecoveryTarget::TeamOwner { fence } if fence.incarnation_id == challenge.registration.team_incarnation => {},
        _ => return Err(SupervisorError::Invalid.into()),
    }
    store.db.query("BEGIN TRANSACTION;
        LET $challenge = (UPDATE type::record('supervisor_challenges',$id)
            SET state='consumed',generation=2,consumed_at=time::now()
            WHERE state='pending' AND generation=1 AND data=$challenge_data RETURN AFTER);
        IF array::len($challenge) != 1 { THROW 'supervisor_conflict'; };
        LET $epoch = (SELECT * FROM type::record('checkpoint_runtime_epochs',$epoch_id))[0];
        IF $epoch = NONE OR $epoch.state NOT IN ['active','supervisor_excluded'] { THROW 'supervisor_conflict'; };
        UPDATE type::record('checkpoint_runtime_epochs',$epoch_id)
            SET state='supervisor_excluded',exclusion_id=$id,excluded_at=time::now();
        CREATE type::record('supervisor_exclusions',$id)
            SET challenge_data=$challenge_data,observation_data=$observation,committed_at=time::now();
        COMMIT TRANSACTION;")
        .bind(("id",challenge.id.clone())).bind(("challenge_data",serde_json::to_string(challenge)?))
        .bind(("epoch_id",runtime_epoch.clone())).bind(("observation",proof.observation_json().to_owned()))
        .await?.check()?;
    Ok(())
}
