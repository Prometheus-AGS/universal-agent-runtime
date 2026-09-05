use crate::session::Session;
use crate::uar::a2ui::presentations::{Presentation, PresentationDraft};
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
use crate::uar::domain::prompt_caching::UserPromptCachingSettings;
use crate::uar::domain::skills::{Skill, SkillMatch};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::persistence::agent_threads::{self, AgentThreadStoreError, PersistedAgentThread};
use crate::uar::persistence::presentations::{self, PresentationStoreError};
use crate::uar::runtime::thread::{AgentEdge, AgentThread};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use surrealdb::Surreal;
use surrealdb::engine::any::{self, Any};
use surrealdb::opt::auth::Root;

#[derive(Debug)]
pub struct SurrealDbProvider {
    db: Surreal<Any>,
}

impl SurrealDbProvider {
    /// Connect to SurrealDB.
    ///
    /// For server endpoints (`ws://`, `wss://`, `http://`, `https://`) the
    /// caller may supply optional root credentials.  When credentials are
    /// absent the defaults `root` / `root` are used, which matches a
    /// freshly-started SurrealDB server.  For embedded endpoints (SurrealKV,
    /// in-memory) authentication is not performed.
    pub async fn new(
        connection_string: &str,
        surreal_user: Option<&str>,
        surreal_pass: Option<&str>,
        surreal_ns: Option<&str>,
        surreal_db: Option<&str>,
    ) -> Result<Self> {
        let endpoint = normalize_endpoint(connection_string);
        tracing::info!("Connecting to SurrealDB: {}", endpoint);

        let db = any::connect(&endpoint).await?;

        // Server-mode endpoints require signin before namespace/db selection.
        if is_server_endpoint(&endpoint) {
            let username = surreal_user.unwrap_or("root").to_string();
            let password = surreal_pass.unwrap_or("root").to_string();
            db.signin(Root {
                username: username.clone(),
                password,
            })
            .await?;
            tracing::info!("SurrealDB server signin completed as '{}'", username);
        }

        let ns = surreal_ns.unwrap_or("uar");
        let database = surreal_db.unwrap_or("uar");
        db.use_ns(ns).use_db(database).await?;
        tracing::info!("SurrealDB using ns='{}' db='{}'", ns, database);

        db.query(include_str!(
            "../../../../migrations/surrealdb/agent_threads.surql"
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

        tracing::info!("SurrealDB connected successfully");

        Ok(Self { db })
    }

    pub fn client(&self) -> Surreal<Any> {
        self.db.clone()
    }
}

/// Treat SurrealDB's "table does not exist" as an empty result set.
///
/// SurrealDB is schemaless: a table only comes into being when its first
/// record is written. Reads against a table that has never been written to
/// therefore fail rather than returning nothing, which would surface to API
/// clients as a 500 on any first-run or fresh-deploy instance. An absent
/// table and an empty table are indistinguishable to a caller listing
/// records, so both map to the empty case.
///
/// Errors that are not a missing table are returned unchanged.
///
/// # Errors
///
/// Returns the original error whenever its message does not identify a
/// missing table.
///
/// # Examples
///
/// ```ignore
/// let rows: Vec<Value> = empty_when_table_missing(response.take(0))?;
/// ```
pub(crate) fn empty_when_table_missing<T, E: std::fmt::Display>(
    result: std::result::Result<Vec<T>, E>,
) -> Result<Vec<T>> {
    result.or_else(|e| {
        if e.to_string().contains("does not exist") {
            Ok(Vec::new())
        } else {
            Err(anyhow::anyhow!(e.to_string()))
        }
    })
}

/// Treat SurrealDB's "table does not exist" as an absent record.
///
/// The single-record counterpart to [`empty_when_table_missing`]; see that
/// function for why a missing table is not an error on a read path.
///
/// # Errors
///
/// Returns the original error whenever its message does not identify a
/// missing table.
///
/// # Examples
///
/// ```ignore
/// let row: Option<Value> = none_when_table_missing(db.select(("t", id)).await)?;
/// ```
pub(crate) fn none_when_table_missing<T, E: std::fmt::Display>(
    result: std::result::Result<Option<T>, E>,
) -> Result<Option<T>> {
    result.or_else(|e| {
        if e.to_string().contains("does not exist") {
            Ok(None)
        } else {
            Err(anyhow::anyhow!(e.to_string()))
        }
    })
}

/// Returns `true` for network-accessible SurrealDB endpoints that require
/// explicit authentication before selecting a namespace/database.
fn is_server_endpoint(endpoint: &str) -> bool {
    let l = endpoint.to_ascii_lowercase();
    l.starts_with("ws://")
        || l.starts_with("wss://")
        || l.starts_with("http://")
        || l.starts_with("https://")
}

fn normalize_endpoint(connection_string: &str) -> String {
    let trimmed = connection_string.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("rocksdb://") {
        trimmed.replacen("rocksdb://", "surrealkv://", 1)
    } else if trimmed.contains("://")
        || lower == "memory"
        || lower == "mem"
        || lower == "surrealkv"
        || lower == "rocksdb"
    {
        if lower == "memory" || lower == "mem" {
            "mem://".to_string()
        } else if lower == "surrealkv" {
            "surrealkv://".to_string()
        } else if lower == "rocksdb" {
            "surrealkv://./data/uar.db".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        format!("surrealkv://{trimmed}")
    }
}

pub(crate) fn to_db_value<T: Serialize>(value: &T) -> Result<serde_json::Value> {
    serde_json::to_value(value).context("failed to serialize value for SurrealDB")
}

fn from_db_value<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    serde_json::from_value(value).context("failed to deserialize value from SurrealDB")
}

fn from_db_opt<T: DeserializeOwned>(value: Option<serde_json::Value>) -> Result<Option<T>> {
    value.map(from_db_value).transpose()
}

fn from_db_vec<T: DeserializeOwned>(values: Vec<serde_json::Value>) -> Result<Vec<T>> {
    values.into_iter().map(from_db_value).collect()
}

fn agent_thread_payload(record: &PersistedAgentThread) -> Result<serde_json::Value> {
    let thread = &record.thread;
    Ok(serde_json::json!({
        "owner_id": thread.owner_id,
        "thread_id": thread.thread_id,
        "root_thread_id": thread.root_thread_id,
        "root_run_id": thread.root_run_id,
        "parent_thread_id": thread.parent_thread_id,
        "canonical_path": thread.canonical_path,
        "revision": record.revision,
        "spawn_fence": 0,
        "data": serde_json::to_string(record)?,
    }))
}

fn check_agent_thread_write(mut response: surrealdb::IndexedResults) -> Result<()> {
    let errors = response.take_errors();
    // A transaction can mark earlier statements as cancelled. Inspect every
    // error so that cancellation wrappers do not obscure our deliberate refusal.
    for error in errors.values() {
        let message = error.to_string();
        if message.contains("uar_agent_thread_conflict") {
            return Err(AgentThreadStoreError::Conflict.into());
        }
        if message.contains("uar_agent_thread_exists") {
            return Err(AgentThreadStoreError::AlreadyExists.into());
        }
        if message.contains("uar_agent_thread_missing") {
            return Err(AgentThreadStoreError::NotFound.into());
        }
    }
    if let Some((_, error)) = errors.into_iter().min_by_key(|(index, _)| *index) {
        return Err(error.into());
    }
    Ok(())
}

fn presentation_payload(record: &Presentation) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "owner_id": record.owner_id, "presentation_id": record.id,
        "revision": record.revision, "data": serde_json::to_string(record)?
    }))
}

fn check_presentation_write(mut response: surrealdb::IndexedResults) -> Result<()> {
    let errors = response.take_errors();
    for error in errors.values() {
        if error.to_string().contains("uar_presentation_conflict") {
            return Err(PresentationStoreError::Conflict.into());
        }
    }
    if let Some((_, error)) = errors.into_iter().min_by_key(|(index, _)| *index) {
        return Err(error.into());
    }
    Ok(())
}

fn tenant_record_payload<T: Serialize>(value: &T, logical_id: &str) -> Result<serde_json::Value> {
    let mut payload = to_db_value(value)?;
    let fields = payload
        .as_object_mut()
        .context("tenant-owned record must serialize as an object")?;
    fields.remove("id");
    fields.insert(
        "logical_id".to_string(),
        serde_json::Value::String(logical_id.to_string()),
    );
    Ok(payload)
}

fn restore_tenant_record_id(mut value: serde_json::Value) -> serde_json::Value {
    if let Some(fields) = value.as_object_mut()
        && let Some(logical_id) = fields.remove("logical_id")
    {
        fields.insert("id".to_string(), logical_id);
    }
    value
}

fn restore_tenant_record_ids(values: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
    values.into_iter().map(restore_tenant_record_id).collect()
}

impl SurrealDbProvider {
    async fn save_tenant_record<T: Serialize>(
        &self,
        table: &str,
        owner_id: &str,
        logical_id: &str,
        value: &T,
    ) -> Result<()> {
        let storage_key = crate::uar::persistence::tenant_storage_key(owner_id, logical_id);
        let payload = tenant_record_payload(value, logical_id)?;
        let _: Option<surrealdb::types::Value> = self
            .db
            .upsert((table, storage_key))
            .content(payload)
            .await?;

        if let Some(legacy) = self.fetch_one(table, logical_id).await?
            && legacy.get("owner_id").and_then(serde_json::Value::as_str) == Some(owner_id)
        {
            let _: Option<surrealdb::types::Value> = self.db.delete((table, logical_id)).await?;
        }
        Ok(())
    }

