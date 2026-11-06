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
    // utoipa 5 accepts operation parameters inline, but does not deserialize
    // reusable components.parameters references.
    let admission_id = serde_json::json!({"name": "admission_id", "in": "path", "required": true, "schema": {"type": "string"}});
    let task_id = serde_json::json!({"name": "task_id", "in": "path", "required": true, "schema": {"type": "string", "pattern": "^fh-"}});
    let workspace_id = serde_json::json!({"name": "x-uar-workspace-id", "in": "header", "required": true, "schema": {"type": "string", "minLength": 1}, "description": "Authenticated workspace partition for admission, reconciliation, observation, and control"});
    let mut spec = serde_json::json!({
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
            {"name": "full-harness", "description": "Process-ephemeral, retry-safe full-run delegation"},
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
                    "description": "Checks the configured persistence backend, the memory service, and the MCP registry",
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
            "/v1/embeddings": {
                "post": {
                    "summary": "Create embeddings",
                    "description": "OpenAI-compatible embeddings from the runtime's configured embedding backend. `model` must name the served model (`<backend>/<model>` or `<model>`); `input` is a string or an array of up to 2048 non-empty strings; `dimensions`, when present, must equal the served dimension; `encoding_format` is `float` (default) or `base64`.",
                    "tags": ["models"],
                    "responses": {
                        "200": { "description": "Embedding list: `data[].embedding` in input order, `model`, `usage`" },
                        "400": { "description": "Invalid input, dimensions or encoding format" },
                        "404": { "description": "The requested model is not the served embedding model" },
                        "502": { "description": "The embedding provider failed or answered malformed data" },
                        "503": { "description": "No embedding backend is available" }
                    }
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
                    "description": "Creates a governed run from exactly one registered agent id or inline compatibility artifact and returns its identifier and event stream URL",
                    "tags": ["runs"],
                    "requestBody": {
                        "required": true,
                        "content": {"application/json": {"schema": {
                            "type": "object",
                            "required": ["input"],
                            "oneOf": [
                                {"required": ["agent_id"], "not": {"required": ["artifact"]}},
                                {"required": ["artifact"], "not": {"required": ["agent_id"]}}
                            ],
                            "properties": {
                                "agent_id": {"type": "string", "description": "Explicit registered agent id resolved from the UAR catalog"},
                                "artifact": {"type": "object", "description": "Explicit inline compatibility artifact defining the run policy and prompt"},
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
            "/api/uar/runs/{id}/events": {
                "get": {
                    "summary": "Read owner-scoped run event snapshot",
                    "description": "Read existing bounded process-local public SSE projections without subscribing or cancelling. No durable replay guarantee. gapReason reports incomplete retention or a cursor ahead of this snapshot.",
                    "tags": ["runs"],
                    "security": [{"bearerAuth": []}],
                    "parameters": [
                        {"name": "id", "in": "path", "required": true, "schema": {"type": "string"}},
                        {"name": "after", "in": "query", "required": false, "schema": {"type": "integer", "minimum": 0, "default": 0}, "description": "Exclusive event cursor"}
                    ],
                    "responses": {
                        "200": {"description": "Versioned bounded public event snapshot", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/RunEventSnapshot"}}}},
                        "400": {"description": "Invalid query cursor"},
                        "404": {"description": "Run/history unavailable or outside current owner scope"}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks": {
                "post": {
                    "summary": "Admit a complete delegated run",
                    "description": "Reserves owner-and-workspace-scoped admission, task, and native run identities before entering UAR's sole execution loop. Exact retries return the existing process-local receipt; this profile does not claim restart recovery.",
                    "tags": ["full-harness"],
                    "security": [{"bearerAuth": []}],
                    "parameters": [workspace_id.clone()],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessAdmissionRequest"}}}},
                    "responses": {
                        "202": {"description": "Task admitted or exact accepted admission replayed", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessTaskReceipt"}}}},
                        "400": {"description": "Invalid admission request or native admission refusal", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "401": {"description": "Verified principal required", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "409": {"description": "Admission ID reused with a different canonical request", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/capabilities": {
                "get": {
                    "summary": "Describe the process-ephemeral full-harness profile",
                    "description": "Returns the current runtime epoch and explicit unsupported-after-restart recovery posture so clients can distinguish restart loss from an unknown task.",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "responses": {
                        "200": {"description": "Current full-harness runtime descriptor", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessRuntimeDescriptor"}}}},
                        "401": {"description": "Verified principal required", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/admissions/{admission_id}": {
                "get": {
                    "summary": "Reconcile an admission",
                    "description": "Returns the owner-scoped process-local receipt without creating or replaying a run.",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [admission_id, workspace_id.clone()],
                    "responses": {
                        "200": {"description": "Authoritative admission receipt", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessTaskReceipt"}}}},
                        "404": {"description": "No current process-local admission record", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Advertised retention expired", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks/{task_id}": {
                "get": {
                    "summary": "Get delegated task status",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [task_id.clone(), workspace_id.clone()],
                    "responses": {
                        "200": {"description": "Authoritative process-local task receipt", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessTaskReceipt"}}}},
                        "404": {"description": "Task ID is outside the full-harness authority", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "409": {"description": "Prior runtime epoch with unsupported recovery", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Expired record or unresolved current-epoch task", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks/{task_id}/stream": {
                "get": {
                    "summary": "Observe delegated run events without owning its lifetime",
                    "description": "Replays retained native events and follows the live stream. Disconnecting detaches the observer and does not cancel the run.",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [
                        task_id.clone(),
                        workspace_id.clone(),
                        {"name": "last_event_id", "in": "query", "required": false, "schema": {"type": "integer", "format": "uint64"}},
                        {"name": "Last-Event-ID", "in": "header", "required": false, "schema": {"type": "integer", "format": "uint64"}, "description": "Used when last_event_id is omitted"}
                    ],
                    "responses": {
                        "200": {"description": "Normalized native run event stream", "content": {"text/event-stream": {"schema": {"type": "string"}}}},
                        "409": {"description": "Prior runtime epoch with unsupported recovery", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Task or native run stream is unavailable", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks/{task_id}/tool-approval": {
                "post": {
                    "summary": "Resolve the current native tool approval",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [task_id.clone(), workspace_id.clone()],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessApprovalRequest"}}}},
                    "responses": {
                        "200": {"description": "Updated receipt after approval forwarding", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessTaskReceipt"}}}},
                        "409": {"description": "Revision conflict, unresolved approval, or prior runtime epoch", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "404": {"description": "Task not found", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Task retention expired or native task is unresolved", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks/{task_id}/cancel": {
                "post": {
                    "summary": "Request cancellation of the native run",
                    "description": "The receipt distinguishes request, executor acknowledgement, terminal cancellation, and cleanup uncertainty.",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [task_id.clone(), workspace_id.clone()],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessMutationRequest"}}}},
                    "responses": {
                        "200": {"description": "Updated cancellation receipt; acknowledgement is not terminal completion", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessTaskReceipt"}}}},
                        "409": {"description": "Revision conflict or prior runtime epoch", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "404": {"description": "Task not found", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Task retention expired or unresolved", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks/{task_id}/detach": {
                "post": {
                    "summary": "Detach from a delegated task",
                    "description": "Records detachment without requesting cancellation.",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [task_id.clone(), workspace_id.clone()],
                    "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessDetachRequest"}}}},
                    "responses": {
                        "200": {"description": "Updated receipt with detached set", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessTaskReceipt"}}}},
                        "400": {"description": "observer_id is empty", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "409": {"description": "Revision conflict or prior runtime epoch", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "404": {"description": "Task not found", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Task retention expired or unresolved", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
                }
            },
            "/api/uar/full-harness/v1/tasks/{task_id}/steer": {
                "post": {
                    "summary": "Steer a delegated task",
                    "description": "The process-ephemeral v1 profile explicitly refuses steering and never creates a replacement run.",
                    "tags": ["full-harness"], "security": [{"bearerAuth": []}],
                    "parameters": [task_id, workspace_id],
                    "responses": {
                        "404": {"description": "Task not found", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "409": {"description": "Prior runtime epoch with unsupported recovery", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "410": {"description": "Task retention expired or unresolved", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}},
                        "422": {"description": "capability_unsupported", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/FullHarnessError"}}}}
                    }
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
            "/api/uar/skills/deployment-catalog": {
                "get": {
                    "summary": "Read trusted installed skill deployment metadata",
                    "description": "Admin Read only. Requires an authenticated nonanonymous principal and the configured x-uar-admin-key. Returns current registry identities including tombstones; portable SkillRef is separate from private installedLocation. required=true/config={} are authoring defaults. Discovery grants no binding or tool authority. Also mounted under /api/skills/deployment-catalog.",
                    "tags": ["skills"],
                    "security": [{"bearerAuth": [], "uarAdminKey": []}],
                    "responses": {
                        "200": {"description": "Stored deployment catalog", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/SkillDeploymentCatalog"}}}},
                        "401": {"description": "Authenticated nonanonymous principal required"},
                        "403": {"description": "Configured admin key required; no optional-auth bypass"}
                    }
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
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {"type": "http", "scheme": "bearer", "bearerFormat": "JWT"},
                "uarAdminKey": {"type": "apiKey", "in": "header", "name": "x-uar-admin-key"}
            },
            "schemas": {
                "SkillDeploymentCatalog": {
                    "type": "object", "additionalProperties": false,
                    "required": ["schemaVersion", "entries"],
                    "properties": {
                        "schemaVersion": {"type": "integer", "enum": [1]},
                        "entries": {"type": "array", "items": {"$ref": "#/components/schemas/SkillDeploymentCatalogEntry"}}
                    }
                },
                "SkillDeploymentCatalogEntry": {
                    "type": "object", "additionalProperties": false,
                    "required": ["skillId", "title", "description", "enabled", "tombstoned", "availability", "reasons", "skillRef", "privateBinding"],
                    "properties": {
                        "skillId": {"type": "string"}, "title": {"type": "string"}, "description": {"type": "string"},
                        "enabled": {"type": "boolean"}, "tombstoned": {"type": "boolean"},
                        "availability": {"type": "string", "enum": ["available", "unavailable"]},
                        "reasons": {"type": "array", "items": {"type": "string", "enum": ["missing-skill-id", "missing-version", "missing-artifact-digest", "missing-installed-location", "disabled", "tombstoned"]}},
                        "skillRef": {"nullable": true, "allOf": [{"$ref": "#/components/schemas/DeploymentCatalogSkillRef"}]},
                        "privateBinding": {"nullable": true, "allOf": [{"$ref": "#/components/schemas/SkillInstallationBinding"}]}
                    }
                },
                "DeploymentCatalogSkillRef": {
                    "type": "object", "additionalProperties": false,
                    "description": "Portable domain SkillRef. required=true/config={} are explicit authoring defaults; callers may author them. Artifact identity, entrypoint and requiredTools are stored values.",
                    "required": ["id", "version", "digest", "required", "config", "entrypoint", "requiredTools"],
                    "properties": {
                        "id": {"type": "string"}, "version": {"type": "string"}, "digest": {"type": "string"},
                        "required": {"type": "boolean"}, "config": {"type": "object"},
                        "entrypoint": {"type": "string", "nullable": true},
                        "requiredTools": {"type": "array", "items": {"type": "string"}}
                    }
                },
                "SkillInstallationBinding": {
                    "type": "object", "additionalProperties": false,
                    "description": "Private deployment metadata; must never be included in portable definitions or renderer display.",
                    "required": ["installedLocation"],
                    "properties": {"installedLocation": {"type": "string"}}
                },
                "RunEventSnapshot": super::run_events::snapshot_schema(),
                "FullHarnessAdmissionRequest": {
                    "type": "object",
                    "required": ["admission_id", "native_task_id", "input"],
                    "oneOf": [
                        {"required": ["deployment_binding_id"], "not": {"anyOf": [{"required": ["agent_id"]}, {"required": ["artifact"]}]}},
                        {"required": ["agent_id"], "not": {"anyOf": [{"required": ["deployment_binding_id"]}, {"required": ["artifact"]}]}},
                        {"required": ["artifact"], "not": {"anyOf": [{"required": ["deployment_binding_id"]}, {"required": ["agent_id"]}]}}
                    ],
                    "properties": {
                        "admission_id": {"type": "string", "minLength": 1, "description": "Owner-scoped idempotency identity"},
                        "native_task_id": {"type": "string", "minLength": 1, "description": "Caller correlation identity; not UAR execution authority"},
                        "artifact": {"type": ["object", "null"]}, "agent_id": {"type": ["string", "null"]},
                        "deployment_binding_id": {"type": ["string", "null"]}, "service_placement": {"type": ["object", "null"]},
                        "input": {"type": "string"}, "session_id": {"type": ["string", "null"]},
                        "run_credentials": {"type": ["array", "null"], "items": {"type": "object"}},
                        "mcp_servers": {"type": ["array", "null"], "items": {"type": "object"}},
                        "tool_admission": {"type": ["object", "null"]}, "working_directory": {"type": ["string", "null"]},
                        "reasoning_effort": {"type": ["string", "null"]}, "history": {"type": ["object", "null"]},
                        "skill_attachments": {"type": "array", "items": {"type": "string"}, "default": []},
                        "presentation_mode": {"type": ["string", "null"], "enum": ["auto", "text", "a2ui", "hybrid", null]},
                        "client_rendering": {"type": ["object", "null"], "properties": {"a2ui_profiles": {"type": "array", "items": {"type": "string"}}}}
                    },
                    "description": "UAR computes the canonical digest from the complete deserialized request plus normalized workspace identity and never returns or persists request credential material in the receipt."
                },
                "FullHarnessRetention": {
                    "type": "object", "required": ["mode", "terminal_ttl_seconds", "terminal_record_cap"],
                    "properties": {"mode": {"type": "string", "enum": ["process_ephemeral"]}, "terminal_ttl_seconds": {"type": "integer", "format": "uint64"}, "terminal_record_cap": {"type": "integer", "minimum": 0}}
                },
                "FullHarnessRuntimeDescriptor": {
                    "type": "object", "required": ["profile", "runtime_epoch", "recovery", "retention", "steer_supported"],
                    "properties": {
                        "profile": {"type": "string", "enum": ["full_harness_v1"]},
                        "runtime_epoch": {"type": "string"},
                        "recovery": {"type": "string", "enum": ["unsupported_after_restart"]},
                        "retention": {"$ref": "#/components/schemas/FullHarnessRetention"},
                        "steer_supported": {"type": "boolean", "const": false}
                    }
                },
                "FullHarnessCancellation": {
                    "type": "object", "required": ["requested", "acknowledged", "terminal", "cleanup_uncertain"],
                    "properties": {"requested": {"type": "boolean"}, "acknowledged": {"type": "boolean"}, "terminal": {"type": "boolean"}, "cleanup_uncertain": {"type": "boolean"}}
                },
                "FullHarnessMutationRequest": {
                    "type": "object", "required": ["expected_revision"],
                    "properties": {"expected_revision": {"type": "integer", "format": "uint64"}}
                },
                "FullHarnessApprovalRequest": {
                    "type": "object", "required": ["expected_revision", "approved", "approval_id"],
                    "properties": {"expected_revision": {"type": "integer", "format": "uint64"}, "approved": {"type": "boolean"}, "approval_id": {"type": "string", "minLength": 1}}
                },
                "FullHarnessDetachRequest": {
                    "type": "object", "required": ["expected_revision", "observer_id"],
                    "properties": {"expected_revision": {"type": "integer", "format": "uint64"}, "observer_id": {"type": "string", "minLength": 1}}
                },
                "FullHarnessTaskReceipt": {
                    "type": "object",
                    "required": ["admission_id", "task_id", "native_task_id", "run_id", "workspace_id", "runtime_epoch", "revision", "state", "retention", "diagnostics", "cancellation", "detached", "detached_observers", "created_at", "unsupported_semantics", "links"],
                    "properties": {
                        "admission_id": {"type": "string"}, "task_id": {"type": "string", "pattern": "^fh-"}, "native_task_id": {"type": "string"},
                        "run_id": {"type": "string"}, "agent_id": {"type": ["string", "null"]}, "workspace_id": {"type": "string"}, "runtime_epoch": {"type": "string"},
                        "revision": {"type": "integer", "format": "uint64"}, "cursor": {"type": ["integer", "null"], "format": "uint64"},
                        "state": {"type": "string", "enum": ["reserved", "submitted", "working", "input_required", "completed", "failed", "cancelled", "rejected"]},
                        "retention": {"$ref": "#/components/schemas/FullHarnessRetention"},
                        "effective_service_binding": {"type": ["object", "null"]},
                        "diagnostics": {"type": "array", "items": {"type": "object", "required": ["code", "message"], "properties": {"code": {"type": "string"}, "message": {"type": "string"}}}},
                        "cancellation": {"$ref": "#/components/schemas/FullHarnessCancellation"}, "detached": {"type": "boolean"},
                        "detached_observers": {"type": "array", "items": {"type": "string"}},
                        "created_at": {"type": "string", "format": "date-time"}, "terminal_at": {"type": ["string", "null"], "format": "date-time"}, "expires_at": {"type": ["string", "null"], "format": "date-time"},
                        "unsupported_semantics": {"type": "array", "items": {"type": "string"}},
                        "links": {"type": "object", "required": ["status", "stream", "tool_approval", "cancel", "detach", "steer"], "properties": {"status": {"type": "string"}, "stream": {"type": "string"}, "tool_approval": {"type": "string"}, "cancel": {"type": "string"}, "detach": {"type": "string"}, "steer": {"type": "string"}}}
                    }
                },
                "FullHarnessError": {
                    "type": "object", "required": ["error"],
                    "properties": {"error": {"type": "object", "required": ["code", "message"], "properties": {
                        "code": {"type": "string", "description": "Stable full-harness or native run-admission error code, including admission_invalid, admission_digest_conflict, workspace_required, recovery_unsupported, retention_expired, task_unresolved, task_not_found, revision_conflict, detach_invalid, approval_invalid, approval_unresolved, capability_unsupported, and principal_invalid"},
                        "message": {"type": "string"}, "task_id": {"type": ["string", "null"]}, "admission_id": {"type": ["string", "null"]}
                    }}}
                }
            }
        }
    });
    extend_current_contracts(&mut spec);
    super::delegation_grants::extend_openapi(&mut spec);
    serde_json::from_value(spec).expect("OpenAPI spec JSON is valid")
}

/// Add source-authored contracts without replacing the existing route inventory.
/// These schemas describe wire availability and recovery limits, not acceptance.
fn extend_current_contracts(spec: &mut serde_json::Value) {
    use serde_json::json;
    // Utoipa deserializes a null `type` as AnyValue and omits it on the wire.
    let json_value = json!({"type": null});
    let schemas = json!({
        "UsageCounterAvailability": {
            "type":"object", "required":["available","source"],
            "properties": {
                "available":{"type":"boolean","description":"False means a compatible numeric zero is an unavailable placeholder, not measured or billed zero"},
                "source":{"type":"string","enum":["provider_reported","unavailable"]}
            }
        },
        "EmbeddingResponse": {
            "type":"object", "required":["object","data","model","usage","uar_usage"],
            "properties": {
                "object":{"type":"string","enum":["list"]},
                "model":{"type":"string"},
                "data":{"type":"array","items":{"type":"object","required":["object","index","embedding"],"properties":{
                    "object":{"type":"string","enum":["embedding"]},
                    "index":{"type":"integer","minimum":0},
                    "embedding":{"oneOf":[{"type":"array","items":{"type":"number"}},{"type":"string","description":"base64-encoded float32 values"}]}
                }}},
                "usage":{"type":"object","required":["prompt_tokens","total_tokens"],"properties":{
                    "prompt_tokens":{"type":"integer","minimum":0},"total_tokens":{"type":"integer","minimum":0}
                }},
                "uar_usage":{"type":"object","required":["prompt_tokens","total_tokens","billed_input_tokens"],"properties":{
                    "prompt_tokens":{"$ref":"#/components/schemas/UsageCounterAvailability"},
                    "total_tokens":{"$ref":"#/components/schemas/UsageCounterAvailability"},
                    "billed_input_tokens":{"type":"object","required":["value","available","source"],"properties":{
                        "value":{"type":["integer","null"],"minimum":0},
                        "available":{"type":"boolean"},
                        "source":{"type":"string","enum":["provider_billed_units","unavailable"]}
                    },"description":"Independent native billing counter; never copied into prompt_tokens or total_tokens"}
                }}
            }
        },
        "AnthropicUsage": {
            "type":"object","required":["input_tokens","output_tokens"],
            "description":"Provider-reported uncached input and output. Input excludes valid reported cache reads/creation; unavailable counters retain compatible zero with uar_usage availability false. Optional cache counters are omitted when unavailable or invalid.",
            "properties":{
                "input_tokens":{"type":"integer","minimum":0},"output_tokens":{"type":"integer","minimum":0},
                "cache_read_input_tokens":{"type":"integer","minimum":0},"cache_creation_input_tokens":{"type":"integer","minimum":0}
            }
        },
        "AnthropicUsageAvailability": {
            "type":"object","required":["input_tokens","output_tokens"],"properties":{
                "input_tokens":{"$ref":"#/components/schemas/UsageCounterAvailability"},
                "output_tokens":{"$ref":"#/components/schemas/UsageCounterAvailability"}
            }
        },
        "AnthropicMessage": {
            "type":"object","required":["id","type","role","model","content","stop_reason","stop_sequence","usage","uar_usage"],
            "properties":{
                "id":{"type":"string"},"type":{"type":"string","enum":["message"]},"role":{"type":"string","enum":["assistant"]},
                "model":{"type":"string"},"content":{"type":"array","items":{"type":"object"}},
                "stop_reason":{"type":["string","null"]},"stop_sequence":{"type":["string","null"]},
                "usage":{"$ref":"#/components/schemas/AnthropicUsage"},
                "uar_usage":{"$ref":"#/components/schemas/AnthropicUsageAvailability"}
            }
        },
        "NativeChatTurnReceipt": {
            "type":"object","required":["retry_allowed"],
            "description":"Read-only original reservation. Unknown or admission_unconfirmed never grants another execution. Nonterminal durable run without a resident lane is interrupted.",
            "properties":{
                "client_turn_id":{"type":"string"},"user_message_id":{"type":["string","null"]},
                "session_id":{"type":"string"},"run_id":{"type":"string"},
                "state":{"type":"string","enum":["unknown","pending","running","paused","complete","error","cancelled","interrupted","admission_unconfirmed"]},
                "recovery":{"type":"string","enum":["available","unavailable"]},"retry_allowed":{"type":"boolean","enum":[false]}
            }
        },
        "MemoryContextObservation": {
            "type":"object","required":["status","reason_code"],
            "description":"Content-free /memory_context state-patch observation. selected_prepared is not submission. submitted requires selected fragment in final driver request plus observed provider response, not a nonempty retrieval hit list or proof of model understanding. Config memory.injection_min_similarity must be finite in [-1,1]; absent cutoff refuses automatic injection.",
            "properties":{
                "status":{"type":"string","enum":["disabled","opt_out","no_match","bounded_out","unavailable","refused","selected_prepared","submitted"]},
                "reason_code":{"type":"string"},"retrieval_method":{"type":"string"},
                "memory_ids":{"type":"array","items":{"type":"string"}},
                "evidence_basis":{"type":"string","enum":["final_driver_request_and_provider_response"]},
                "request_content_sha256":{"type":"string"},"model":{"type":"string"}
            }
        },
        "TeamAttemptReconciliation": {
            "type":"object","required":["schemaVersion","attemptId","runId","rootId","executionEpoch","ownershipEpoch","executionOutcome","accountingState","effectDisposition","usageRevision","teamReservation","teamUsage","teamCapacityRetained","accountingInspection","automaticTeamProjection","effectReplayAllowed","procedure","disposition"],
            "description":"Owner/workspace-qualified read. Provider call settlement does not release a separate team reservation. No automatic replay or team usage projection; preserve retained capacity while original evidence is unresolved.",
            "properties":{
                "schemaVersion":{"type":"integer","enum":[1]},"attemptId":{"type":"string"},"runId":{"type":"string"},"rootId":{"type":"string"},
                "executionEpoch":{"type":"integer","minimum":0},"ownershipEpoch":{"type":"integer","minimum":0},
                "executionOutcome":{"type":["string","null"]},"accountingState":{"type":"string"},"effectDisposition":{"type":"string"},
                "usageRevision":{"type":"integer","minimum":0},"teamReservation":{"type":"object"},"teamUsage":{"type":["object","null"]},
                "teamCapacityRetained":{"type":"boolean"},"automaticTeamProjection":{"type":"boolean","enum":[false]},"effectReplayAllowed":{"type":"boolean","enum":[false]},
                "accountingInspection":{"type":"object","properties":{"pathTemplate":{"type":"string"},"runId":{"type":"string"},"callIdentitySource":{"type":"string"},"settlementAuthority":{"type":"string"}}},
                "procedure":{"type":"string"},"disposition":{"type":"string","enum":["manual_evidence_required_team_capacity_retained","team_usage_settled"]}
            }
        },
        "A2uiAction": {
            "type":"object","required":["surfaceId","name","sourceComponentId"],
            "description":"Existing camelCase fields; interactionId and surfaceGeneration also accept snake_case aliases. Protected actions require both logical ID and string generation. Reuse original immutable payload/ID/generation for uncertain intent; a new click gets a new ID. Legacy absent-ID requests explicitly lack retry protection. Existing host-resource/approval/policy requirements remain.",
            "properties":{
                "surfaceId":{"type":"string"},"name":{"type":"string"},"sourceComponentId":{"type":"string"},
                "interactionId":{"type":["string","null"]},"surfaceGeneration":{"type":["string","null"]},
                "interaction_id":{"type":["string","null"],"description":"Alias of interactionId"},
                "surface_generation":{"type":["string","null"],"description":"Alias of surfaceGeneration"},
                "timestamp":{"type":["string","null"]},"context":json_value.clone(),"a2uiClientDataModel":json_value
            }
        },
        "A2uiContinuationReceipt": {
            "type":"object","required":["run_id","continuation_run_id","status","interaction_id","retry_protected","outcome"],
            "properties":{
                "run_id":{"type":"string"},"continuation_run_id":{"type":"string"},"status":{"type":"string","enum":["continued"]},
                "interaction_id":{"type":["string","null"]},"retry_protected":{"type":"boolean"},
                "outcome":{"type":"string","enum":["running","completed","failed","cancelled","uncertain"]}
            },"description":"Acknowledgement is not proof of effect settlement. A protected receipt never permits another dispatch, including when its reserved run is absent."
        },
        "A2uiAguiReceipt": {
            "type":"object","required":["interaction_id","continuation_run_id","wire_run_id","thread_id","retry_protected","outcome"],
            "properties":{
                "interaction_id":{"type":"string"},"continuation_run_id":{"type":"string"},"wire_run_id":{"type":"string"},"thread_id":{"type":"string"},
                "retry_protected":{"type":"boolean","enum":[true]},"outcome":{"type":"string","enum":["running","completed","failed","cancelled","uncertain"]}
            }
        },
        "A2uiSurfaceAuthority": {
            "type":"object","required":["status","authoritative","max_prior_runs","max_surfaces","recovery_hint","surfaces"],
            "properties":{
                "status":{"type":"string","enum":["complete","truncated","expired","unavailable"]},"authoritative":{"type":"boolean"},
                "max_prior_runs":{"type":"integer","enum":[64]},"max_surfaces":{"type":"integer","enum":[48]},"recovery_hint":{"type":"string"},
                "surfaces":{"type":"array","items":{"type":"object","required":["surface_id","surface_generation","source_run_id"],"properties":{
                    "surface_id":{"type":"string"},"surface_generation":{"type":"string"},"source_run_id":{"type":"string"}
                }}}
            },"description":"Only complete authoritative=true can authorize a fresh declared action. Partial, expired or unavailable state cannot. Reconcile uncertain intent before opening a fresh conversation/surface."
        }
    });
    if let (Some(target), Some(additions)) = (
        spec["components"]["schemas"].as_object_mut(),
        schemas.as_object(),
    ) {
        target.extend(additions.clone());
    }
    spec["paths"]["/v1/embeddings"]["post"]["responses"]["200"] = json!({
        "description":"Input-ordered vectors and independent provider counter availability; compatible zero placeholders are not billed zero. See docs/operations/embedding-usage-accounting.md.",
        "content":{"application/json":{"schema":{"$ref":"#/components/schemas/EmbeddingResponse"}}}
    });
    spec["paths"]["/v1/chat/completions"]["post"]["description"] = json!(
        "OpenAI-compatible completion with existing stream modes and tools. Optional native recovery requires client_turn_id and user_message_id together plus a stable resolved session. The owner/session/workspace-qualified semantic digest covers original rich content, policy, attachments and stream format. Only a newly reserved intent dispatches its reserved run; exact retry returns the original JSON turn receipt, and changed semantics conflict. Caller-owned tool-batch continuations use their separate durable identity and refuse native turn IDs. Requests without IDs retain legacy behavior without durable turn recovery. Source-authored, current-tree runtime qualification unverified; see docs/operations/chat-turn-recovery.md."
    );
    spec["paths"]["/metrics"]["get"]["description"] = json!(
        "Own-recorder Prometheus export only. Host-owned recorder returns explicit 503 rather than healthy empty scrape; compiled-out telemetry returns 404. HTTP labels use matched templates/unmatched sentinel, never resource IDs. See docs/operations/telemetry-ownership.md; no hard exporter-flush promise."
    );
    spec["paths"]["/metrics"]["get"]["responses"]["503"] = json!({"description":"telemetry_scrape_unavailable with metrics_status=host_owned or unavailable; use host exporter when host-owned"});
    spec["paths"]["/metrics"]["get"]["responses"]["404"] =
        json!({"description":"Telemetry capability is not compiled"});
    let paths = json!({
        "/v1/messages":{"post":{
            "summary":"Anthropic-compatible agent Messages", "tags":["chat"],"security":[{"bearerAuth":[]}],
            "description":"Existing agent-mode Messages route, verified owner required. uar_usage is a top-level sibling of usage on the nonstream message and streamed message_start.message/message_delta. Input/output availability is independent; cache fields are optional. Provider errors/cancellation do not become successful zero-usage replies. See docs/ANTHROPIC_MESSAGES.md; authentication is externally provisioned.",
            "responses":{
                "200":{"description":"Anthropic message or SSE; initial streamed availability may be false until actual provider usage arrives", "content":{
                    "application/json":{"schema":{"$ref":"#/components/schemas/AnthropicMessage"}},"text/event-stream":{"schema":{"type":"string"}}
                }},"400":{"description":"Unsupported or contradictory request fields"},"401":{"description":"Verified owner required"},"422":{"description":"Model/provider refused"},"502":{"description":"Started nonstream execution failed or cancelled; streamed execution emits error event"}
            }
        }},
        "/api/uar/chat-turns/{client_turn_id}":{"get":{
            "summary":"Read original native chat turn admission", "tags":["chat"],"security":[{"bearerAuth":[]}],
            "description":"Owner/session-qualified read without execution. Stable client_turn_id and original user_message_id identify immutable intent. Unknown or absent execution never permits automatic resend. Durable capability required; no anonymous fallback. Ordinary native POST now reserves paired client_turn_id/user_message_id under a stable resolved session before dispatch: only a new reservation executes its original run ID; an existing reservation returns this original JSON receipt. Changed semantic input conflicts, and uncertain admission never permits resend. Caller-owned tool-batch continuations keep their separate identity and refuse these IDs. Source-authored; current-tree runtime qualification remains unverified.",
            "parameters":[{"name":"client_turn_id","in":"path","required":true,"schema":{"type":"string","minLength":1,"maxLength":512}},{"name":"session_id","in":"query","required":true,"schema":{"type":"string","minLength":1,"maxLength":512}}],
            "responses":{
                "200":{"description":"Original owned reservation or unknown with retry_allowed=false","content":{"application/json":{"schema":{"$ref":"#/components/schemas/NativeChatTurnReceipt"}}}},
                "400":{"description":"Invalid native chat identity"},"401":{"description":"Verified principal required"},"409":{"description":"Native turn binding conflict"},"503":{"description":"Durable admission recovery unavailable"}
            }
        }},
        "/api/v1/collaboration/team-instances/{id}/attempts/{attempt}/reconciliation":{"get":{
            "summary":"Read retained team attempt accounting/effect disposition", "tags":["runs"],"security":[{"bearerAuth":[]}],
            "description":"Surreal team execution capability only. Owner/workspace-qualified original attempt/run correlation; no dispatch, billing attestation or team reservation release. Generic confirmed call settlement may leave teamCapacityRetained=true and automaticTeamProjection=false. See docs/operations/team-control-reconciliation.md.",
            "parameters":[{"name":"id","in":"path","required":true,"schema":{"type":"string"}},{"name":"attempt","in":"path","required":true,"schema":{"type":"string"}},{"name":"x-uar-workspace-id","in":"header","required":true,"schema":{"type":"string","minLength":1}}],
            "responses":{"200":{"description":"Original attempt correlation and retained reservation disposition","content":{"application/json":{"schema":{"$ref":"#/components/schemas/TeamAttemptReconciliation"}}}},"401":{"description":"Verified owner required"},"403":{"description":"Owner/workspace authority refused"},"404":{"description":"Attempt unavailable in owner/workspace scope"},"409":{"description":"Current revision/ownership conflict"},"422":{"description":"Invalid team/workspace request"},"503":{"description":"Team capability unsupported or catalog storage unavailable"}}
        }},
        "/api/uar/runs/{run_id}/a2ui/actions":{"post":{
            "summary":"Submit declared A2UI action with optional protected intent", "tags":["runs"],"security":[{"bearerAuth":[]}],
            "description":"Protected logical ID/generation is durably reserved before native continuation. Exact retry returns original receipt; changed semantics conflict. Existing latest declaration, policy, approval and host-resource checks remain. Single owned host only, no distributed multiwriter ordering or global exactly-once claim.",
            "parameters":[{"name":"run_id","in":"path","required":true,"schema":{"type":"string"}}],
            "requestBody":{"required":true,"content":{"application/json":{"schema":{"$ref":"#/components/schemas/A2uiAction"}}}},
            "responses":{"202":{"description":"Continuation acknowledgement, protected or explicit legacy unprotected","content":{"application/json":{"schema":{"$ref":"#/components/schemas/A2uiContinuationReceipt"}}}},"401":{"description":"Verified owner required for protected action"},"403":{"description":"Current authority denied"},"404":{"description":"Source unavailable to owner"},"409":{"description":"Intent/generation/authority conflict"},"422":{"description":"Invalid or undeclared action"},"503":{"description":"Durable action recovery unavailable"}}
        }},
        "/api/uar/runs/{run_id}/a2ui/actions/{interaction_id}":{"get":{
            "summary":"Read original A2UI intent receipt", "tags":["runs"],"security":[{"bearerAuth":[]}],
            "description":"Read-only owner-qualified recovery. Missing receipt or uncertain continuation is not permission to re-execute.",
            "parameters":[{"name":"run_id","in":"path","required":true,"schema":{"type":"string"}},{"name":"interaction_id","in":"path","required":true,"schema":{"type":"string"}}],
            "responses":{"202":{"description":"Original continuation receipt","content":{"application/json":{"schema":{"$ref":"#/components/schemas/A2uiContinuationReceipt"}}}},"404":{"description":"Owned interaction unavailable"},"503":{"description":"Action recovery unavailable"}}
        }},
        "/api/uar/runs/{run_id}/a2ui/surface-authority":{"get":{
            "summary":"Read bounded authoritative surface generations", "tags":["runs"],"security":[{"bearerAuth":[]}],
            "parameters":[{"name":"run_id","in":"path","required":true,"schema":{"type":"string"}}],
            "responses":{"200":{"description":"Explicit complete/truncated/expired/unavailable authority, never inferred from nonempty surfaces","content":{"application/json":{"schema":{"$ref":"#/components/schemas/A2uiSurfaceAuthority"}}}},"404":{"description":"Source unavailable to owner"}}
        }},
        "/api/uar/ag-ui/agents/{agent_id}/threads/{thread_id}/actions/{interaction_id}":{"get":{
            "summary":"Read protected AG-UI A2UI intent receipt", "tags":["runs"],"security":[{"bearerAuth":[]}],
            "description":"Owner/workspace-qualified original wire/native continuation identity. forwardedProps.a2uiAction carries interaction_id and string surface_generation. CUSTOM uar.a2ui.retry_protection and uar.a2ui.surface_authority disclose protection/authority; ACTIVITY_SNAPSHOT adds surface_generation. No receipt read dispatches work.",
            "parameters":[{"name":"agent_id","in":"path","required":true,"schema":{"type":"string"}},{"name":"thread_id","in":"path","required":true,"schema":{"type":"string"}},{"name":"interaction_id","in":"path","required":true,"schema":{"type":"string"}},{"name":"x-uar-workspace-id","in":"header","required":false,"schema":{"type":"string"}}],
            "responses":{"200":{"description":"Original protected AG-UI receipt","content":{"application/json":{"schema":{"$ref":"#/components/schemas/A2uiAguiReceipt"}}}},"400":{"description":"Invalid identity"},"401":{"description":"Verified principal required"},"403":{"description":"Current authority refused"},"404":{"description":"Owned interaction unavailable"},"409":{"description":"Original receipt belongs to another protocol"},"503":{"description":"Durable recovery unavailable"}}
        }}
    });
    if let (Some(target), Some(additions)) = (spec["paths"].as_object_mut(), paths.as_object()) {
        target.extend(additions.clone());
    }
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
            "/v1/embeddings",
            "/api/uar/runs",
            "/api/uar/full-harness/v1/tasks",
            "/api/uar/full-harness/v1/capabilities",
            "/api/uar/full-harness/v1/admissions/{admission_id}",
            "/api/uar/full-harness/v1/tasks/{task_id}",
            "/api/uar/full-harness/v1/tasks/{task_id}/stream",
            "/api/uar/full-harness/v1/tasks/{task_id}/tool-approval",
            "/api/uar/full-harness/v1/tasks/{task_id}/cancel",
            "/api/uar/full-harness/v1/tasks/{task_id}/detach",
            "/api/uar/full-harness/v1/tasks/{task_id}/steer",
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
        let action = &spec["components"]["schemas"]["A2uiAction"]["properties"];
        for field in ["context", "a2uiClientDataModel"] {
            assert_eq!(
                action[field],
                serde_json::json!({}),
                "A2UI {field} must document arbitrary JSON values"
            );
        }
        let event_data = &spec["components"]["schemas"]["RunEventSnapshot"]["properties"]
            ["events"]["items"]["properties"]["data"];
        assert_eq!(
            event_data,
            &serde_json::json!({"description": "Existing public payload for eventName; no internal admission or credential objects"})
        );
    }
}
