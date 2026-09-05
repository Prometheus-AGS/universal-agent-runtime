//! OpenAPI specification and Swagger UI integration.
//!
//! Provides an OpenAPI 3.1 spec as a JSON value for the Swagger UI endpoint.

/// Build the OpenAPI specification as a `serde_json::Value`.
/// This avoids utoipa builder API version issues by constructing the spec directly.
#[expect(
    clippy::expect_used,
    reason = "static json! literal is guaranteed to parse"
)]
pub fn build_openapi_spec() -> utoipa::openapi::OpenApi {
    serde_json::from_value(serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Universal Agent Runtime",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Agentic streaming LLM runtime with MCP tool integration, A2A/AG-UI/A2UI protocol support, and a 269-provider discovery catalog.",
            "license": { "name": "MIT" }
        },
        "tags": [
            {"name": "health", "description": "Liveness and readiness probes"},
            {"name": "chat", "description": "Chat completions (OpenAI-compatible)"},
            {"name": "models", "description": "Model listing (OpenAI-compatible)"},
            {"name": "metrics", "description": "Prometheus metrics endpoint"},
            {"name": "tools", "description": "MCP tool discovery and health"},
            {"name": "skills", "description": "Skill management"},
            {"name": "runs", "description": "Governed agent run lifecycle"},
            {"name": "providers", "description": "Runtime provider configuration"},
            {"name": "knowledge", "description": "Tenant-scoped knowledge bases and retrieval"},
            {"name": "auth", "description": "API key management and token exchange"},
            {"name": "realtime", "description": "Realtime entity mutation streams"}
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
            "/api/uar/runs": {
                "post": {
                    "summary": "Start an agent run",
                    "description": "Creates a governed run and returns its identifier and event stream URL",
                    "tags": ["runs"],
                    "requestBody": {
                        "required": true,
                        "content": {"application/json": {"schema": {
                            "type": "object",
                            "required": ["artifact", "input"],
                            "properties": {
                                "artifact": {"type": "object", "description": "Agent artifact defining the run policy and prompt"},
                                "input": {"type": "string"},
                                "session_id": {"type": ["string", "null"]},
                                "skill_attachments": {
                                    "type": "array", "items": {"type": "string"}, "default": [],
                                    "description": "Skill IDs to activate before the first model call, intersected with effective eligibility"
                                }
                            }
                        }}}
                    },
                    "responses": { "200": {
                        "description": "Run created; rejected attachments appear in activation_failures without adding their bodies or tools",
                        "content": {"application/json": {"schema": {
                            "type": "object", "required": ["run_id", "stream_url"],
                            "properties": {
                                "run_id": {"type": "string"},
                                "stream_url": {"type": "string"},
                                "activation_failures": {"type": "array", "items": {
                                    "type": "object", "required": ["code", "skill_id"],
                                    "properties": {
                                        "code": {"type": "string", "enum": ["missing", "ineligible", "disabled", "dependency_invalid", "limit_reached"]},
                                        "skill_id": {"type": "string"},
                                        "reason": {"type": "string"},
                                        "limit": {"type": "integer", "minimum": 0}
                                    }
                                }}
                            }
                        }}}
                    } }
                }
            },
            "/api/uar/runs/{id}/stream": {
                "get": {
                    "summary": "Stream run events",
                    "tags": ["runs"],
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": { "200": { "description": "Normalized SSE event stream" } }
                }
            },
            "/api/uar/providers": {
                "get": {
                    "summary": "List configured providers",
                    "tags": ["providers"],
                    "responses": { "200": { "description": "Provider list" } }
                },
                "post": {
                    "summary": "Create a provider configuration",
                    "tags": ["providers"],
                    "responses": { "201": { "description": "Provider created" } }
                }
            },
            "/api/uar/skills": {
                "get": {
                    "summary": "List skills",
                    "tags": ["skills"],
                    "responses": { "200": { "description": "Skill list" } }
                },
                "post": {
                    "summary": "Create a user skill",
                    "tags": ["skills"],
                    "responses": { "201": { "description": "Skill created" } }
                }
            },
            "/api/uar/skills/refresh": {
                "post": {
                    "summary": "Refresh skills",
                    "description": "Reloads skills from configured storage providers",
                    "tags": ["skills"],
                    "responses": { "200": { "description": "Refresh result" } }
                }
            },
            "/api/uar/skills/reload": {
                "post": {
                    "summary": "Reload skills",
                    "description": "Manually refreshes the active skill registry",
                    "tags": ["skills"],
                    "responses": { "200": { "description": "Reload result" } }
                }
            },
            "/api/uar/knowledge-bases": {
                "get": {
                    "summary": "List knowledge bases",
                    "tags": ["knowledge"],
                    "responses": { "200": { "description": "Tenant-scoped knowledge base list" } }
                },
                "post": {
                    "summary": "Create a knowledge base",
                    "tags": ["knowledge"],
                    "responses": { "201": { "description": "Knowledge base created" } }
                }
            },
            "/api/uar/knowledge-bases/{id}/documents": {
                "get": {
                    "summary": "List knowledge base documents",
                    "tags": ["knowledge"],
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": { "200": { "description": "Document list" } }
                },
                "post": {
                    "summary": "Upload a knowledge base document",
                    "tags": ["knowledge"],
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": { "202": { "description": "Document accepted for ingestion" } }
                }
            },
            "/api/uar/knowledge-bases/{id}/search": {
                "post": {
                    "summary": "Search a knowledge base",
                    "tags": ["knowledge"],
                    "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": { "200": { "description": "Retrieval results" } }
                }
            },
            "/api/uar/auth/keys": {
                "get": {
                    "summary": "List API keys",
                    "tags": ["auth"],
                    "responses": { "200": { "description": "API key metadata" } }
                },
                "post": {
                    "summary": "Create an API key",
                    "tags": ["auth"],
                    "responses": { "201": { "description": "API key created" } }
                }
            },
            "/api/uar/auth/exchange": {
                "post": {
                    "summary": "Exchange an API key for a JWT",
                    "tags": ["auth"],
                    "responses": { "200": { "description": "Short-lived JWT" } }
                }
            },
            "/api/live/{topic}": {
                "get": {
                    "summary": "Stream entity mutations",
                    "tags": ["realtime"],
                    "parameters": [{"name": "topic", "in": "path", "required": true, "schema": {"type": "string"}}],
                    "responses": { "200": { "description": "Entity mutation SSE stream" } }
                }
            }
        }
    }))
    .expect("OpenAPI spec JSON is valid")
}

#[cfg(test)]
mod tests {
    use super::build_openapi_spec;

    #[test]
    fn spec_uses_package_version_and_documents_customer_routes() {
        let spec = serde_json::to_value(build_openapi_spec()).expect("OpenAPI serializes");

        assert_eq!(spec["info"]["version"], env!("CARGO_PKG_VERSION"));
        for path in [
            "/v1/chat/completions",
            "/api/uar/runs",
            "/api/uar/providers",
            "/api/uar/skills",
            "/api/uar/skills/refresh",
            "/api/uar/skills/reload",
            "/api/uar/knowledge-bases",
            "/api/uar/auth/exchange",
            "/api/live/{topic}",
        ] {
            assert!(spec["paths"].get(path).is_some(), "missing path {path}");
        }
        assert!(
            spec["paths"]["/api/uar/providers"]["post"]["responses"]
                .get("201")
                .is_some()
        );
        assert!(
            spec["paths"]["/api/uar/knowledge-bases/{id}/documents"]["post"]["responses"]
                .get("202")
                .is_some()
        );
    }
}
