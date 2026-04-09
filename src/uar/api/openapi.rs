//! OpenAPI specification and Swagger UI integration.
//!
//! Provides an OpenAPI 3.1 spec as a JSON value for the Swagger UI endpoint.

/// Build the OpenAPI specification as a `serde_json::Value`.
/// This avoids utoipa builder API version issues by constructing the spec directly.
pub fn build_openapi_spec() -> utoipa::openapi::OpenApi {
    serde_json::from_value(serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Universal Agent Runtime",
            "version": "0.1.0",
            "description": "Agentic streaming LLM runtime with MCP tool integration, A2A/AG-UI/A2UI protocol support, and 142+ LLM provider coverage.",
            "license": { "name": "AGPL-3.0-only" }
        },
        "tags": [
            {"name": "health", "description": "Liveness and readiness probes"},
            {"name": "chat", "description": "Chat completions (OpenAI-compatible)"},
            {"name": "models", "description": "Model listing (OpenAI-compatible)"},
            {"name": "metrics", "description": "Prometheus metrics endpoint"},
            {"name": "tools", "description": "MCP tool discovery and health"},
            {"name": "skills", "description": "Skill management"}
        ],
        "paths": {
            "/healthz": {
                "get": {
                    "summary": "Liveness probe",
                    "description": "Lightweight check — returns 200 if process is alive",
                    "tags": ["health"],
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/readyz": {
                "get": {
                    "summary": "Readiness probe",
                    "description": "Checks PostgreSQL, SurrealDB, and MCP connectivity",
                    "tags": ["health"],
                    "responses": {
                        "200": { "description": "All dependencies ready" },
                        "503": { "description": "One or more dependencies unavailable" }
                    }
                }
            },
            "/v1/models": {
                "get": {
                    "summary": "List available models",
                    "description": "Returns models from configured providers in OpenAI format",
                    "tags": ["models"],
                    "responses": { "200": { "description": "Model list" } }
                }
            },
            "/v1/models/{model_id}": {
                "get": {
                    "summary": "Get model details",
                    "description": "Returns capabilities and limits for a specific model",
                    "tags": ["models"],
                    "parameters": [{"name": "model_id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": {
                        "200": { "description": "Model details" },
                        "404": { "description": "Model not found" }
                    }
                }
            },
            "/v1/chat/completions": {
                "post": {
                    "summary": "Create chat completion",
                    "description": "OpenAI-compatible streaming chat completion with tool calling. Supports stream_mode: openai, agui, or dual.",
                    "tags": ["chat"],
                    "responses": { "200": { "description": "Streaming SSE response" } }
                }
            },
            "/metrics": {
                "get": {
                    "summary": "Prometheus metrics",
                    "description": "Prometheus text exposition format with request, LLM, tool, and session metrics",
                    "tags": ["metrics"],
                    "responses": { "200": { "description": "Prometheus metrics text" } }
                }
            },
            "/api/uar/mcp/health": {
                "get": {
                    "summary": "MCP server health",
                    "description": "Returns connection status and tool count for all configured MCP servers",
                    "tags": ["tools"],
                    "responses": { "200": { "description": "MCP health status" } }
                }
            },
            "/api/uar/skills/reload": {
                "post": {
                    "summary": "Reload skills",
                    "description": "Triggers a refresh of all skills from storage providers",
                    "tags": ["skills"],
                    "responses": { "200": { "description": "Reload result" } }
                }
            }
        }
    }))
    .expect("OpenAPI spec JSON is valid")
}
