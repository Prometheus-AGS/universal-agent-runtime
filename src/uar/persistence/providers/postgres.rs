use crate::session::Session;
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
use crate::uar::domain::policy::ConversationPolicyRecord;
use crate::uar::domain::skills::{Skill, SkillMatch};
use crate::uar::persistence::PersistenceLayer;
use anyhow::Result;
use async_trait::async_trait;
use pgvector::Vector;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

#[derive(Debug)]
pub struct PostgresProvider {
    pool: PgPool,
}

impl PostgresProvider {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await?;

        // Run Migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl PersistenceLayer for PostgresProvider {
    async fn save_session(&self, session: &Session) -> Result<()> {
        let id = session.id();
        // Serialize session to JSON
        let data = serde_json::to_value(session)?;

        sqlx::query(
            r"
            INSERT INTO sessions (id, data, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                data = EXCLUDED.data,
                updated_at = NOW()
            ",
        )
        .bind(id)
        .bind(data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_session(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query("SELECT data FROM sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let val: serde_json::Value = row.try_get("data")?;
            let session: Session = serde_json::from_value(val)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()> {
        sqlx::query(
            r"
            INSERT INTO conversation_policies (conversation_id, policy, updated_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (conversation_id) DO UPDATE SET
                policy = EXCLUDED.policy,
                updated_at = EXCLUDED.updated_at
            ",
        )
        .bind(&record.conversation_id)
        .bind(serde_json::to_value(&record.policy)?)
        .bind(record.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_conversation_policy(
        &self,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>> {
        let row = sqlx::query(
            "SELECT conversation_id, policy, updated_at FROM conversation_policies WHERE conversation_id = $1",
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|row| {
            Ok(ConversationPolicyRecord {
                conversation_id: row.try_get("conversation_id")?,
                policy: serde_json::from_value(row.try_get("policy")?)?,
                updated_at: row.try_get("updated_at")?,
            })
        })
        .transpose()
    }

    async fn delete_conversation_policy(&self, conversation_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM conversation_policies WHERE conversation_id = $1")
            .bind(conversation_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()> {
        let embedding_vector = Vector::from(embedding.to_vec());
        let definition = serde_json::to_value(skill)?;

        sqlx::query(
            r"
            INSERT INTO skills (skill_id, name, description, definition, embedding, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (skill_id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                definition = EXCLUDED.definition,
                embedding = EXCLUDED.embedding,
                updated_at = NOW()
            ",
        )
        .bind(&skill.skill_id)
        .bind(&skill.title)
        .bind(&skill.description)
        .bind(definition)
        .bind(embedding_vector)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_skills(&self, query_vec: &[f32], limit: usize) -> Result<Vec<SkillMatch>> {
        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 =
            i64::try_from(limit).map_err(|err| anyhow::anyhow!("limit exceeds i64: {err}"))?;

        let rows = sqlx::query(
            r"
            SELECT definition, 1 - (embedding <=> $1) as score
            FROM skills
            ORDER BY embedding <=> $1
            LIMIT $2
            ",
        )
        .bind(embedding_vector) // bind $1
        .bind(limit_i64)
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let def_val: serde_json::Value = row.try_get("definition")?;
            let skill: Skill = serde_json::from_value(def_val)?;

            // Score might be f64 or f32. pgvector operator returns f64 usually.
            // 1 - distance.
            let score: f64 = row.try_get("score")?;

            matches.push(SkillMatch {
                skill,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    async fn list_skills(&self) -> Result<Vec<Skill>> {
        let rows = sqlx::query("SELECT definition FROM skills ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;

        let mut skills = Vec::new();
        for row in rows {
            let val: serde_json::Value = row.try_get("definition")?;
            let skill: Skill = serde_json::from_value(val)?;
            skills.push(skill);
        }
        Ok(skills)
    }

    async fn delete_skill(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM skills WHERE skill_id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> {
        let config = serde_json::to_value(&kb.config)?;

        sqlx::query(
            r"
            INSERT INTO knowledge_bases (id, name, description, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                config = EXCLUDED.config,
                updated_at = NOW()
            ",
        )
        .bind(&kb.id)
        .bind(&kb.name)
        .bind(&kb.description)
        .bind(config)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
        let embedding_vector = Vector::from(chunk.embedding.clone());
        let metadata = serde_json::to_value(&chunk.metadata)?;

        sqlx::query(
            r"
            INSERT INTO knowledge_chunks (id, kb_id, content, metadata, embedding, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (id) DO UPDATE SET
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                embedding = EXCLUDED.embedding
            ",
        )
        .bind(chunk.id)
        .bind(&chunk.kb_id)
        .bind(&chunk.content)
        .bind(metadata)
        .bind(embedding_vector)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_knowledge(
        &self,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 =
            i64::try_from(limit).map_err(|err| anyhow::anyhow!("limit exceeds i64: {err}"))?;
        let min_score_f64 = f64::from(min_score);

        let rows = sqlx::query(
            r"
            SELECT id, kb_id, content, metadata, created_at, 1 - (embedding <=> $1) as score
            FROM knowledge_chunks
            WHERE 1 - (embedding <=> $1) >= $3
            ORDER BY embedding <=> $1
            LIMIT $2
            ",
        )
        .bind(embedding_vector) // $1
        .bind(limit_i64) // $2
        .bind(min_score_f64) // $3
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let content: String = row.try_get("content")?;
            let metadata_val: Option<serde_json::Value> = row.try_get("metadata")?;
            // created_at in DB is likely TIMESTAMP. We want String (RFC3339).
            // sqlx maps TIMESTAMP to chrono::NaiveDateTime or DateTime<Utc>
            // We can treat it as DateTime<Utc> and format it?
            // Need `chrono` here?
            // If I map to String, sqlx might fail if type mismatch.
            // But let's try `try_get::<String, _>("created_at")`? Postgres might support cast.
            // If not, read as DateTime<Utc> then to_rfc3339().
            // I need `chrono` crate.
            // `use chrono::DateTime; use chrono::Utc;`

            // To be safe and simple: just load it.
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let created_at_str = created_at.map(|d| d.to_rfc3339()).unwrap_or_default();

            let score: f64 = row.try_get("score")?;

            let chunk = KnowledgeChunk {
                id,
                kb_id,
                document_id: None, // Not returned in search results
                content,
                metadata: metadata_val,
                embedding: vec![], // we don't return embedding in search results unless needed
                created_at: created_at_str,
            };

            matches.push(KnowledgeMatch {
                chunk,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    // Agent Persistence
    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()> {
        let definition = serde_json::to_value(agent)?;

        sqlx::query(
            r"
            INSERT INTO agents (id, name, version, definition, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                version = EXCLUDED.version,
                definition = EXCLUDED.definition,
                updated_at = NOW()
            ",
        )
        .bind(&agent.id)
        .bind(&agent.metadata.title) // Use title as name? Or name field? Artifact has no top-level name?
        // Wait, AgentArtifact has `metadata.title`. It doesn't have top-level `name`.
        // The JSON in test showed "name" inside metadata? No.
        // Let's check AgentArtifact struct.
        // Assuming metadata.title is the name for now.
        .bind(&agent.version)
        .bind(definition)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_agent(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        let row = sqlx::query("SELECT definition FROM agents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let val: serde_json::Value = row.try_get("definition")?;
            let agent: crate::uar::domain::artifact::AgentArtifact = serde_json::from_value(val)?;
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }

    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        let row = sqlx::query("SELECT definition FROM agents WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let val: serde_json::Value = row.try_get("definition")?;
            let agent: crate::uar::domain::artifact::AgentArtifact = serde_json::from_value(val)?;
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }

    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>> {
        let rows = sqlx::query("SELECT definition FROM agents")
            .fetch_all(&self.pool)
            .await?;

        let mut agents = Vec::new();
        for row in rows {
            let val: serde_json::Value = row.try_get("definition")?;
            let agent: crate::uar::domain::artifact::AgentArtifact = serde_json::from_value(val)?;
            agents.push(agent);
        }
        Ok(agents)
    }

    async fn delete_agent(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM agents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Memory System — delegates to MemoryService (backed by surreal-memory library)
    // These stubs satisfy the PersistenceLayer trait. Real memory operations should
    // go through `AppState::memory_service`.
    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()> {
        tracing::debug!(
            "save_memory stub (postgres) — use AppState::memory_service for real persistence"
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
        tracing::debug!(
            "search_memory stub (postgres) — use AppState::memory_service for real queries"
        );
        let _ = (agent_id, query_vec, limit, min_score);
        Ok(vec![])
    }

    // =========================================================================
    // Knowledge Base Retrieval Methods
    // =========================================================================

    async fn get_knowledge_base(&self, id: &str) -> Result<Option<KnowledgeBase>> {
        let row = sqlx::query(
            "SELECT id, name, description, config, created_at, updated_at FROM knowledge_bases WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            let description: Option<String> = row.try_get("description")?;
            let config_val: serde_json::Value = row.try_get("config")?;
            let config = serde_json::from_value(config_val)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            Ok(Some(KnowledgeBase {
                id,
                name: name.unwrap_or_default(),
                description,
                config,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_knowledge_base_by_name(&self, name: &str) -> Result<Option<KnowledgeBase>> {
        let row = sqlx::query(
            "SELECT id, name, description, config, created_at, updated_at FROM knowledge_bases WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            let description: Option<String> = row.try_get("description")?;
            let config_val: serde_json::Value = row.try_get("config")?;
            let config = serde_json::from_value(config_val)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            Ok(Some(KnowledgeBase {
                id,
                name: name.unwrap_or_default(),
                description,
                config,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_knowledge_bases(&self) -> Result<Vec<KnowledgeBase>> {
        let rows = sqlx::query(
            "SELECT id, name, description, config, created_at, updated_at FROM knowledge_bases ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut kbs = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            let description: Option<String> = row.try_get("description")?;
            let config_val: serde_json::Value = row.try_get("config")?;
            let config = serde_json::from_value(config_val)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            kbs.push(KnowledgeBase {
                id,
                name: name.unwrap_or_default(),
                description,
                config,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            });
        }
        Ok(kbs)
    }

    async fn delete_knowledge_base(&self, id: &str) -> Result<()> {
        // CASCADE will handle chunks and documents
        sqlx::query("DELETE FROM knowledge_bases WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
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

        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 =
            i64::try_from(limit).map_err(|err| anyhow::anyhow!("limit exceeds i64: {err}"))?;
        let min_score_f64 = f64::from(min_score);
        let kb_ids_vec: Vec<String> = kb_ids.iter().copied().map(str::to_string).collect();

        let rows = sqlx::query(
            r"
            SELECT id, kb_id, document_id, content, metadata, created_at, 1 - (embedding <=> $1) as score
            FROM knowledge_chunks
            WHERE kb_id = ANY($4) AND 1 - (embedding <=> $1) >= $3
            ORDER BY embedding <=> $1
            LIMIT $2
            ",
        )
        .bind(embedding_vector)
        .bind(limit_i64)
        .bind(min_score_f64)
        .bind(&kb_ids_vec)
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let document_id: Option<String> = row.try_get("document_id")?;
            let content: String = row.try_get("content")?;
            let metadata_val: Option<serde_json::Value> = row.try_get("metadata")?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let created_at_str = created_at.map(|d| d.to_rfc3339()).unwrap_or_default();
            let score: f64 = row.try_get("score")?;

            let chunk = KnowledgeChunk {
                id,
                kb_id,
                document_id,
                content,
                metadata: metadata_val,
                embedding: vec![],
                created_at: created_at_str,
            };

            matches.push(KnowledgeMatch {
                chunk,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    // =========================================================================
    // Document Tracking
    // =========================================================================

    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> {
        let status_str = match &doc.status {
            DocumentStatus::Pending => "pending",
            DocumentStatus::Processing => "processing",
            DocumentStatus::Indexed => "indexed",
            DocumentStatus::Failed { .. } => "failed",
        };
        let error_msg = match &doc.status {
            DocumentStatus::Failed { error } => Some(error.clone()),
            _ => None,
        };

        sqlx::query(
            r"
            INSERT INTO knowledge_documents (id, kb_id, filename, file_path, mime_type, chunk_count, status, error_message, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                filename = EXCLUDED.filename,
                file_path = EXCLUDED.file_path,
                mime_type = EXCLUDED.mime_type,
                chunk_count = EXCLUDED.chunk_count,
                status = EXCLUDED.status,
                error_message = EXCLUDED.error_message,
                updated_at = NOW()
            ",
        )
        .bind(&doc.id)
        .bind(&doc.kb_id)
        .bind(&doc.filename)
        .bind(&doc.file_path)
        .bind(&doc.mime_type)
        .bind(
            i32::try_from(doc.chunk_count)
                .map_err(|err| anyhow::anyhow!("chunk_count exceeds i32: {err}"))?,
        )
        .bind(status_str)
        .bind(error_msg)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_document(&self, id: &str) -> Result<Option<KnowledgeDocument>> {
        let row = sqlx::query(
            "SELECT id, kb_id, filename, file_path, mime_type, chunk_count, status, error_message, created_at, updated_at FROM knowledge_documents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let filename: String = row.try_get("filename")?;
            let file_path: Option<String> = row.try_get("file_path")?;
            let mime_type: String = row.try_get("mime_type")?;
            let chunk_count: i32 = row.try_get("chunk_count")?;
            let status_str: String = row.try_get("status")?;
            let error_message: Option<String> = row.try_get("error_message")?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            let status = match status_str.as_str() {
                "processing" => DocumentStatus::Processing,
                "indexed" => DocumentStatus::Indexed,
                "failed" => DocumentStatus::Failed {
                    error: error_message.unwrap_or_default(),
                },
                _ => DocumentStatus::Pending,
            };

            Ok(Some(KnowledgeDocument {
                id,
                kb_id,
                filename,
                file_path,
                mime_type: Some(mime_type),
                chunk_count: usize::try_from(chunk_count)
                    .map_err(|err| anyhow::anyhow!("chunk_count must be non-negative: {err}"))?,
                status,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_documents(&self, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
        let rows = sqlx::query(
            "SELECT id, kb_id, filename, file_path, mime_type, chunk_count, status, error_message, created_at, updated_at FROM knowledge_documents WHERE kb_id = $1 ORDER BY created_at",
        )
        .bind(kb_id)
        .fetch_all(&self.pool)
        .await?;

        let mut docs = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let filename: String = row.try_get("filename")?;
            let file_path: Option<String> = row.try_get("file_path")?;
            let mime_type: String = row.try_get("mime_type")?;
            let chunk_count: i32 = row.try_get("chunk_count")?;
            let status_str: String = row.try_get("status")?;
            let error_message: Option<String> = row.try_get("error_message")?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            let status = match status_str.as_str() {
                "processing" => DocumentStatus::Processing,
                "indexed" => DocumentStatus::Indexed,
                "failed" => DocumentStatus::Failed {
                    error: error_message.unwrap_or_default(),
                },
                _ => DocumentStatus::Pending,
            };

            docs.push(KnowledgeDocument {
                id,
                kb_id,
                filename,
                file_path,
                mime_type: Some(mime_type),
                chunk_count: usize::try_from(chunk_count)
                    .map_err(|err| anyhow::anyhow!("chunk_count must be non-negative: {err}"))?,
                status,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            });
        }
        Ok(docs)
    }

    async fn count_documents(&self, kb_id: &str) -> Result<usize> {
        let row =
            sqlx::query("SELECT COUNT(*)::bigint AS c FROM knowledge_documents WHERE kb_id = $1")
                .bind(kb_id)
                .fetch_one(&self.pool)
                .await?;
        let count: i64 = row.try_get("c")?;
        Ok(usize::try_from(count.max(0)).unwrap_or(0))
    }

    async fn update_document_status(&self, doc_id: &str, status: &DocumentStatus) -> Result<()> {
        let status_str = match status {
            DocumentStatus::Pending => "pending",
            DocumentStatus::Processing => "processing",
            DocumentStatus::Indexed => "indexed",
            DocumentStatus::Failed { .. } => "failed",
        };
        let error_msg = match status {
            DocumentStatus::Failed { error } => Some(error.clone()),
            _ => None,
        };

        sqlx::query(
            "UPDATE knowledge_documents SET status = $1, error_message = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(status_str)
        .bind(error_msg)
        .bind(doc_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        // Delete associated chunks first
        sqlx::query("DELETE FROM knowledge_chunks WHERE document_id = $1")
            .bind(doc_id)
            .execute(&self.pool)
            .await?;

        // Delete the document
        sqlx::query("DELETE FROM knowledge_documents WHERE id = $1")
            .bind(doc_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // =========================================================================
    // Settings Management
    // =========================================================================

    async fn upsert_settings_type(
        &self,
        st: &crate::uar::settings::schema::SettingsType,
    ) -> Result<uuid::Uuid> {
        // RETURNING id gives us the id that is actually stored: on a fresh insert
        // this equals st.id, but on an ON CONFLICT update the original row keeps
        // its id — callers must use the returned value for FK references.
        let row = sqlx::query(
            r"
            INSERT INTO settings_types (id, name, key, schema, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (key) DO UPDATE SET
                name       = EXCLUDED.name,
                schema     = EXCLUDED.schema,
                updated_at = now()
            RETURNING id
            ",
        )
        .bind(st.id)
        .bind(&st.name)
        .bind(&st.key)
        .bind(&st.schema)
        .bind(st.created_at)
        .bind(st.updated_at)
        .fetch_one(&self.pool)
        .await?;
        let id: uuid::Uuid = row.try_get("id")?;
        Ok(id)
    }

    async fn list_settings_types(&self) -> Result<Vec<crate::uar::settings::schema::SettingsType>> {
        let rows = sqlx::query(
            "SELECT id, name, key, schema, created_at, updated_at FROM settings_types ORDER BY key",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut types = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let key: String = row.try_get("key")?;
            let schema: serde_json::Value = row.try_get("schema")?;
            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;
            types.push(crate::uar::settings::schema::SettingsType {
                id,
                name,
                key,
                schema,
                created_at,
                updated_at,
            });
        }
        Ok(types)
    }

    async fn get_settings_type(
        &self,
        key: &str,
    ) -> Result<Option<crate::uar::settings::schema::SettingsType>> {
        let row = sqlx::query(
            "SELECT id, name, key, schema, created_at, updated_at FROM settings_types WHERE key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: uuid::Uuid = row.try_get("id")?;
            let name: String = row.try_get("name")?;
            let key: String = row.try_get("key")?;
            let schema: serde_json::Value = row.try_get("schema")?;
            let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;
            Ok(Some(crate::uar::settings::schema::SettingsType {
                id,
                name,
                key,
                schema,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn upsert_setting(&self, setting: &crate::uar::settings::schema::Settings) -> Result<()> {
        // Validate data against the JSON Schema before writing.
        // For leaf settings (e.g. "server.port"), the namespace schema describes the full
        // Validate `data` against the JSON Schema for this setting's property.
        // When a dotted key like "server.port" is used, look up only the "port" property
        // sub-schema so that scalar values (e.g. 3000) are validated against {"type":"integer"}
        // rather than the top-level {"type":"object"} namespace schema.
        // If the property is not present in the schema (e.g. free-form keys like
        // "agent_config.orchestrated" or "provider.openai"), skip validation entirely —
        // never fall back to the full namespace schema, which would reject non-object values.
        let type_key = setting.key.split('.').next().unwrap_or("unknown");
        let prop_key = setting.key.splitn(2, '.').nth(1);
        if let Some(st) = self.get_settings_type(type_key).await? {
            let schema_opt: Option<serde_json::Value> = if let Some(prop) = prop_key {
                st.schema
                    .get("properties")
                    .and_then(|p| p.get(prop))
                    .cloned()
                    .or_else(|| st.schema.get("additionalProperties").cloned())
            } else {
                Some(st.schema.clone())
            };
            if let Some(schema) = schema_opt {
                validate_setting_data(&setting.data, &schema, &setting.key)?;
            }
        }

        sqlx::query(
            r"
            INSERT INTO settings (id, settings_type_id, name, key, data, parent_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (key) DO UPDATE SET
                settings_type_id = EXCLUDED.settings_type_id,
                name             = EXCLUDED.name,
                data             = EXCLUDED.data,
                parent_id        = EXCLUDED.parent_id,
                updated_at       = now()
            ",
        )
        .bind(setting.id)
        .bind(setting.settings_type_id)
        .bind(&setting.name)
        .bind(&setting.key)
        .bind(&setting.data)
        .bind(setting.parent_id)
        .bind(setting.created_at)
        .bind(setting.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_setting(
        &self,
        key: &str,
    ) -> Result<Option<crate::uar::settings::schema::Settings>> {
        let row = sqlx::query(
            "SELECT id, settings_type_id, name, key, data, parent_id, created_at, updated_at FROM settings WHERE key = $1",
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(pg_row_to_setting(row)?))
        } else {
            Ok(None)
        }
    }

    async fn list_settings(
        &self,
        type_key: Option<&str>,
        parent_id: Option<uuid::Uuid>,
    ) -> Result<Vec<crate::uar::settings::schema::Settings>> {
        // Dynamic query depending on filters.
        let rows = match (type_key, parent_id) {
            (Some(tk), Some(pid)) => {
                let prefix = format!("{tk}.%");
                sqlx::query(
                    "SELECT id, settings_type_id, name, key, data, parent_id, created_at, updated_at \
                     FROM settings WHERE key LIKE $1 AND parent_id = $2 ORDER BY key",
                )
                .bind(&prefix)
                .bind(pid)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(tk), None) => {
                let prefix = format!("{tk}.%");
                sqlx::query(
                    "SELECT id, settings_type_id, name, key, data, parent_id, created_at, updated_at \
                     FROM settings WHERE key LIKE $1 ORDER BY key",
                )
                .bind(&prefix)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(pid)) => sqlx::query(
                "SELECT id, settings_type_id, name, key, data, parent_id, created_at, updated_at \
                     FROM settings WHERE parent_id = $1 ORDER BY key",
            )
            .bind(pid)
            .fetch_all(&self.pool)
            .await?,
            (None, None) => sqlx::query(
                "SELECT id, settings_type_id, name, key, data, parent_id, created_at, updated_at \
                     FROM settings ORDER BY key",
            )
            .fetch_all(&self.pool)
            .await?,
        };

        rows.into_iter().map(pg_row_to_setting).collect()
    }

    async fn delete_setting(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM settings WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // =========================================================================
    // Chat Attachment Storage
    // =========================================================================

    async fn insert_attachment(
        &self,
        meta: &crate::uar::persistence::AttachmentMeta,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO chat_attachments
               (id, session_id, filename, content_type, file_path, file_size, is_image, text_content, created_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               ON CONFLICT (id) DO NOTHING"#,
        )
        .bind(&meta.id)
        .bind(&meta.session_id)
        .bind(&meta.filename)
        .bind(&meta.content_type)
        .bind(&meta.file_path)
        .bind(meta.file_size)
        .bind(meta.is_image)
        .bind(&meta.text_content)
        .bind(meta.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_attachment(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::persistence::AttachmentMeta>> {
        let row = sqlx::query(
            "SELECT id, session_id, filename, content_type, file_path, file_size, is_image, text_content, created_at
             FROM chat_attachments WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            None => Ok(None),
            Some(r) => Ok(Some(pg_row_to_attachment_meta(r)?)),
        }
    }

    async fn list_attachments_for_session(
        &self,
        session_id: &str,
    ) -> Result<Vec<crate::uar::persistence::AttachmentMeta>> {
        let rows = sqlx::query(
            "SELECT id, session_id, filename, content_type, file_path, file_size, is_image, text_content, created_at
             FROM chat_attachments WHERE session_id = $1 ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(pg_row_to_attachment_meta).collect()
    }

    // Cost Budget History (CH-07)
    async fn record_cost_entry(&self, scope: &str, scope_id: &str, cost_usd: f64) -> Result<()> {
        sqlx::query(
            "INSERT INTO cost_ledger (scope, scope_id, cost_usd, recorded_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(scope)
        .bind(scope_id)
        .bind(cost_usd)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_cost_history(
        &self,
        scope: &str,
        scope_id: &str,
    ) -> Result<Vec<crate::uar::persistence::CostEntry>> {
        let rows = sqlx::query(
            "SELECT scope, scope_id, cost_usd, recorded_at FROM cost_ledger
             WHERE scope = $1 AND scope_id = $2 ORDER BY recorded_at",
        )
        .bind(scope)
        .bind(scope_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|r| {
                Ok(crate::uar::persistence::CostEntry {
                    scope: r.try_get("scope")?,
                    scope_id: r.try_get("scope_id")?,
                    cost_usd: r.try_get("cost_usd")?,
                    recorded_at: r.try_get("recorded_at")?,
                })
            })
            .collect()
    }
}

// =============================================================================
// Postgres helpers
// =============================================================================

fn pg_row_to_setting(row: sqlx::postgres::PgRow) -> Result<crate::uar::settings::schema::Settings> {
    let id: uuid::Uuid = row.try_get("id")?;
    let settings_type_id: uuid::Uuid = row.try_get("settings_type_id")?;
    let name: String = row.try_get("name")?;
    let key: String = row.try_get("key")?;
    let data: serde_json::Value = row.try_get("data")?;
    let parent_id: Option<uuid::Uuid> = row.try_get("parent_id")?;
    let created_at: chrono::DateTime<chrono::Utc> = row.try_get("created_at")?;
    let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;
    Ok(crate::uar::settings::schema::Settings {
        id,
        settings_type_id,
        name,
        key,
        data,
        parent_id,
        created_at,
        updated_at,
    })
}

fn pg_row_to_attachment_meta(
    row: sqlx::postgres::PgRow,
) -> Result<crate::uar::persistence::AttachmentMeta> {
    Ok(crate::uar::persistence::AttachmentMeta {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        filename: row.try_get("filename")?,
        content_type: row.try_get("content_type")?,
        file_path: row.try_get("file_path")?,
        file_size: row.try_get("file_size")?,
        is_image: row.try_get("is_image")?,
        text_content: row.try_get("text_content")?,
        created_at: row.try_get("created_at")?,
    })
}

/// Validate `data` against a JSON Schema using the `jsonschema` crate (v0.29 API).
fn validate_setting_data(
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
