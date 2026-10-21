//! Authoritative, caller-scoped snapshots for lifecycle invalidations.
//!
//! Realtime notifications disclose only that an entity topic changed. This
//! module replays the existing read routes through the fully bound router so
//! each route retains its own authentication, authorization, and masking
//! behavior. A failed or incomplete replay never authorizes a client to erase
//! its current normalized graph state.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Extensions, HeaderMap, Method, Request, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;
use tower::ServiceExt;

use crate::uar::realtime::EntityTopic;

const MAX_SNAPSHOT_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone)]
pub(crate) struct LifecycleSnapshotService {
    router: Router,
}

#[derive(Debug, Serialize)]
pub(crate) struct LifecycleSnapshot {
    version: u8,
    topic: &'static str,
    generation: u64,
    complete: bool,
    records: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

impl LifecycleSnapshotService {
    /// The router must already contain the production list routes and their
    /// state. The outer server owns when this clone is captured and wires the
    /// public snapshot endpoint.
    pub(crate) fn new(router: Router) -> Self {
        Self { router }
    }

    /// Replays established GET routes with the original authenticated request
    /// context. The caller supplies request extensions by value so every
    /// internal request receives an exact clone of the trusted context.
    pub(crate) async fn response(
        &self,
        topic: EntityTopic,
        headers: HeaderMap,
        extensions: Extensions,
        generation: u64,
    ) -> Response {
        let result = match topic {
            EntityTopic::KnowledgeBases => self.array("/api/knowledge", &headers, &extensions).await,
            EntityTopic::KnowledgeDocuments => self.documents(&headers, &extensions).await,
            EntityTopic::Agents => self.agents(&headers, &extensions).await,
            EntityTopic::Providers => self.providers(&headers, &extensions).await,
            EntityTopic::Models => self.models(&headers, &extensions).await,
            EntityTopic::Skills => self.skills(&headers, &extensions).await,
            EntityTopic::Settings => self.array("/api/uar/settings", &headers, &extensions).await,
            EntityTopic::Memory => self.memory(&headers, &extensions).await,
            EntityTopic::CompilerSessions => self.array("/api/compiler/sessions", &headers, &extensions).await,
            EntityTopic::Threads => self.threads(&headers, &extensions).await,
            _ => Err("topic_not_public"),
        };

        let snapshot = match result {
            Ok(records) => LifecycleSnapshot {
                version: 1,
                topic: topic.as_str(),
                generation,
                complete: true,
                records,
                reason: None,
            },
            Err(reason) => LifecycleSnapshot {
                version: 1,
                topic: topic.as_str(),
                generation,
                complete: false,
                records: Vec::new(),
                reason: Some(reason),
            },
        };
        (StatusCode::OK, axum::Json(snapshot)).into_response()
    }

    async fn documents(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let bases = self.array("/api/knowledge", headers, extensions).await?;
        let mut records = Vec::new();
        for base in bases {
            let id = base.get("id").and_then(Value::as_str).ok_or("invalid_knowledge_base")?;
            if !safe_path_segment(id) {
                return Err("invalid_knowledge_base");
            }
            let path = format!("/api/knowledge/{id}/documents");
            records.extend(self.array(&path, headers, extensions).await?);
        }
        unique_records(records)
    }

    async fn agents(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let payload = self.json("/api/agents", headers, extensions).await?;
        let object = payload.as_object().ok_or("invalid_agents_response")?;
        let runtime = object.get("runtime_agents").and_then(Value::as_array).ok_or("invalid_agents_response")?;
        let federated = object.get("federated_agents").and_then(Value::as_array).ok_or("invalid_agents_response")?;
        let mut records = Vec::with_capacity(runtime.len() + federated.len());
        for (source, rows) in [("runtime", runtime), ("federated", federated)] {
            for row in rows {
                let mut record = row.as_object().cloned().ok_or("invalid_agent_record")?;
                record.insert("_type".to_owned(), Value::String(source.to_owned()));
                records.push(Value::Object(record));
            }
        }
        unique_records(records)
    }

    async fn providers(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let payload = self.json("/api/uar/providers", headers, extensions).await?;
        let records = payload
            .get("providers")
            .and_then(Value::as_array)
            .ok_or("invalid_providers_response")?
            .clone();
        unique_records(records)
    }

