use super::PostgresProvider;
use crate::uar::persistence::approval_decisions::{ApprovalRecord, ApprovalState};
use sqlx::Row;

pub(super) async fn create(store: &PostgresProvider, record: &ApprovalRecord) -> anyhow::Result<()> {
    anyhow::ensure!(record.state == ApprovalState::Pending && record.decision.is_none(), "New approval must be pending");
    sqlx::query("INSERT INTO approval_records (owner_key, issuer_id, challenge_id, root_run_id, state, data) VALUES ($1,$2,$3,$4,'pending',$5)")
        .bind(&record.owner_key).bind(&record.issuer_id).bind(&record.challenge_id)
        .bind(&record.root_run_id).bind(serde_json::to_value(record)?)
        .execute(&store.pool).await?;
    Ok(())
}

pub(super) async fn transition(store: &PostgresProvider, before: &ApprovalRecord, after: &ApprovalRecord) -> anyhow::Result<bool> {
    before.validate_transition(after)?;
    if matches!(after.state, ApprovalState::Approved | ApprovalState::Denied) {
        let before_json = serde_json::to_value(before)?;
        let after_json = serde_json::to_value(after)?;
        let mut transaction = store.pool.begin().await?;
        // The deadline is evaluated after waiting for the row lock. NOW()
        // would retain transaction-start time across that wait.
        let locked = sqlx::query("SELECT data FROM approval_records WHERE owner_key = $1 AND issuer_id = $2
            AND challenge_id = $3 AND state = 'pending' AND data = $4 FOR UPDATE")
            .bind(&before.owner_key).bind(&before.issuer_id).bind(&before.challenge_id).bind(&before_json)
            .fetch_optional(&mut *transaction).await?;
        if locked.is_none() { transaction.rollback().await?; return Ok(false); }
        let row = sqlx::query("UPDATE approval_records SET state = 'resolved',
            data = CASE WHEN $6 > clock_timestamp() THEN $4 ELSE $7 END
            WHERE owner_key = $1 AND issuer_id = $2 AND challenge_id = $3 AND state = 'pending' AND data = $5 RETURNING data")
            .bind(&before.owner_key).bind(&before.issuer_id).bind(&before.challenge_id)
            .bind(&after_json).bind(&before_json).bind(before.expires_at)
            .bind(serde_json::to_value(before.expire())?).fetch_optional(&mut *transaction).await?;
        let applied = row.map(|row| row.try_get::<serde_json::Value, _>("data"))
            .transpose()?.is_some_and(|data| data == after_json);
        transaction.commit().await?;
        return Ok(applied);
    }
    let result = sqlx::query("UPDATE approval_records SET state = 'resolved', data = $4 WHERE owner_key = $1 AND issuer_id = $2 AND challenge_id = $3 AND state = 'pending' AND data = $5")
        .bind(&before.owner_key).bind(&before.issuer_id).bind(&before.challenge_id)
        .bind(serde_json::to_value(after)?).bind(serde_json::to_value(before)?)
        .execute(&store.pool).await?;
    Ok(result.rows_affected() == 1)
}

pub(super) async fn list(store: &PostgresProvider, owner: &str, run: &str) -> anyhow::Result<Vec<ApprovalRecord>> {
    let rows = sqlx::query("SELECT data FROM approval_records WHERE owner_key = $1 AND root_run_id = $2 ORDER BY challenge_id")
        .bind(owner).bind(run).fetch_all(&store.pool).await?;
    rows.into_iter().map(|row| Ok(serde_json::from_value(row.try_get("data")?)?)).collect()
}
