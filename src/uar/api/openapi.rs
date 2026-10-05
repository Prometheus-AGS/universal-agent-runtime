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
                        {"name": "after", "in": "query", "schema": {"type": "integer", "minimum": 0, "default": 0}, "description": "Exclusive event cursor"}
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
                "bearerAuth": {"type": "http", "scheme": "bearer", "bearerFormat": "JWT"}
            },
            "schemas": {
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
    super::delegation_grants::extend_openapi(&mut spec);
    serde_json::from_value(spec)
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
    }
}
