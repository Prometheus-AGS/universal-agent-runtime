//! Disjoint storage for verified principals and legacy anonymous knowledge.
//!
//! The host supplies the verified principal storage key. A legacy database row
//! whose subject happens to equal that key is never evidence of ownership.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeNamespace {
    /// Missing markers always mean legacy, including canonical-looking subjects.
    #[default]
    Legacy,
    PrincipalV1,
}

impl KnowledgeNamespace {
    /// Called only with the owner established by the trusted host boundary.
    pub fn for_owner(owner: &str) -> Result<Self> {
        if owner.starts_with("v1:k:") {
            Ok(Self::PrincipalV1)
        } else if owner == "anonymous" {
            Ok(Self::Legacy)
        } else {
            anyhow::bail!("knowledge requires a verified principal or the explicit anonymous boundary")
        }
    }

    pub fn is_legacy(&self) -> bool { *self == Self::Legacy }

    /// Rewrite only host-authored SQL identifiers, never caller text or values.
    pub(crate) fn query(self, sql: &str) -> String {
        match self {
            Self::Legacy => sql.to_owned(),
            Self::PrincipalV1 => sql.replace("knowledge_bases", "principal_knowledge_bases")
                .replace("knowledge_documents", "principal_knowledge_documents")
                .replace("knowledge_chunks", "principal_knowledge_chunks")
                .replace("knowledge_ingestion", "principal_knowledge_ingestion"),
        }
    }

    /// The same host-authored rewrite as [`Self::query`], asserted SQL-safe for
    /// sqlx 0.9, which only accepts `&'static str` or [`sqlx::AssertSqlSafe`].
    #[cfg(feature = "sqlx")]
    pub(crate) fn sql(self, sql: &str) -> sqlx::AssertSqlSafe<String> {
        sqlx::AssertSqlSafe(self.query(sql))
    }

    pub(crate) fn table(self, legacy: &'static str) -> &'static str {
        match (self, legacy) {
            (Self::PrincipalV1, "knowledge_bases") => "principal_knowledge_bases",
            (Self::PrincipalV1, "knowledge_documents") => "principal_knowledge_documents",
            (Self::PrincipalV1, "knowledge_chunks") => "principal_knowledge_chunks",
            (Self::PrincipalV1, "knowledge_ingestion") => "principal_knowledge_ingestion",
            _ => legacy,
        }
    }
}