    async fn fetch_tenant_record(
        &self,
        table: &str,
        owner_id: &str,
        logical_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let storage_key = crate::uar::persistence::tenant_storage_key(owner_id, logical_id);
        if let Some(record) = self.fetch_one(table, &storage_key).await? {
            return Ok(Some(restore_tenant_record_id(record)));
        }
        let legacy = self.fetch_one(table, logical_id).await?;
        Ok(legacy
            .filter(|record| {
                record.get("owner_id").and_then(serde_json::Value::as_str) == Some(owner_id)
            })
            .map(restore_tenant_record_id))
    }

    async fn delete_tenant_record(
        &self,
        table: &str,
        owner_id: &str,
        logical_id: &str,
    ) -> Result<()> {
        let storage_key = crate::uar::persistence::tenant_storage_key(owner_id, logical_id);
        let _: Option<surrealdb::types::Value> = self.db.delete((table, storage_key)).await?;
        if let Some(legacy) = self.fetch_one(table, logical_id).await?
            && legacy.get("owner_id").and_then(serde_json::Value::as_str) == Some(owner_id)
        {
            let _: Option<surrealdb::types::Value> = self.db.delete((table, logical_id)).await?;
        }
        Ok(())
    }

    async fn fetch_all(&self, table: &str) -> Result<Vec<serde_json::Value>> {
        let mut resp = self.db.query(format!("SELECT * FROM {}", table)).await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        rows.into_iter().map(surreal_to_json).collect()
    }

    async fn fetch_one(&self, table: &str, id: &str) -> Result<Option<serde_json::Value>> {
        let sql = format!("SELECT * FROM type::record('{}', $id)", table);
        let mut resp = self.db.query(sql).bind(("id", id.to_string())).await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        rows.into_iter().next().map(surreal_to_json).transpose()
    }
}

