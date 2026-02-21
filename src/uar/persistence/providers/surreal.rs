use crate::session::Session;
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
use crate::uar::domain::skills::{Skill, SkillMatch};
use crate::uar::persistence::PersistenceLayer;
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
    /// freshly-started SurrealDB server.  For embedded endpoints (RocksDB,
    /// in-memory) authentication is not performed.
    pub async fn new(
        connection_string: &str,
        surreal_user: Option<&str>,
        surreal_pass: Option<&str>,
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

        db.use_ns("uar").use_db("uar").await?;

        tracing::info!("SurrealDB connected successfully");

        Ok(Self { db })
    }

    pub fn client(&self) -> Surreal<Any> {
        self.db.clone()
    }
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
    if trimmed.contains("://")
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
            "rocksdb://./data/uar.db".to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        format!("rocksdb://{trimmed}")
    }
}

fn to_db_value<T: Serialize>(value: &T) -> Result<serde_json::Value> {
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

#[async_trait]
impl PersistenceLayer for SurrealDbProvider {
    // Session Management
    async fn save_session(&self, session: &Session) -> Result<()> {
        let id = session.id().to_string();
        let payload = to_db_value(session)?;
        let _: Option<serde_json::Value> =
            self.db.upsert(("sessions", id)).content(payload).await?;
        Ok(())
    }

    async fn load_session(&self, id: &str) -> Result<Option<Session>> {
        let session: Option<serde_json::Value> = self.db.select(("sessions", id)).await?;
        from_db_opt(session)
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

        let _: Option<serde_json::Value> = self
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

        let skills_raw: Vec<serde_json::Value> = self.db.select("skills").await?;
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

        let records_raw: Vec<serde_json::Value> = self.db.select("skills").await?;
        let records: Vec<SkillRecord> = from_db_vec(records_raw)?;
        Ok(records.into_iter().map(|r| r.skill).collect())
    }

    async fn delete_skill(&self, id: &str) -> Result<()> {
        let _: Option<serde_json::Value> = self.db.delete(("skills", id)).await?;
        Ok(())
    }

    // Knowledge Base Management
    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> {
        let payload = to_db_value(kb)?;
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("knowledge_bases", kb.id.clone()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
        let payload = to_db_value(chunk)?;
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("knowledge_chunks", chunk.id.to_string()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn search_knowledge(
        &self,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        let chunks_raw: Vec<serde_json::Value> = self.db.select("knowledge_chunks").await?;
        let chunks: Vec<KnowledgeChunk> = from_db_vec(chunks_raw)?;

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
        let _: Option<serde_json::Value> = self
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
        let agent: Option<serde_json::Value> = self.db.select(("agents", id)).await?;
        from_db_opt(agent)
    }

    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        // Select where name = $name
        // Assume metadata.title contains name.
        // This is inefficient without index but fine for now.
        let sql = "SELECT * FROM agents WHERE metadata.title = $name LIMIT 1";
        let mut response = self.db.query(sql).bind(("name", name.to_string())).await?;
        let agent: Option<serde_json::Value> = response.take(0)?;
        from_db_opt(agent)
    }

    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>> {
        let agents_raw: Vec<serde_json::Value> = self.db.select("agents").await?;
        from_db_vec(agents_raw)
    }

    // Memory System — delegates to MemoryService (backed by surreal-memory library)
    // These stubs satisfy the PersistenceLayer trait. Real memory operations should
    // use `AppState::memory_service` (a MemoryService wrapping SurrealStorage from
    // the surreal-memory library in its own embedded RocksDB store).
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

    async fn get_knowledge_base(&self, id: &str) -> Result<Option<KnowledgeBase>> {
        let kb: Option<serde_json::Value> = self.db.select(("knowledge_bases", id)).await?;
        from_db_opt(kb)
    }

    async fn get_knowledge_base_by_name(&self, name: &str) -> Result<Option<KnowledgeBase>> {
        let sql = "SELECT * FROM knowledge_bases WHERE name = $name LIMIT 1";
        let mut response = self.db.query(sql).bind(("name", name.to_string())).await?;
        let kb: Option<serde_json::Value> = response.take(0)?;
        from_db_opt(kb)
    }

    async fn list_knowledge_bases(&self) -> Result<Vec<KnowledgeBase>> {
        let kbs_raw: Vec<serde_json::Value> = self.db.select("knowledge_bases").await?;
        from_db_vec(kbs_raw)
    }

    async fn delete_knowledge_base(&self, id: &str) -> Result<()> {
        // Delete the KB - SurrealDB doesn't have FK CASCADE, so we delete related records first
        let _: Option<serde_json::Value> = self.db.delete(("knowledge_bases", id)).await?;
        // Also delete related chunks and documents
        let sql = "DELETE FROM knowledge_chunks WHERE kb_id = $id";
        self.db.query(sql).bind(("id", id.to_string())).await?;
        let sql = "DELETE FROM knowledge_documents WHERE kb_id = $id";
        self.db.query(sql).bind(("id", id.to_string())).await?;
        Ok(())
    }

    // =========================================================================
    // Scoped Knowledge Search
    // =========================================================================

    async fn search_knowledge_scoped(
        &self,
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
        let sql = "SELECT * FROM knowledge_chunks WHERE kb_id IN $kb_ids";
        let mut res = self.db.query(sql).bind(("kb_ids", kb_ids_vec)).await?;
        let chunks_raw: Vec<serde_json::Value> = res.take(0)?;
        let chunks: Vec<KnowledgeChunk> = from_db_vec(chunks_raw)?;

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
        let payload = to_db_value(doc)?;
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("knowledge_documents", doc.id.clone()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn get_document(&self, id: &str) -> Result<Option<KnowledgeDocument>> {
        let doc: Option<serde_json::Value> = self.db.select(("knowledge_documents", id)).await?;
        from_db_opt(doc)
    }

    async fn list_documents(&self, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
        let sql = "SELECT * FROM knowledge_documents WHERE kb_id = $kb_id ORDER BY created_at";
        let mut res = self
            .db
            .query(sql)
            .bind(("kb_id", kb_id.to_string()))
            .await?;
        let docs_raw: Vec<serde_json::Value> = res.take(0)?;
        from_db_vec(docs_raw)
    }

    async fn update_document_status(&self, doc_id: &str, status: &DocumentStatus) -> Result<()> {
        let sql = "UPDATE knowledge_documents SET status = $status, updated_at = time::now() WHERE id = $id";
        self.db
            .query(sql)
            .bind(("id", doc_id.to_string()))
            .bind(("status", serde_json::to_value(status)?))
            .await?;
        Ok(())
    }

    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        // Delete associated chunks first
        let sql = "DELETE FROM knowledge_chunks WHERE document_id = $doc_id";
        self.db
            .query(sql)
            .bind(("doc_id", doc_id.to_string()))
            .await?;

        // Delete the document
        let _: Option<serde_json::Value> = self.db.delete(("knowledge_documents", doc_id)).await?;

        Ok(())
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
        let val: Option<serde_json::Value> = res.take(0)?;
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
        let vals: Vec<serde_json::Value> = res.take(0)?;
        vals.into_iter()
            .map(surreal_json_to_attachment_meta)
            .collect()
    }
}

/// Convert a `surrealdb::types::Value` to a standard `serde_json::Value`.
///
/// `surrealdb_types::Value` implements `Serialize` but uses enum-variant JSON:
/// `{"Object": {"name": {"String": "Server"}, ...}}`, `{"Array": [...]}`, `"Null"`.
/// This function recursively unwraps those variants into flat standard JSON.
fn surreal_to_json(v: surrealdb::types::Value) -> Result<serde_json::Value> {
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
                    // "RecordId", "Uuid", "Bytes", etc. — collapse to null / ignore
                    "RecordId" | "Uuid" | "Bytes" | "Regexp" | "Range" => {
                        // For record IDs embedded in a row the value isn't meaningful
                        // at the application level; drop them so they don't confuse
                        // the row-adapter functions.
                        J::Null
                    }
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
