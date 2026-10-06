//! Additive OpenAPI contract for the managed-local authentication boundary.

use serde_json::{Value, json};

pub(crate) fn extend_openapi(spec: &mut Value) {
    spec["paths"]["/api/uar/delegation-grants"] = json!({
        "post": {
            "summary": "Issue a scoped process-local delegation grant",
            "description": "Managed local sidecar only. Requires the original launch credential and an approved x-uar-principal. Captures verified principal; cannot be called with a grant or an external JWT. Returns the secret once with Cache-Control: no-store. Grants never become host-authenticated.",
            "tags": ["auth"], "security": [{"sidecarHostAuth": []}],
            "parameters": [{"name": "x-uar-principal", "in": "header", "required": true, "schema": {"type": "string"}}],
            "requestBody": {"required": true, "content": {"application/json": {"schema": {"$ref": "#/components/schemas/DelegationGrantRequest"}}}},
            "responses": {
                "201": {"description": "Fresh 256-bit grant; 15 minute maximum lifetime", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/DelegationGrantReceipt"}}}},
                "400": {"description": "Missing approved principal, empty or invalid scope"},
                "401": {"description": "Invalid launch credential or non-authorized grant"},
                "403": {"description": "Origin, authority, or host-only authentication refused"},
                "404": {"description": "Grant API unavailable on external JWT transport"}
            }
        }
    });
    spec["paths"]["/api/uar/delegation-grants/{id}"] = json!({
        "delete": {
            "summary": "Revoke a process-local delegation grant",
            "description": "Original launching host only. Idempotent; unknown or already revoked ID returns 204. Credentials cannot revoke themselves or issue replacement grants.",
            "tags": ["auth"], "security": [{"sidecarHostAuth": []}],
            "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "string"}}],
            "responses": {"204": {"description": "Grant invalidated"}, "401": {"description": "Invalid launch credential"}, "403": {"description": "Host authentication required"}, "404": {"description": "External JWT transport has no grant API"}}
        }
    });
    let operations = json!({"type": "array", "minItems": 1, "items": {"type": "string", "enum": ["discovery", "model_read", "model_completion", "full_harness_delegation"]}});
    let workspaces = json!({"type": "array", "minItems": 1, "items": {"type": "string", "minLength": 1}, "description": "Explicit workspace identifiers; no wildcard. Completion and full-harness requests must carry one matching x-uar-workspace-id."});
    spec["components"]["schemas"]["DelegationGrantRequest"] = json!({
        "type": "object", "additionalProperties": false,
        "required": ["workspace_ids", "operations"],
        "properties": {"workspace_ids": workspaces.clone(), "operations": operations.clone()}
    });
    spec["components"]["schemas"]["DelegationGrantReceipt"] = json!({
        "type": "object",
        "required": ["id", "token", "token_type", "expires_at", "expires_in", "principal", "workspace_ids", "operations", "instance_id", "runtime_epoch"],
        "properties": {
            "id": {"type": "string", "format": "uuid"},
            "token": {"type": "string", "pattern": "^[a-f0-9]{64}$", "description": "Secret credential returned only to the issuing host; never send to a renderer or persist/log it"},
            "token_type": {"type": "string", "enum": ["Bearer"]},
            "expires_at": {"type": "string", "format": "date-time"},
            "expires_in": {"type": "integer", "enum": [900]},
            "principal": {"type": "string"}, "workspace_ids": workspaces, "operations": operations,
            "instance_id": {"type": "string"}, "runtime_epoch": {"type": "string", "description": "Same epoch as full_harness_v1; restart invalidates old grants"}
        }
    });
    spec["components"]["securitySchemes"]["sidecarHostAuth"] = json!({"type": "http", "scheme": "bearer", "bearerFormat": "UAR launch token", "description": "Original host's private stdin launch credential, managed-local only"});
    spec["components"]["securitySchemes"]["delegationGrantAuth"] = json!({"type": "http", "scheme": "bearer", "bearerFormat": "UAR scoped delegation grant", "description": "Opaque short-lived grant limited to explicit methods, paths and workspaces"});
    spec["paths"]["/api/chat/completion"] = json!({
        "post": {
            "summary": "Native driver model completion",
            "description": "Existing native completion contract, stream:false or SSE. A managed-local grant requires model_completion and a scoped workspace header; no /v1/chat/completions or /api/uar/test authority is implied.",
            "tags": ["chat"], "security": [{"bearerAuth": []}, {"sidecarHostAuth": []}, {"delegationGrantAuth": []}],
            "parameters": [{"name": "x-uar-workspace-id", "in": "header", "required": false, "schema": {"type": "string"}, "description": "Required for a scoped delegation grant"}],
            "responses": {"200": {"description": "Existing completion result or SSE stream"}, "401": {"description": "Grant expired, revoked, or scope invalid"}}
        }
    });
    for path in [
        "/api/uar/full-harness/v1/tasks",
        "/api/uar/full-harness/v1/capabilities",
        "/api/uar/full-harness/v1/admissions/{admission_id}",
        "/api/uar/full-harness/v1/tasks/{task_id}",
        "/api/uar/full-harness/v1/tasks/{task_id}/stream",
        "/api/uar/full-harness/v1/tasks/{task_id}/tool-approval",
        "/api/uar/full-harness/v1/tasks/{task_id}/cancel",
        "/api/uar/full-harness/v1/tasks/{task_id}/detach",
        "/api/uar/full-harness/v1/tasks/{task_id}/steer",
    ] {
        for method in ["get", "post"] {
            if let Some(operation) = spec["paths"][path][method].as_object_mut() {
                operation.insert("security".to_owned(), json!([{"bearerAuth": []}, {"sidecarHostAuth": []}, {"delegationGrantAuth": []}]));
            }
        }
    }
}