#[async_trait]
impl PersistenceLayer for SurrealDbProvider {
    async fn create_presentation(
        &self,
        owner_id: &str,
        draft: &PresentationDraft,
    ) -> Result<Presentation> {
        let record = presentations::new_record(owner_id, draft)?;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, &record.id);
        self.db
            .query("CREATE type::record('presentations', $key) CONTENT $payload")
            .bind(("key", key))
            .bind(("payload", presentation_payload(&record)?))
            .await?
            .check()?;
        Ok(record)
    }

    async fn get_presentation(&self, owner_id: &str, id: &str) -> Result<Option<Presentation>> {
        let mut response = self.db.query("SELECT VALUE data FROM presentations WHERE owner_id = $owner AND presentation_id = $id")
            .bind(("owner", owner_id.to_string())).bind(("id", id.to_string())).await?.check()?;
        let rows: Vec<String> = response.take(0)?;
        rows.into_iter()
            .next()
            .map(|data| Ok(serde_json::from_str(&data)?))
            .transpose()
    }

    async fn list_presentations(&self, owner_id: &str) -> Result<Vec<Presentation>> {
        let mut response = self
            .db
            .query("SELECT * FROM presentations WHERE owner_id = $owner ORDER BY presentation_id")
            .bind(("owner", owner_id.to_string()))
            .await?
            .check()?;
        let rows: Vec<serde_json::Value> = response.take(0)?;
        rows.into_iter()
            .map(|row| {
                let data = row["data"]
                    .as_str()
                    .context("Presentation envelope has no data")?;
                Ok(serde_json::from_str(data)?)
            })
            .collect()
    }

    async fn update_presentation(
        &self,
        owner_id: &str,
        id: &str,
        expected_revision: u64,
        draft: &PresentationDraft,
    ) -> Result<Presentation> {
        let current = self
            .get_presentation(owner_id, id)
            .await?
            .ok_or(PresentationStoreError::NotFound)?;
        let next = presentations::next_record(&current, expected_revision, draft)?;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, id);
        let response = self
            .db
            .query(
                "BEGIN TRANSACTION;
             LET $old = (SELECT * FROM type::record('presentations', $key))[0];
             IF $old = NONE OR $old.revision != $expected { THROW 'uar_presentation_conflict'; };
             UPDATE type::record('presentations', $key) CONTENT $payload;
             COMMIT TRANSACTION;",
            )
            .bind(("key", key))
            .bind(("expected", current.revision as i64))
            .bind(("payload", presentation_payload(&next)?))
            .await?;
        check_presentation_write(response)?;
        Ok(next)
    }

    async fn delete_presentation(
        &self,
        owner_id: &str,
        id: &str,
        expected_revision: u64,
    ) -> Result<()> {
        let current = self
            .get_presentation(owner_id, id)
            .await?
            .ok_or(PresentationStoreError::NotFound)?;
        if current.revision != expected_revision {
            return Err(PresentationStoreError::Conflict.into());
        }
        let key = crate::uar::persistence::tenant_storage_key(owner_id, id);
        let response = self
            .db
            .query(
                "BEGIN TRANSACTION;
             LET $old = (SELECT * FROM type::record('presentations', $key))[0];
             IF $old = NONE OR $old.revision != $expected { THROW 'uar_presentation_conflict'; };
             DELETE type::record('presentations', $key);
             COMMIT TRANSACTION;",
            )
            .bind(("key", key))
            .bind(("expected", current.revision as i64))
            .await?;
        check_presentation_write(response)
    }

    async fn create_agent_root(
        &self,
        owner_id: &str,
        thread: &AgentThread,
    ) -> Result<PersistedAgentThread> {
        let record = agent_threads::new_root(owner_id, thread)?;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, &thread.thread_id);
        let response = self.db.query(
            "BEGIN TRANSACTION;
             LET $old = (SELECT * FROM type::record('agent_threads', $key))[0];
             LET $path = (SELECT VALUE thread_id FROM agent_threads
                 WHERE owner_id = $owner AND root_run_id = $root_run AND canonical_path = $path_name)[0];
             IF $old != NONE OR $path != NONE { THROW 'uar_agent_thread_exists'; };
             CREATE type::record('agent_threads', $key) CONTENT $payload;
             COMMIT TRANSACTION;",
        ).bind(("key", key)).bind(("owner", owner_id.to_string()))
            .bind(("root_run", thread.root_run_id.clone())).bind(("path_name", thread.canonical_path.clone()))
            .bind(("payload", agent_thread_payload(&record)?)).await?;
        check_agent_thread_write(response)?;
        Ok(record)
    }

    async fn create_agent_child(
        &self,
        owner_id: &str,
        thread: &AgentThread,
        edge: &AgentEdge,
    ) -> Result<PersistedAgentThread> {
        let parent_id = thread
            .parent_thread_id
            .as_deref()
            .ok_or(AgentThreadStoreError::InvalidTransition)?;
        let root = self
            .load_agent_thread(owner_id, &thread.root_thread_id)
            .await?
            .ok_or(AgentThreadStoreError::NotFound)?;
        let parent = if parent_id == thread.root_thread_id {
            root.clone()
        } else {
            self.load_agent_thread(owner_id, parent_id)
                .await?
                .ok_or(AgentThreadStoreError::NotFound)?
        };
        let record = agent_threads::new_child(owner_id, thread, edge, &parent, &root)?;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, &thread.thread_id);
        let parent_key = crate::uar::persistence::tenant_storage_key(owner_id, parent_id);
        let root_key =
            crate::uar::persistence::tenant_storage_key(owner_id, &thread.root_thread_id);
        let edge_payload = serde_json::json!({
            "owner_id": edge.owner_id, "child_thread_id": edge.child_thread_id,
            "parent_thread_id": edge.parent_thread_id, "root_thread_id": edge.root_thread_id,
            "root_run_id": edge.root_run_id, "canonical_path": edge.canonical_path,
            "data": serde_json::to_string(edge)?,
        });
        let response = self.db.query(
            "BEGIN TRANSACTION;
             LET $parent = (SELECT * FROM type::record('agent_threads', $parent_key))[0];
             LET $root = (SELECT * FROM type::record('agent_threads', $root_key))[0];
             IF $parent = NONE OR $root = NONE { THROW 'uar_agent_thread_missing'; };
             IF $parent.data != $parent_data OR $root.data != $root_data {
                 THROW 'uar_agent_thread_conflict';
             };
             LET $old = (SELECT * FROM type::record('agent_threads', $key))[0];
             LET $old_edge = (SELECT * FROM type::record('agent_edges', $key))[0];
             LET $path = (SELECT VALUE thread_id FROM agent_threads
                 WHERE owner_id = $owner AND root_run_id = $root_run AND canonical_path = $path_name)[0];
             IF $old != NONE OR $old_edge != NONE OR $path != NONE { THROW 'uar_agent_thread_exists'; };
             -- Write decision records too: a concurrent parent/root state
             -- change must conflict, not commit a spawn from an obsolete read.
             UPDATE type::record('agent_threads', $root_key) SET spawn_fence += 1;
             IF $parent_key != $root_key {
                 UPDATE type::record('agent_threads', $parent_key) SET spawn_fence += 1;
             };
             CREATE type::record('agent_threads', $key) CONTENT $payload;
             CREATE type::record('agent_edges', $key) CONTENT $edge_payload;
             COMMIT TRANSACTION;",
        ).bind(("key", key)).bind(("parent_key", parent_key)).bind(("root_key", root_key))
            .bind(("parent_data", serde_json::to_string(&parent)?)).bind(("root_data", serde_json::to_string(&root)?))
            .bind(("owner", owner_id.to_string())).bind(("root_run", thread.root_run_id.clone()))
            .bind(("path_name", thread.canonical_path.clone())).bind(("payload", agent_thread_payload(&record)?))
            .bind(("edge_payload", edge_payload)).await?;
        check_agent_thread_write(response)?;
        Ok(record)
    }

    async fn load_agent_thread(
        &self,
        owner_id: &str,
        thread_id: &str,
    ) -> Result<Option<PersistedAgentThread>> {
        let mut response = self.db.query(
            "SELECT VALUE data FROM agent_threads WHERE owner_id = $owner AND thread_id = $thread",
        ).bind(("owner", owner_id.to_string())).bind(("thread", thread_id.to_string())).await?.check()?;
        let rows: Vec<String> = response.take(0)?;
        if rows.len() > 1 {
            return Err(AgentThreadStoreError::AlreadyExists.into());
        }
        rows.into_iter()
            .next()
            .map(|data| {
                let record: PersistedAgentThread = serde_json::from_str(&data)?;
                agent_threads::validate_lookup(&record, owner_id, thread_id)?;
                Ok(record)
            })
            .transpose()
    }

    async fn update_agent_thread(
        &self,
        owner_id: &str,
        expected_revision: u64,
        thread: &AgentThread,
    ) -> Result<PersistedAgentThread> {
        let current = self
            .load_agent_thread(owner_id, &thread.thread_id)
            .await?
            .ok_or(AgentThreadStoreError::NotFound)?;
        let next = agent_threads::next_record(owner_id, &current, expected_revision, thread)?;
        let key = crate::uar::persistence::tenant_storage_key(owner_id, &thread.thread_id);
        let response = self.db.query(
            "BEGIN TRANSACTION;
             LET $old = (SELECT * FROM type::record('agent_threads', $key))[0];
             IF $old = NONE { THROW 'uar_agent_thread_missing'; };
             IF $old.revision != $expected OR $old.data != $current_data { THROW 'uar_agent_thread_conflict'; };
             UPDATE type::record('agent_threads', $key) CONTENT $payload;
             COMMIT TRANSACTION;",
        ).bind(("key", key)).bind(("expected", expected_revision as i64))
            .bind(("current_data", serde_json::to_string(&current)?))
            .bind(("payload", agent_thread_payload(&next)?)).await?;
        check_agent_thread_write(response)?;
        Ok(next)
    }

    async fn list_agent_threads(
        &self,
        owner_id: &str,
        root_run_id: &str,
    ) -> Result<Vec<PersistedAgentThread>> {
        let mut response = self.db.query(
            "SELECT VALUE data FROM agent_threads WHERE owner_id = $owner AND root_run_id = $root_run",
        ).bind(("owner", owner_id.to_string())).bind(("root_run", root_run_id.to_string())).await?.check()?;
        let rows: Vec<String> = response.take(0)?;
        let records = rows
            .into_iter()
            .map(|data| serde_json::from_str(&data))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(agent_threads::ordered_threads(
            records,
            owner_id,
            root_run_id,
        )?)
    }

    async fn list_agent_edges(&self, owner_id: &str, root_run_id: &str) -> Result<Vec<AgentEdge>> {
        let mut response = self.db.query(
            "SELECT VALUE data FROM agent_edges WHERE owner_id = $owner AND root_run_id = $root_run",
        ).bind(("owner", owner_id.to_string())).bind(("root_run", root_run_id.to_string())).await?.check()?;
        let rows: Vec<String> = response.take(0)?;
        let edges = rows
            .into_iter()
            .map(|data| serde_json::from_str(&data))
            .collect::<Result<Vec<_>, _>>()?;
        let threads = self.list_agent_threads(owner_id, root_run_id).await?;
        Ok(agent_threads::ordered_edges(
            edges,
            &threads,
            owner_id,
            root_run_id,
        )?)
    }

    // Session Management
    async fn save_session(&self, session: &Session) -> Result<()> {
        self.save_tenant_record("sessions", session.owner_id(), session.id(), session)
            .await
    }

    async fn load_session(&self, owner_id: &str, id: &str) -> Result<Option<Session>> {
        if let Some(session) = self.fetch_tenant_record("sessions", owner_id, id).await? {
            return from_db_opt(Some(session));
        }
        if owner_id != crate::session::ANONYMOUS_SESSION_OWNER {
            return Ok(None);
        }
        let Some(legacy) = self.fetch_one("sessions", id).await? else {
            return Ok(None);
        };
        let session: Session = from_db_value(legacy)?;
        self.save_session(&session).await?;
        let _: Option<surrealdb::types::Value> = self.db.delete(("sessions", id)).await?;
        Ok(Some(session))
    }

    async fn save_conversation_policy(
        &self,
        record: &crate::uar::domain::policy::ConversationPolicyRecord,
    ) -> Result<()> {
        self.save_tenant_record(
            "conversation_policies",
            &record.owner_id,
            &record.conversation_id,
            record,
        )
        .await
    }

    async fn load_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<crate::uar::domain::policy::ConversationPolicyRecord>> {
        if let Some(record) = self
            .fetch_tenant_record("conversation_policies", owner_id, conversation_id)
            .await?
        {
            return from_db_opt(Some(record));
        }
        if owner_id != crate::session::ANONYMOUS_SESSION_OWNER {
            return Ok(None);
        }
        let Some(legacy) = self
            .fetch_one("conversation_policies", conversation_id)
            .await?
        else {
            return Ok(None);
        };
        let record: crate::uar::domain::policy::ConversationPolicyRecord = from_db_value(legacy)?;
        self.save_conversation_policy(&record).await?;
        let _: Option<surrealdb::types::Value> = self
            .db
            .delete(("conversation_policies", conversation_id))
            .await?;
        Ok(Some(record))
    }

    async fn delete_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<()> {
        self.delete_tenant_record("conversation_policies", owner_id, conversation_id)
            .await
    }

    async fn save_principal_conversation_policy(
        &self,
        record: &crate::uar::domain::policy::ConversationPolicyRecord,
        expected: Option<&crate::uar::domain::policy::RunPolicy>,
    ) -> Result<bool> {
        let key =
            crate::uar::persistence::tenant_storage_key(&record.owner_id, &record.conversation_id);
        let payload = tenant_record_payload(record, &record.conversation_id)?;
        if let Some(expected) = expected {
            let mut response = self.db.query("UPDATE type::record('principal_conversation_policies', $key) CONTENT $payload WHERE policy = $expected RETURN AFTER")
                .bind(("key", key)).bind(("payload", payload))
                .bind(("expected", serde_json::to_value(expected)?)).await?;
            let rows: Vec<surrealdb::types::Value> = response.take(0)?;
            Ok(rows.len() == 1)
        } else {
            // CREATE never overwrites an interleaved first save. An uncertain
            // backend failure remains an error, not a fabricated conflict result.
            self.db
                .query(
                    "CREATE type::record('principal_conversation_policies', $key) CONTENT $payload",
                )
                .bind(("key", key))
                .bind(("payload", payload))
                .await?
                .check()?;
            Ok(true)
        }
    }

    async fn load_principal_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<crate::uar::domain::policy::ConversationPolicyRecord>> {
        let key = crate::uar::persistence::tenant_storage_key(owner_id, conversation_id);
        // This table has no legacy records. Do not use the legacy-migrating helpers.
        let mut response = self
            .db
            .query("SELECT * FROM type::record('principal_conversation_policies', $key)")
            .bind(("key", key))
            .await?;
        let rows: Vec<surrealdb::types::Value> = response.take(0)?;
        let record = rows.into_iter().next().map(surreal_to_json).transpose()?;
        from_db_opt(record.map(restore_tenant_record_id))
    }

    async fn save_user_prompt_caching_settings(
        &self,
        settings: &UserPromptCachingSettings,
    ) -> Result<()> {
        let payload = to_db_value(settings)?;
        let _: Option<surrealdb::types::Value> = self
            .db
            .upsert(("user_prompt_caching_settings", settings.user_id.clone()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn load_user_prompt_caching_settings(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserPromptCachingSettings>> {
        from_db_opt(
            self.fetch_one("user_prompt_caching_settings", principal_id)
                .await?,
        )
    }

    // Skill Management
    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()> {
        // We need to store embedding alongside skill.
        // Create a wrapper struct
        #[derive(Serialize, Deserialize)]
        struct SkillRecord {
            #[serde(flatten)]
            skill: Skill,
            embedding: Vec<f32>,
        }

        let record = SkillRecord {
            skill: skill.clone(),
            embedding: embedding.to_vec(),
        };
        let payload = to_db_value(&record)?;

        let _: Option<surrealdb::types::Value> = self
            .db
            .upsert(("skills", skill.skill_id.clone()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn search_skills(&self, query_vec: &[f32], limit: usize) -> Result<Vec<SkillMatch>> {
        // Fallback: Fetch all, compute cosine similarity in memory
        // Ideally use vector search plugin/feature if available.
        #[derive(Deserialize)]
        struct SkillRecord {
            #[serde(flatten)]
            skill: Skill,
            embedding: Vec<f32>,
        }

        let skills_raw = self.fetch_all("skills").await?;
        let skills: Vec<SkillRecord> = from_db_vec(skills_raw)?;

        let mut matches: Vec<SkillMatch> = skills
            .into_iter()
            .map(|s| {
                let score = cosine_similarity(&s.embedding, query_vec);
                SkillMatch {
                    skill: s.skill,
                    score,
                }
            })
            .collect();

        // Sort descending
        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(limit);

        Ok(matches)
    }

    async fn list_skills(&self) -> Result<Vec<Skill>> {
        #[derive(Deserialize)]
        struct SkillRecord {
            #[serde(flatten)]
            skill: Skill,
            #[allow(dead_code)]
            embedding: Vec<f32>,
        }

        let records_raw = self.fetch_all("skills").await?;
        let records: Vec<SkillRecord> = from_db_vec(records_raw)?;
        Ok(records.into_iter().map(|r| r.skill).collect())
    }

    async fn delete_skill(&self, id: &str) -> Result<()> {
        let _: Option<surrealdb::types::Value> = self.db.delete(("skills", id)).await?;
        Ok(())
    }

    // Knowledge Base Management
    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> {
        self.save_tenant_record("knowledge_bases", &kb.owner_id, &kb.id, kb)
            .await
    }

    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
        if self
            .get_knowledge_base(&chunk.owner_id, &chunk.kb_id)
            .await?
            .is_none()
        {
            anyhow::bail!("knowledge base is not accessible to chunk owner");
        }
        if let Some(document_id) = &chunk.document_id
            && self
                .get_document(&chunk.owner_id, document_id)
                .await?
                .is_none()
        {
            anyhow::bail!("knowledge document is not accessible to chunk owner");
        }
        self.save_tenant_record(
            "knowledge_chunks",
            &chunk.owner_id,
            &chunk.id.to_string(),
            chunk,
        )
        .await
    }

    async fn search_knowledge(
        &self,
        owner_id: &str,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        let mut response = self
            .db
            .query("SELECT * FROM knowledge_chunks WHERE owner_id = $owner_id")
            .bind(("owner_id", owner_id.to_string()))
            .await?;
        let chunks_raw: Vec<surrealdb::types::Value> = response.take(0)?;
        let chunks_raw = chunks_raw
            .into_iter()
            .map(surreal_to_json)
            .collect::<Result<Vec<_>>>()?;
        let chunks: Vec<KnowledgeChunk> = from_db_vec(restore_tenant_record_ids(chunks_raw))?;
        warn_zero_norm_chunks(&chunks);

        let mut matches: Vec<KnowledgeMatch> = chunks
            .into_iter()
            .map(|c| {
                let score = cosine_similarity(&c.embedding, query_vec);
                KnowledgeMatch { chunk: c, score }
            })
            .filter(|m| m.score >= min_score)
            .collect();

        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(limit);

        Ok(matches)
    }

    // Agent Persistence
    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()> {
        let payload = to_db_value(agent)?;
        let _: Option<surrealdb::types::Value> = self
            .db
            .upsert(("agents", agent.id.clone()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn load_agent(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        let agent = self.fetch_one("agents", id).await?;
        from_db_opt(agent)
    }

    async fn save_agent_if_unchanged(
        &self,
        expected: &crate::uar::domain::artifact::AgentArtifact,
        updated: &crate::uar::domain::artifact::AgentArtifact,
    ) -> Result<bool> {
        anyhow::ensure!(expected.id == updated.id, "Agent update identity mismatch");
        let Some(mut baseline) = self.fetch_one("agents", &expected.id).await? else {
            return Ok(false);
        };
        let current: crate::uar::domain::artifact::AgentArtifact = from_db_value(baseline.clone())?;
        if serde_json::to_value(current)? != serde_json::to_value(expected)? {
            return Ok(false);
        }
        baseline
            .as_object_mut()
            .context("Agent record must be an object")?
            .remove("id");
        let mut payload = to_db_value(updated)?;
        payload
            .as_object_mut()
            .context("Agent update must be an object")?
            .remove("id");
        let mut response = self.db.query("UPDATE type::record('agents', $id) CONTENT $payload WHERE object::remove($this, 'id') = $baseline RETURN AFTER")
            .bind(("id", expected.id.clone())).bind(("payload", payload)).bind(("baseline", baseline)).await?;
        let rows: Vec<surrealdb::types::Value> = response.take(0)?;
        Ok(rows.len() == 1)
    }

    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        let sql = "SELECT * FROM agents WHERE metadata.title = $name LIMIT 1";
        let mut response = self.db.query(sql).bind(("name", name.to_string())).await?;
        let rows: Vec<surrealdb::types::Value> = response.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let agent = rows.into_iter().next().map(surreal_to_json).transpose()?;
        from_db_opt(agent)
    }

    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>> {
        let agents_raw = self.fetch_all("agents").await?;
        from_db_vec(agents_raw)
    }

    async fn delete_agent(&self, id: &str) -> Result<()> {
        let _: Option<surrealdb::types::Value> = self.db.delete(("agents", id)).await?;
        Ok(())
    }

    // Memory System — delegates to MemoryService (backed by surreal-memory library)
    // These stubs satisfy the PersistenceLayer trait. Real memory operations should
    // use `AppState::memory_service` (a MemoryService wrapping SurrealStorage from
    // the surreal-memory library in its own embedded SurrealKV store).
    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()> {
        // The surreal-memory library owns memory persistence in its own SurrealDB instance.
        // This stub is a no-op; callers should use AppState::memory_service.
        tracing::debug!(
            "save_memory stub called — use AppState::memory_service for real persistence"
        );
        let _ = memory;
        Ok(())
    }

    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>> {
        // The surreal-memory library owns memory persistence in its own SurrealDB instance.
        // This stub returns empty; callers should use AppState::memory_service.
        tracing::debug!(
            "search_memory stub called — use AppState::memory_service for real queries"
        );
        let _ = (agent_id, query_vec, limit, min_score);
        Ok(vec![])
    }

    // =========================================================================
    // Knowledge Base Retrieval Methods
    // =========================================================================

    async fn get_knowledge_base(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeBase>> {
        from_db_opt(
            self.fetch_tenant_record("knowledge_bases", owner_id, id)
                .await?,
        )
    }

    async fn get_knowledge_base_by_name(
        &self,
        owner_id: &str,
        name: &str,
    ) -> Result<Option<KnowledgeBase>> {
        let sql =
            "SELECT * FROM knowledge_bases WHERE owner_id = $owner_id AND name = $name LIMIT 1";
        let mut response = self
            .db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("name", name.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = response.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let kb = rows.into_iter().next().map(surreal_to_json).transpose()?;
        from_db_opt(kb.map(restore_tenant_record_id))
    }

    async fn list_knowledge_bases(&self, owner_id: &str) -> Result<Vec<KnowledgeBase>> {
        let mut response = self
            .db
            .query("SELECT * FROM knowledge_bases WHERE owner_id = $owner_id")
            .bind(("owner_id", owner_id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = response.take(0)?;
        let rows = rows
            .into_iter()
            .map(surreal_to_json)
            .collect::<Result<Vec<_>>>()?;
        from_db_vec(restore_tenant_record_ids(rows))
    }

    async fn delete_knowledge_base(&self, owner_id: &str, id: &str) -> Result<()> {
        // Delete the KB - SurrealDB doesn't have FK CASCADE, so we delete related records first
        self.delete_tenant_record("knowledge_bases", owner_id, id)
            .await?;
        // Also delete related chunks and documents
        let sql = "DELETE FROM knowledge_chunks WHERE owner_id = $owner_id AND kb_id = $id";
        self.db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("id", id.to_string()))
            .await?;
        let sql = "DELETE FROM knowledge_documents WHERE owner_id = $owner_id AND kb_id = $id";
        self.db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("id", id.to_string()))
            .await?;
        Ok(())
    }

    // =========================================================================
    // Scoped Knowledge Search
    // =========================================================================

    async fn search_knowledge_scoped(
        &self,
        owner_id: &str,
        kb_ids: &[&str],
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        if kb_ids.is_empty() {
            return Ok(vec![]);
        }

        // Query with kb_id filter
        let kb_ids_vec: Vec<String> = kb_ids.iter().copied().map(str::to_string).collect();
        let sql = "SELECT * FROM knowledge_chunks WHERE owner_id = $owner_id AND kb_id IN $kb_ids";
        let mut res = self
            .db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("kb_ids", kb_ids_vec))
            .await?;
        let chunks_raw: Vec<surrealdb::types::Value> = res.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let chunks_json: Vec<serde_json::Value> = chunks_raw
            .into_iter()
            .map(surreal_to_json)
            .collect::<Result<_>>()?;
        let chunks: Vec<KnowledgeChunk> = from_db_vec(restore_tenant_record_ids(chunks_json))?;
        warn_zero_norm_chunks(&chunks);

        // In-memory cosine similarity
        let mut matches: Vec<KnowledgeMatch> = chunks
            .into_iter()
            .map(|c| {
                let score = cosine_similarity(&c.embedding, query_vec);
                KnowledgeMatch { chunk: c, score }
            })
            .filter(|m| m.score >= min_score)
            .collect();

        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches.truncate(limit);

        Ok(matches)
    }

    // =========================================================================
    // Document Tracking
    // =========================================================================

    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> {
        if self
            .get_knowledge_base(&doc.owner_id, &doc.kb_id)
            .await?
            .is_none()
        {
            anyhow::bail!("knowledge base is not accessible to document owner");
        }
        self.save_tenant_record("knowledge_documents", &doc.owner_id, &doc.id, doc)
            .await
    }

    async fn get_document(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeDocument>> {
        from_db_opt(
            self.fetch_tenant_record("knowledge_documents", owner_id, id)
                .await?,
        )
    }

    async fn list_documents(&self, owner_id: &str, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
        let sql = "SELECT * FROM knowledge_documents WHERE owner_id = $owner_id AND kb_id = $kb_id ORDER BY created_at";
        let mut res = self
            .db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("kb_id", kb_id.to_string()))
            .await?;
        let docs_raw: Vec<surrealdb::types::Value> = res.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let docs_json: Vec<serde_json::Value> = docs_raw
            .into_iter()
            .map(surreal_to_json)
            .collect::<Result<_>>()?;
        from_db_vec(restore_tenant_record_ids(docs_json))
    }

    async fn count_documents(&self, owner_id: &str, kb_id: &str) -> Result<usize> {
        let sql = "SELECT count() AS c FROM knowledge_documents WHERE owner_id = $owner_id AND kb_id = $kb_id GROUP ALL";
        let mut res = self
            .db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("kb_id", kb_id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = res.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let count = rows
            .into_iter()
            .next()
            .and_then(|v| {
                let j = surreal_to_json(v).ok()?;
                j.get("c").and_then(|c| c.as_u64()).map(|n| n as usize)
            })
            .unwrap_or(0);
        Ok(count)
    }

    async fn update_document_status(
        &self,
        owner_id: &str,
        doc_id: &str,
        status: &DocumentStatus,
    ) -> Result<()> {
        let sql = "UPDATE knowledge_documents SET status = $status, updated_at = time::now() WHERE owner_id = $owner_id AND (logical_id = $id OR id = type::record('knowledge_documents', $id))";
        self.db
            .query(sql)
            .bind(("id", doc_id.to_string()))
            .bind(("owner_id", owner_id.to_string()))
            .bind(("status", serde_json::to_value(status)?))
            .await?
            .check()?;
        Ok(())
    }

    async fn delete_document(&self, owner_id: &str, doc_id: &str) -> Result<()> {
        // Delete associated chunks first
        let sql =
            "DELETE FROM knowledge_chunks WHERE owner_id = $owner_id AND document_id = $doc_id";
        self.db
            .query(sql)
            .bind(("owner_id", owner_id.to_string()))
            .bind(("doc_id", doc_id.to_string()))
            .await?;

        self.delete_tenant_record("knowledge_documents", owner_id, doc_id)
            .await
    }

    // =========================================================================
    // Settings Management
    // =========================================================================

    async fn upsert_settings_type(
        &self,
        st: &crate::uar::settings::schema::SettingsType,
    ) -> Result<uuid::Uuid> {
        let payload: serde_json::Value = serde_json::json!({
            "name": st.name,
            "key": st.key,
            "schema": st.schema,
            "created_at": st.created_at,
            "updated_at": st.updated_at,
        });
        self.db
            .query("UPSERT type::record('settings_types', $key) CONTENT $data")
            .bind(("key", st.key.clone()))
            .bind(("data", payload))
            .await
            .with_context(|| format!("upserting settings_type '{}'", st.key))?;
        // SurrealDB uses string record IDs; return st.id as the FK identifier.
        Ok(st.id)
    }

    async fn list_settings_types(&self) -> Result<Vec<crate::uar::settings::schema::SettingsType>> {
        let mut resp = self
            .db
            .query("SELECT * FROM settings_types")
            .await
            .context("listing settings_types")?;
        let rows: Vec<surrealdb::types::Value> = resp
            .take::<Vec<surrealdb::types::Value>>(0)
            .or_else(|e| {
                if e.to_string().contains("does not exist") {
                    Ok(vec![])
                } else {
                    Err(anyhow::anyhow!(e))
                }
            })
            .context("taking settings_types results")?;
        rows.into_iter()
            .map(|v| surreal_value_to_settings_type(surreal_to_json(v)?))
            .collect()
    }

    async fn get_settings_type(
        &self,
        key: &str,
    ) -> Result<Option<crate::uar::settings::schema::SettingsType>> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::record('settings_types', $key)")
            .bind(("key", key.to_string()))
            .await
            .with_context(|| format!("get_settings_type({key})"))?;
        let rows: Vec<surrealdb::types::Value> = resp
            .take::<Vec<surrealdb::types::Value>>(0)
            .or_else(|e| {
                if e.to_string().contains("does not exist") {
                    Ok(vec![])
                } else {
                    Err(anyhow::anyhow!(e))
                }
            })
            .with_context(|| format!("taking settings_type({key})"))?;
        rows.into_iter()
            .next()
            .map(|v| surreal_value_to_settings_type(surreal_to_json(v)?))
            .transpose()
    }

    async fn upsert_setting(&self, setting: &crate::uar::settings::schema::Settings) -> Result<()> {
        // Validate data against the leaf property schema extracted from the parent type.
        // Setting keys are dotted: `server.port` → type key `server`, property key `port`.
        // We validate `data` against `type.schema.properties.port` (leaf schema), not
        // the root type schema (which is type:object) — leaf values are primitives.
        let type_key = setting.key.split('.').next().unwrap_or("unknown");
        let leaf_key = setting.key.splitn(2, '.').nth(1).unwrap_or(&setting.key);
        if let Some(st) = self.get_settings_type(type_key).await? {
            // Try to resolve the leaf-level property schema from the type's schema.
            // Setting keys are dotted: `server.port` → leaf key `port`.
            // We validate only if a matching property schema exists; otherwise skip.
            // This avoids false failures when types use a namespace-level schema without
            // a 1:1 property-per-setting mapping (e.g. knowledge_bases.named_keys).
            if let Some(leaf_schema) = st
                .schema
                .get("properties")
                .and_then(|props| props.get(leaf_key))
                .or_else(|| st.schema.get("additionalProperties"))
            {
                validate_against_schema(&setting.data, leaf_schema, &setting.key)?;
            }
        }

        let record_id = setting.key.replace('.', "_");
        let payload: serde_json::Value = serde_json::json!({
            "settings_type_key": type_key,
            "name": setting.name,
            "key": setting.key,
            "data": setting.data,
            "parent_id": setting.parent_id,
            "created_at": setting.created_at,
            "updated_at": setting.updated_at,
        });
        self.db
            .query("UPSERT type::record('settings', $rid) CONTENT $data")
            .bind(("rid", record_id))
            .bind(("data", payload))
            .await
            .with_context(|| format!("upserting setting '{}'", setting.key))?;
        Ok(())
    }

    async fn get_setting(
        &self,
        key: &str,
    ) -> Result<Option<crate::uar::settings::schema::Settings>> {
        let record_id = key.replace('.', "_");
        let mut resp = self
            .db
            .query("SELECT * FROM type::record('settings', $rid)")
            .bind(("rid", record_id))
            .await
            .with_context(|| format!("get_setting({key})"))?;
        let rows: Vec<surrealdb::types::Value> = resp
            .take::<Vec<surrealdb::types::Value>>(0)
            .or_else(|e| {
                if e.to_string().contains("does not exist") {
                    Ok(vec![])
                } else {
                    Err(anyhow::anyhow!(e))
                }
            })
            .with_context(|| format!("taking setting({key})"))?;
        rows.into_iter()
            .next()
            .map(|v| surreal_value_to_setting(surreal_to_json(v)?))
            .transpose()
    }

    async fn compare_and_swap_global_presentation_policy(
        &self,
        expected: &serde_json::Value,
        selection: &crate::uar::domain::policy::ResourceSelection,
    ) -> Result<bool> {
        let next = presentations::global_policy_with_presentations(expected, selection)?;
        let mut response = self.db.query("UPDATE type::record('settings', 'run_policy_global') SET data = $next, updated_at = $updated WHERE key = 'run_policy.global' AND data = $expected RETURN AFTER")
            .bind(("next", next)).bind(("expected", expected.clone()))
            .bind(("updated", chrono::Utc::now().to_rfc3339())).await?;
        let rows: Vec<surrealdb::types::Value> = response.take(0)?;
        Ok(rows.len() == 1)
    }

    async fn list_settings(
        &self,
        type_key: Option<&str>,
        parent_id: Option<uuid::Uuid>,
    ) -> Result<Vec<crate::uar::settings::schema::Settings>> {
        let mut resp = self
            .db
            .query("SELECT * FROM settings")
            .await
            .context("listing settings")?;
        let rows: Vec<surrealdb::types::Value> = resp
            .take::<Vec<surrealdb::types::Value>>(0)
            .or_else(|e| {
                if e.to_string().contains("does not exist") {
                    Ok(vec![])
                } else {
                    Err(anyhow::anyhow!(e))
                }
            })
            .context("taking settings results")?;

        let all: Vec<crate::uar::settings::schema::Settings> = rows
            .into_iter()
            .map(|v| surreal_value_to_setting(surreal_to_json(v)?))
            .collect::<Result<_>>()?;

        let filtered = all.into_iter().filter(|s| {
            let type_ok = type_key
                .map(|tk| s.key.starts_with(&format!("{tk}.")))
                .unwrap_or(true);
            let parent_ok = parent_id
                .map(|pid| s.parent_id == Some(pid))
                .unwrap_or(true);
            type_ok && parent_ok
        });

        Ok(filtered.collect())
    }

    async fn delete_setting(&self, key: &str) -> Result<()> {
        let record_id = key.replace('.', "_");
        self.db
            .query("DELETE type::record('settings', $rid)")
            .bind(("rid", record_id))
            .await
            .with_context(|| format!("delete_setting({key})"))?;
        Ok(())
    }

    // =========================================================================
    // Chat Attachment Storage
    // =========================================================================

    async fn insert_attachment(
        &self,
        meta: &crate::uar::persistence::AttachmentMeta,
    ) -> Result<()> {
        use serde::{Deserialize, Serialize};
        #[derive(Serialize, Deserialize)]
        struct AttachRecord {
            id: String,
            session_id: String,
            filename: String,
            content_type: String,
            file_path: String,
            file_size: i64,
            is_image: bool,
            text_content: Option<String>,
            created_at: chrono::DateTime<chrono::Utc>,
        }
        let record = AttachRecord {
            id: meta.id.clone(),
            session_id: meta.session_id.clone(),
            filename: meta.filename.clone(),
            content_type: meta.content_type.clone(),
            file_path: meta.file_path.clone(),
            file_size: meta.file_size,
            is_image: meta.is_image,
            text_content: meta.text_content.clone(),
            created_at: meta.created_at,
        };
        self.db
            .query("CREATE type::record('chat_attachments', $id) CONTENT $data")
            .bind(("id", meta.id.clone()))
            .bind((
                "data",
                serde_json::to_value(&record).context("serialize attach record")?,
            ))
            .await
            .context("insert_attachment")?;
        Ok(())
    }

    async fn get_attachment(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::persistence::AttachmentMeta>> {
        let id_owned = id.to_string();
        let mut res = self
            .db
            .query("SELECT * FROM type::record('chat_attachments', $id)")
            .bind(("id", id_owned))
            .await
            .context("get_attachment")?;
        let val: Option<serde_json::Value> = none_when_table_missing(res.take(0))?;
        match val {
            None => Ok(None),
            Some(v) => Ok(Some(surreal_json_to_attachment_meta(v)?)),
        }
    }

    async fn list_attachments_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<crate::uar::persistence::AttachmentMeta>> {
        let sid_owned = session_id.to_string();
        let mut res = self
            .db
            .query("SELECT * FROM chat_attachments WHERE session_id = $sid ORDER BY created_at ASC")
            .bind(("sid", sid_owned))
            .await
            .context("list_attachments_for_session")?;
        let vals: Vec<serde_json::Value> = empty_when_table_missing(res.take(0))?;
        vals.into_iter()
            .map(surreal_json_to_attachment_meta)
            .collect()
    }

    // =========================================================================
    // Checkpoint Persistence
    // =========================================================================

    async fn save_checkpoint(
        &self,
        checkpoint: &crate::uar::runtime::checkpoint::Checkpoint,
    ) -> Result<()> {
        let mut payload = to_db_value(checkpoint)?;
        // Store a redundant `_cp_id` field so list queries can recover the id
        // even when the SurrealDB driver serializes the RecordId opaquely.
        if let Some(obj) = payload.as_object_mut() {
            obj.insert(
                "_cp_id".to_string(),
                serde_json::Value::String(checkpoint.id.clone()),
            );
        }
        let _: Option<surrealdb::types::Value> = self
            .db
            .upsert(("checkpoints", checkpoint.id.clone()))
            .content(payload)
            .await
            .context("save_checkpoint")?;
        Ok(())
    }

    async fn load_checkpoint(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::runtime::checkpoint::Checkpoint>> {
        let raw = self.fetch_one("checkpoints", id).await?;
        match raw {
            None => Ok(None),
            Some(mut json) => {
                // RecordId round-trip may produce null for the id field depending on
                // the surrealdb driver version — restore it from the query parameter.
                if json.get("id").map_or(true, |v| v.is_null()) {
                    json["id"] = serde_json::Value::String(id.to_string());
                }
                let cp = serde_json::from_value(json).context("deserialise checkpoint")?;
                Ok(Some(cp))
            }
        }
    }

    async fn list_checkpoints(
        &self,
        run_id: &str,
    ) -> Result<Vec<crate::uar::runtime::checkpoint::Checkpoint>> {
        let run_id_owned = run_id.to_string();
        let mut res = self
            .db
            .query("SELECT * FROM checkpoints WHERE run_id = $run_id ORDER BY created_at ASC")
            .bind(("run_id", run_id_owned))
            .await
            .context("list_checkpoints")?;
        let raw: Vec<surrealdb::types::Value> = res.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        raw.into_iter()
            .map(|v| {
                let mut json = surreal_to_json(v)?;
                // Restore id from the redundant _cp_id field if RecordId extraction returned null.
                if json.get("id").map_or(true, |v| v.is_null()) {
                    if let Some(fallback) = json
                        .get("_cp_id")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                    {
                        json["id"] = serde_json::Value::String(fallback);
                    }
                }
                serde_json::from_value(json).map_err(anyhow::Error::from)
            })
            .collect()
    }

    // Cost Budget History (CH-07)
    async fn record_cost_entry(&self, scope: &str, scope_id: &str, cost_usd: f64) -> Result<()> {
        #[derive(Serialize)]
        struct CostLedgerRow {
            scope: String,
            scope_id: String,
            cost_usd: f64,
            recorded_at: chrono::DateTime<chrono::Utc>,
        }
        let row = CostLedgerRow {
            scope: scope.to_string(),
            scope_id: scope_id.to_string(),
            cost_usd,
            recorded_at: chrono::Utc::now(),
        };
        let payload = to_db_value(&row)?;
        let _: Option<surrealdb::types::Value> =
            self.db.create("cost_ledger").content(payload).await?;
        Ok(())
    }

    async fn list_cost_history(
        &self,
        scope: &str,
        scope_id: &str,
    ) -> Result<Vec<crate::uar::persistence::CostEntry>> {
        #[derive(Deserialize)]
        struct CostLedgerRow {
            scope: String,
            scope_id: String,
            cost_usd: f64,
            recorded_at: chrono::DateTime<chrono::Utc>,
        }
        let sql = "SELECT * FROM cost_ledger WHERE scope = $scope AND scope_id = $scope_id ORDER BY recorded_at";
        let mut res = self
            .db
            .query(sql)
            .bind(("scope", scope.to_string()))
            .bind(("scope_id", scope_id.to_string()))
            .await?;
        let raw: Vec<surrealdb::types::Value> = res.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let json: Vec<serde_json::Value> = raw
            .into_iter()
            .map(surreal_to_json)
            .collect::<Result<_>>()?;
        let rows: Vec<CostLedgerRow> = from_db_vec(json)?;
        Ok(rows
            .into_iter()
            .map(|r| crate::uar::persistence::CostEntry {
                scope: r.scope,
                scope_id: r.scope_id,
                cost_usd: r.cost_usd,
                recorded_at: r.recorded_at,
            })
            .collect())
    }
}

/// Convert a `surrealdb::types::Value` to a standard `serde_json::Value`.
///
/// `surrealdb_types::Value` implements `Serialize` but uses enum-variant JSON:
/// `{"Object": {"name": {"String": "Server"}, ...}}`, `{"Array": [...]}`, `"Null"`.
/// This function recursively unwraps those variants into flat standard JSON.
/// Convert a SurrealDB value into a JSON value. Public so the live-bus can
/// reuse the same serialization path used by every other persistence method.
pub fn value_to_json(v: surrealdb::types::Value) -> Result<serde_json::Value> {
    surreal_to_json(v)
}

pub(crate) fn surreal_to_json(v: surrealdb::types::Value) -> Result<serde_json::Value> {
    let raw = serde_json::to_value(&v).context("serialising surrealdb Value")?;
    Ok(unwrap_surreal_value(raw))
}

/// Recursively unwrap surrealdb_types enum-variant JSON into flat JSON.
///
/// Handles:
/// - `{"String": "foo"}` → `"foo"`
/// - `{"Object": {fields}}` → `{fields}` (each field recursively unwrapped)
/// - `{"Array": [...]}` → `[...]` (each element recursively unwrapped)
/// - `{"Integer": n}` / `{"Float": n}` / `{"Number": n}` → the numeric value
/// - `{"Bool": b}` → `b`
/// - `{"DateTime": s}` → `s`
/// - `{"Uuid": s}` / `{"RecordId": ...}` → handled as opaque string/object
/// - `"Null"` (string) / `null` → `null`
/// - Raw primitives (from nested serialization) → passed-through as-is
fn unwrap_surreal_value(v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        // `"Null"` comes through as the JSON string "Null"
        J::String(ref s) if s == "Null" => J::Null,
        // Already a raw primitive — return as-is
        J::Bool(_) | J::Number(_) | J::Null | J::String(_) => v,
        J::Array(arr) => J::Array(arr.into_iter().map(unwrap_surreal_value).collect()),
        J::Object(map) => {
            // SurrealDB enum wrappers are single-key objects whose key is the variant name.
            if map.len() == 1 {
                let (variant, inner) = map.into_iter().next().unwrap();
                match variant.as_str() {
                    "String" | "DateTime" => {
                        // unwrap: may itself be a wrapped string
                        unwrap_surreal_value(inner)
                    }
                    "Integer" | "Float" => unwrap_surreal_value(inner),
                    "Bool" => unwrap_surreal_value(inner),
                    "Object" => {
                        // inner is an JSON object of wrapped field values
                        if let J::Object(fields) = inner {
                            J::Object(
                                fields
                                    .into_iter()
                                    .map(|(k, v)| (k, unwrap_surreal_value(v)))
                                    .collect(),
                            )
                        } else {
                            unwrap_surreal_value(inner)
                        }
                    }
                    "Array" => {
                        if let J::Array(arr) = inner {
                            J::Array(arr.into_iter().map(unwrap_surreal_value).collect())
                        } else {
                            unwrap_surreal_value(inner)
                        }
                    }
                    // RecordId: {"tb": "<table>", "id": <wrapped-id>} or
                    // {"table": "<table>", "key": <wrapped-id>} depending on
                    // surrealdb crate version.
                    // Extract the inner `id` field so that record IDs round-trip
                    // back to their original string/number value.
                    "RecordId" => {
                        if let J::Object(ref fields) = inner {
                            if let Some(id_val) = fields.get("id").or_else(|| fields.get("key")) {
                                let extracted = unwrap_surreal_value(id_val.clone());
                                if !matches!(extracted, J::Null) {
                                    return extracted;
                                }
                            }
                        }
                        J::Null
                    }
                    // Uuid wraps a UUID string directly — unwrap it.
                    "Uuid" => unwrap_surreal_value(inner),
                    // Bytes, Regexp, Range — not meaningful as application values.
                    "Bytes" | "Regexp" | "Range" => J::Null,
                    // Unknown single-key variant — recurse into the inner value
                    _ => unwrap_surreal_value(inner),
                }
            } else {
                // Multi-key object — just recurse into each field (shouldn't normally occur
                // at the top variant level but supports future-proofing)
                J::Object(
                    map.into_iter()
                        .map(|(k, v)| (k, unwrap_surreal_value(v)))
                        .collect(),
                )
            }
        }
    }
}

// =============================================================================
// SurrealDB row → domain type adapters
// =============================================================================
//
// SurrealDB returns record IDs in its own format (a Thing struct that serialises
// as e.g. `{"tb":"settings_types","id":{"String":"server"}}`) rather than a
// Uuid. We use lightweight sub-structs that accept serde_json::Value and convert
// to our domain types without touching the public API structs.

/// Convert a raw SurrealDB JSON value to `SettingsType`.
fn surreal_value_to_settings_type(
    v: serde_json::Value,
) -> Result<crate::uar::settings::schema::SettingsType> {
    use serde_json::Value as V;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("expected object, got: {v}"))?;

    // The key is the SurrealDB record identifier sub-field (String variant).
    let key = obj.get("key").and_then(V::as_str).unwrap_or("").to_string();

    let name = obj
        .get("name")
        .and_then(V::as_str)
        .unwrap_or(&key)
        .to_string();

    let schema = obj
        .get("schema")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let created_at = obj
        .get("created_at")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_else(chrono::Utc::now);

    let updated_at = obj
        .get("updated_at")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    Ok(crate::uar::settings::schema::SettingsType {
        id: uuid::Uuid::new_v4(), // stable in-memory proxy; SurrealDB uses key as real ID
        name,
        key,
        schema,
        created_at,
        updated_at,
    })
}

/// Convert a raw SurrealDB JSON value to `Settings`.
fn surreal_value_to_setting(
    v: serde_json::Value,
) -> Result<crate::uar::settings::schema::Settings> {
    use serde_json::Value as V;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("expected object, got: {v}"))?;

    let key = obj.get("key").and_then(V::as_str).unwrap_or("").to_string();

    let name = obj
        .get("name")
        .and_then(V::as_str)
        .unwrap_or(&key)
        .to_string();

    let data = obj.get("data").cloned().unwrap_or(serde_json::Value::Null);

    let parent_id: Option<uuid::Uuid> = obj
        .get("parent_id")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let created_at = obj
        .get("created_at")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_else(chrono::Utc::now);

    let updated_at = obj
        .get("updated_at")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    Ok(crate::uar::settings::schema::Settings {
        id: uuid::Uuid::new_v4(),            // in-memory proxy
        settings_type_id: uuid::Uuid::nil(), // looked up via settings_type_key if needed
        name,
        key,
        data,
        parent_id,
        created_at,
        updated_at,
    })
}

// =============================================================================
// Helper: JSON Schema validation
// =============================================================================

/// Validate `data` against a JSON Schema using the `jsonschema` crate (v0.29 API).
fn validate_against_schema(
    data: &serde_json::Value,
    schema: &serde_json::Value,
    setting_key: &str,
) -> Result<()> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|e| anyhow::anyhow!("Invalid JSON Schema for setting '{setting_key}': {e}"))?;
    let errors: Vec<String> = validator.iter_errors(data).map(|e| e.to_string()).collect();
    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Setting '{}' data failed JSON Schema validation:\n{}",
            setting_key,
            errors.join("\n")
        ));
    }
    Ok(())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// Stale-index self-identification (fix-embeddings-fastembed, design D4):
/// chunks ingested before real embedding inference landed carry zero-vector
/// embeddings and can never match anything. Instead of silently returning
/// empty results, name the affected knowledge bases and point at re-ingestion.
fn warn_zero_norm_chunks(chunks: &[KnowledgeChunk]) {
    let mut stale_kbs: Vec<&str> = chunks
        .iter()
        .filter(|c| c.embedding.iter().all(|x| *x == 0.0))
        .map(|c| c.kb_id.as_str())
        .collect();
    stale_kbs.sort_unstable();
    stale_kbs.dedup();
    if !stale_kbs.is_empty() {
        tracing::error!(
            knowledge_bases = %stale_kbs.join(", "),
            "knowledge base contains zero-vector (stale) chunk embeddings from a \
             pre-fix index — these chunks can never match; re-ingest the affected \
             documents (see the upgrade guide)"
        );
    }
}

fn surreal_json_to_attachment_meta(
    v: serde_json::Value,
) -> anyhow::Result<crate::uar::persistence::AttachmentMeta> {
    use anyhow::Context as _;
    let id = v["id"].as_str().unwrap_or_default().to_string();
    let session_id = v["session_id"].as_str().unwrap_or_default().to_string();
    let filename = v["filename"].as_str().unwrap_or_default().to_string();
    let content_type = v["content_type"].as_str().unwrap_or_default().to_string();
    let file_path = v["file_path"].as_str().unwrap_or_default().to_string();
    let file_size = v["file_size"].as_i64().unwrap_or(0);
    let is_image = v["is_image"].as_bool().unwrap_or(false);
    let text_content = v["text_content"].as_str().map(ToString::to_string);
    let created_at_str = v["created_at"].as_str().unwrap_or("1970-01-01T00:00:00Z");
    let created_at: chrono::DateTime<chrono::Utc> =
        chrono::DateTime::parse_from_rfc3339(created_at_str)
            .context("parse created_at")?
            .with_timezone(&chrono::Utc);
    Ok(crate::uar::persistence::AttachmentMeta {
        id,
        session_id,
        filename,
        content_type,
        file_path,
        file_size,
        is_image,
        text_content,
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::unwrap_surreal_value;
    use serde_json::json;

    #[test]
    fn unwrap_record_id_supports_table_key_shape() {
        let value = json!({
            "RecordId": {
                "table": "knowledge_documents",
                "key": { "String": "doc-123" }
            }
        });

        assert_eq!(unwrap_surreal_value(value), json!("doc-123"));
    }

    #[tokio::test]
    async fn document_status_reaches_indexed_on_embedded_surrealdb() {
        use super::SurrealDbProvider;
        use crate::uar::domain::knowledge::{DocumentStatus, KnowledgeBase, KnowledgeDocument};
        use crate::uar::persistence::PersistenceLayer;

        let dir = tempfile::tempdir().expect("tempdir");
        let endpoint = format!("surrealkv://{}", dir.path().join("status.db").display());
        let provider = SurrealDbProvider::new(&endpoint, None, None, None, None)
            .await
            .expect("connect to embedded SurrealKV");
        let now = chrono::Utc::now().to_rfc3339();
        let kb = KnowledgeBase {
            id: "status-kb".into(),
            owner_id: "status-owner".into(),
            name: "status-kb".into(),
            description: None,
            config: Default::default(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let document = KnowledgeDocument {
            id: "status-doc".into(),
            owner_id: kb.owner_id.clone(),
            kb_id: kb.id.clone(),
            filename: "status.txt".into(),
            file_path: None,
            mime_type: Some("text/plain".into()),
            chunk_count: 1,
            status: DocumentStatus::Pending,
            created_at: now.clone(),
            updated_at: now,
        };

        provider.save_knowledge_base(&kb).await.expect("save KB");
        provider
            .save_document(&document)
            .await
            .expect("save document");
        provider
            .update_document_status(&kb.owner_id, &document.id, &DocumentStatus::Indexed)
            .await
            .expect("update status to indexed");

        let indexed = provider
            .get_document(&kb.owner_id, &document.id)
            .await
            .expect("read updated document")
            .expect("document exists");
        assert_eq!(indexed.status, DocumentStatus::Indexed);
    }

    #[tokio::test]
    async fn knowledge_rows_with_identical_ids_remain_partitioned_by_owner() {
        use super::SurrealDbProvider;
        use crate::uar::domain::knowledge::{
            DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument,
        };
        use crate::uar::persistence::PersistenceLayer;

        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("tenant_knowledge_test.db");
        let endpoint = format!("surrealkv://{}", db_path.display());
        let provider = SurrealDbProvider::new(&endpoint, None, None, None, None)
            .await
            .expect("connect to embedded SurrealKV");
        let now = chrono::Utc::now().to_rfc3339();
        let chunk_id = uuid::Uuid::new_v4();

        for owner in ["alice", "bob"] {
            let kb = KnowledgeBase {
                id: "shared-kb-id".into(),
                owner_id: owner.into(),
                name: format!("{owner}-private"),
                description: None,
                config: Default::default(),
                created_at: now.clone(),
                updated_at: now.clone(),
            };
            let document = KnowledgeDocument {
                id: "shared-doc-id".into(),
                owner_id: owner.into(),
                kb_id: kb.id.clone(),
                filename: format!("{owner}.txt"),
                file_path: None,
                mime_type: Some("text/plain".into()),
                chunk_count: 1,
                status: DocumentStatus::Indexed,
                created_at: now.clone(),
                updated_at: now.clone(),
            };
            let chunk = KnowledgeChunk {
                id: chunk_id,
                owner_id: owner.into(),
                kb_id: kb.id.clone(),
                document_id: Some(document.id.clone()),
                content: format!("{owner} secret"),
                metadata: None,
                embedding: vec![1.0],
                created_at: now.clone(),
            };

            provider.save_knowledge_base(&kb).await.expect("save KB");
            provider
                .save_document(&document)
                .await
                .expect("save document");
            provider.save_chunk(&chunk).await.expect("save chunk");
        }

        for owner in ["alice", "bob"] {
            let kb = provider
                .get_knowledge_base(owner, "shared-kb-id")
                .await
                .expect("get KB")
                .expect("owned KB");
            assert_eq!(kb.owner_id, owner);
            let document = provider
                .get_document(owner, "shared-doc-id")
                .await
                .expect("get document")
                .expect("owned document");
            assert_eq!(document.owner_id, owner);
            let matches = provider
                .search_knowledge(owner, &[1.0], 10, -1.0)
                .await
                .expect("search chunks");
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].chunk.owner_id, owner);
            assert_eq!(matches[0].chunk.content, format!("{owner} secret"));
        }

        provider
            .delete_knowledge_base("bob", "shared-kb-id")
            .await
            .expect("delete Bob KB");
        assert!(
            provider
                .get_knowledge_base("alice", "shared-kb-id")
                .await
                .expect("get Alice KB after Bob deletion")
                .is_some()
        );
        assert_eq!(
            provider
                .search_knowledge("alice", &[1.0], 10, -1.0)
                .await
                .expect("search Alice after Bob deletion")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn legacy_session_is_preserved_as_anonymous_without_becoming_claimable() {
        use super::{SurrealDbProvider, to_db_value};
        use crate::session::{ANONYMOUS_SESSION_OWNER, SessionStore};
        use crate::uar::persistence::PersistenceLayer;

        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("legacy_session_test.db");
        let endpoint = format!("surrealkv://{}", db_path.display());
        let provider = SurrealDbProvider::new(&endpoint, None, None, None, None)
            .await
            .expect("connect to embedded SurrealKV");
        let legacy_id = "legacy-session";
        let legacy = SessionStore::new().get_or_create(legacy_id);
        let mut payload = to_db_value(&legacy).expect("serialize legacy session");
        payload
            .as_object_mut()
            .expect("session object")
            .remove("owner_id");
        let _: Option<surrealdb::types::Value> = provider
            .client()
            .upsert(("sessions", legacy_id))
            .content(payload)
            .await
            .expect("write legacy row");

        assert!(
            provider
                .load_session("alice", legacy_id)
                .await
                .expect("authenticated lookup")
                .is_none(),
            "an authenticated caller must not claim an ownerless legacy row"
        );
        let loaded = provider
            .load_session(ANONYMOUS_SESSION_OWNER, legacy_id)
            .await
            .expect("anonymous lookup")
            .expect("legacy session preserved");
        assert_eq!(loaded.id(), legacy_id);
        assert_eq!(loaded.owner_id(), ANONYMOUS_SESSION_OWNER);
        assert!(
            provider
                .load_session(ANONYMOUS_SESSION_OWNER, legacy_id)
                .await
                .expect("migrated lookup")
                .is_some(),
            "lazy migration must remain readable"
        );
    }

    #[tokio::test]
    async fn cost_ledger_round_trips_against_a_real_embedded_db() {
        use super::SurrealDbProvider;
        use crate::uar::persistence::PersistenceLayer;

        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("cost_ledger_test.db");
        let endpoint = format!("surrealkv://{}", db_path.display());
        let provider = SurrealDbProvider::new(&endpoint, None, None, None, None)
            .await
            .expect("connect to embedded SurrealKV");

        provider
            .record_cost_entry("agent", "agent-1", 0.05)
            .await
            .expect("record first entry");
        provider
            .record_cost_entry("agent", "agent-1", 0.10)
            .await
            .expect("record second entry");
        provider
            .record_cost_entry("agent", "agent-2", 1.00)
            .await
            .expect("record unrelated-scope entry");

        let history = provider
            .list_cost_history("agent", "agent-1")
            .await
            .expect("list history");

        assert_eq!(history.len(), 2, "only agent-1's 2 entries, not agent-2's");
        assert_eq!(history[0].scope, "agent");
        assert_eq!(history[0].scope_id, "agent-1");
        assert!((history[0].cost_usd - 0.05).abs() < 1e-9);
        assert!((history[1].cost_usd - 0.10).abs() < 1e-9);
        assert!(
            history[0].recorded_at <= history[1].recorded_at,
            "ordered by recorded_at ascending"
        );

        let empty = provider
            .list_cost_history("agent", "agent-nonexistent")
            .await
            .expect("list history for unknown scope_id");
        assert!(empty.is_empty());
    }

    #[test]
    fn user_prompt_caching_settings_survive_process_reload() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("user_prompt_caching_reload.db");
        let endpoint = format!("surrealkv://{}", db_path.display());
        let test_name = "uar::persistence::providers::surreal::tests::user_prompt_caching_settings_process_phase";

        for phase in ["save", "load"] {
            let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
                .args(["--exact", test_name, "--nocapture"])
                .env("UAR_PROMPT_CACHING_TEST_PHASE", phase)
                .env("UAR_PROMPT_CACHING_TEST_ENDPOINT", &endpoint)
                .status()
                .expect("run isolated persistence phase");
            assert!(status.success(), "{phase} child process failed");
        }
    }

    #[tokio::test]
    async fn user_prompt_caching_settings_process_phase() {
        use super::SurrealDbProvider;
        use crate::uar::domain::prompt_caching::UserPromptCachingSettings;
        use crate::uar::persistence::PersistenceLayer;

        let Ok(phase) = std::env::var("UAR_PROMPT_CACHING_TEST_PHASE") else {
            return;
        };
        let endpoint = std::env::var("UAR_PROMPT_CACHING_TEST_ENDPOINT").expect("endpoint");
        let principal_id = "v1:t:8:tenant-a:s:3:sam";
        let provider = SurrealDbProvider::new(&endpoint, None, None, None, None)
            .await
            .expect("connect provider");

        if phase == "save" {
            let mut settings = UserPromptCachingSettings::new(principal_id);
            settings.prompt_caching_enabled = Some(true);
            provider
                .save_user_prompt_caching_settings(&settings)
                .await
                .expect("save user settings");
        } else {
            let loaded = provider
                .load_user_prompt_caching_settings(principal_id)
                .await
                .expect("load user settings")
                .expect("persisted record");
            assert_eq!(loaded.user_id, principal_id);
            assert_eq!(loaded.prompt_caching_enabled, Some(true));
        }
    }
}
