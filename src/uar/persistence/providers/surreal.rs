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

#[derive(Debug)]
pub struct SurrealDbProvider {
    db: Surreal<Any>,
}

impl SurrealDbProvider {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let endpoint = normalize_endpoint(connection_string);
        tracing::info!("Connecting to SurrealDB: {}", endpoint);

        // The any::connect function will automatically determine the engine type
        // from the connection string (e.g., "rocksdb://./data/uar.db" or "ws://localhost:8000")
        let db = any::connect(endpoint).await?;

        // Use default namespace and database
        db.use_ns("uar").use_db("uar").await?;

        tracing::info!("SurrealDB connected successfully");

        Ok(Self { db })
    }
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

    // Memory System
    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()> {
        // memory has embedding field
        let payload = to_db_value(memory)?;
        let _: Option<serde_json::Value> = self
            .db
            .upsert(("memories", memory.id.clone()))
            .content(payload)
            .await?;
        Ok(())
    }

    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>> {
        // Fetch all (or filter by agent_id first if indexed)
        // Then cosine similarity

        let memories_raw: Vec<serde_json::Value> = if let Some(aid) = agent_id {
            let sql = "SELECT * FROM memories WHERE agent_id = $aid OR agent_id IS NULL";
            let mut res = self.db.query(sql).bind(("aid", aid.to_string())).await?;
            res.take(0)?
        } else {
            // Global only? or ALL? Logic in Postgres was: where (agent_id = $1 OR agent_id IS NULL).
            // If agent_id arg is None, we probably only want global ones (agent_id IS NULL)?
            // Postgres query used: `WHERE (agent_id = $1 OR agent_id IS NULL)`
            // If $1 is NULL, `agent_id = NULL` is false (in SQL usually), so only `agent_id IS NULL` matches.
            // So if input agent_id is None, we fetch globals.
            let sql = "SELECT * FROM memories WHERE agent_id IS NULL";
            let mut res = self.db.query(sql).await?;
            res.take(0)?
        };
        let memories: Vec<crate::uar::domain::memory::Memory> = from_db_vec(memories_raw)?;

        let mut matches: Vec<crate::uar::domain::memory::MemoryMatch> = memories
            .into_iter()
            .map(|m| {
                let score = cosine_similarity(&m.embedding, query_vec);
                crate::uar::domain::memory::MemoryMatch { memory: m, score }
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