    async fn skills(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let payload = self.json("/api/skills", headers, extensions).await?;
        let rows = payload.as_array().ok_or("invalid_skills_response")?;
        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            let mut record = row.as_object().cloned().ok_or("invalid_skill_record")?;
            let id = record
                .get("skill_id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .ok_or("invalid_skill_record")?;
            record.insert("id".to_owned(), Value::String(id.to_owned()));
            records.push(Value::Object(record));
        }
        unique_records(records)
    }

    async fn models(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let payload = self.json("/api/models", headers, extensions).await?;
        let providers = payload.as_object().ok_or("invalid_models_response")?;
        let mut records = Vec::new();
        for (provider_id, provider) in providers {
            let models = provider
                .get("models")
                .and_then(Value::as_object)
                .ok_or("invalid_models_response")?;
            for (model_id, model) in models {
                let mut record = model.as_object().cloned().ok_or("invalid_model_record")?;
                // `features/models/model/models-graph.ts` already owns this
                // provider-qualified graph identity. Reuse it instead of
                // letting two providers overwrite the same model row.
                record.insert(
                    "id".to_owned(),
                    Value::String(format!("{provider_id}/{model_id}")),
                );
                record.insert("model_id".to_owned(), Value::String(model_id.clone()));
                record.insert("provider_id".to_owned(), Value::String(provider_id.clone()));
                records.push(Value::Object(record));
            }
        }
        unique_records(records)
    }

    async fn threads(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let sessions = self.array("/api/uar/sessions", headers, extensions).await?;
        let mut records = Vec::with_capacity(sessions.len());
        for session in sessions {
            let object = session.as_object().ok_or("invalid_session_record")?;
            let id = object
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .ok_or("invalid_session_record")?;
            let created_at = object
                .get("created_at")
                .and_then(Value::as_str)
                .ok_or("invalid_session_record")?;
            let updated_at = object
                .get("last_activity")
                .and_then(Value::as_str)
                .ok_or("invalid_session_record")?;
            // The session store has no durable title field. The established
            // thread-registry fallback is the only truthful Thread title.
            records.push(serde_json::json!({
                "id": id,
                "title": "New conversation",
                "created_at": created_at,
                "updated_at": updated_at,
            }));
        }
        unique_records(records)
    }

    async fn memory(
        &self,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        let payload = self.json("/api/admin/memories", headers, extensions).await?;
        let records = payload
            .get("items")
            .and_then(Value::as_array)
            .ok_or("invalid_memory_response")?
            .clone();
        unique_records(records)
    }

    async fn array(
        &self,
        path: &str,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Vec<Value>, &'static str> {
        self.json(path, headers, extensions)
            .await?
            .as_array()
            .cloned()
            .ok_or("invalid_list_response")
            .and_then(unique_records)
    }

    async fn json(
        &self,
        path: &str,
        headers: &HeaderMap,
        extensions: &Extensions,
    ) -> Result<Value, &'static str> {
        let mut request = Request::builder()
            .method(Method::GET)
            .uri(path)
            .body(Body::empty())
            .map_err(|_| "invalid_snapshot_request")?;
        *request.headers_mut() = headers.clone();
        *request.extensions_mut() = extensions.clone();

        let response = match self.router.clone().oneshot(request).await {
            Ok(response) => response,
            Err(never) => match never {},
        };
        if !response.status().is_success() {
            return Err("snapshot_source_unavailable");
        }
        let body = to_bytes(response.into_body(), MAX_SNAPSHOT_BYTES)
            .await
            .map_err(|_| "snapshot_source_unavailable")?;
        serde_json::from_slice(&body).map_err(|_| "invalid_snapshot_response")
    }
}

fn unique_records(records: Vec<Value>) -> Result<Vec<Value>, &'static str> {
    let mut ids = std::collections::HashSet::with_capacity(records.len());
    for record in &records {
        let id = record
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or("invalid_snapshot_record")?;
        if !ids.insert(id.to_owned()) {
            return Err("ambiguous_snapshot_identity");
        }
    }
    Ok(records)
}

fn safe_path_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}
