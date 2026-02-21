//! A2UI artifact schema definitions.
//!
//! Each [`ArtifactSchema`] describes one UI surface type an agent can declare.
//! The `schema_id` matches the `a2ui_schema_id` field in UAR-AGENT-MD §06.

use serde::{Deserialize, Serialize};

/// How the artifact should be rendered and evaluated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    /// A structured form driven by a JSON Schema. The user fills fields and submits.
    /// The agent receives the field values as a tool result.
    Form,
    /// A simple yes/no or accept/cancel confirmation prompt.
    Confirm,
    /// A single-choice dropdown populated from `options` in the metadata.
    Select,
    /// A single free-text input field. Simpler than `form` for one-field cases.
    TextInput,
    /// A read-only display artifact (rendered markdown, HTML, or code). No user response required.
    Display,
    /// A data chart or visualization. Read-only.
    Chart,
    /// An audio or video media artifact. Read-only.
    Media,
}

impl ArtifactType {
    /// Returns true if the artifact requires a user response before the agent can continue.
    pub fn requires_response(&self) -> bool {
        matches!(self, Self::Form | Self::Confirm | Self::Select | Self::TextInput)
    }
}

/// Definition of one A2UI surface type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSchema {
    /// Unique schema identifier — matches `a2ui_schema_id` in UAR-AGENT-MD §06.
    pub schema_id: String,
    /// Display name of this schema for developer tooling.
    pub title: String,
    /// Description of what this schema is for.
    pub description: String,
    /// How the artifact is rendered and whether user input is required.
    pub artifact_type: ArtifactType,
    /// JSON Schema string describing the expected content format.
    /// For `form`, this drives field generation. For `display`, describes content structure.
    pub json_schema: serde_json::Value,
    /// Optional rendering hint for the UI layer (e.g. "modal", "inline", "sidebar").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render_hint: Option<String>,
    /// Whether this schema is built into UAR (true) or user-defined (false).
    pub builtin: bool,
}

// ── Built-in schema constructors ──────────────────────────────────────────────

impl ArtifactSchema {
    /// `a2ui/form` — generic structured form from a JSON Schema.
    pub fn builtin_form() -> Self {
        Self {
            schema_id: "a2ui/form".to_string(),
            title: "Structured Form".to_string(),
            description: "A form with labeled fields derived from a JSON Schema. The agent pauses until the user submits the form.".to_string(),
            artifact_type: ArtifactType::Form,
            json_schema: serde_json::json!({
                "type": "object",
                "description": "Pass any JSON Schema as the artifact content. Each property becomes a form field.",
                "properties": {
                    "title": { "type": "string", "description": "Form title shown to the user" },
                    "fields": {
                        "type": "object",
                        "description": "JSON Schema object — each key is a field name, value is a JSON Schema field definition",
                        "additionalProperties": true
                    }
                }
            }),
            render_hint: Some("inline".to_string()),
            builtin: true,
        }
    }

    /// `a2ui/confirm` — yes/no or accept/cancel confirmation prompt.
    pub fn builtin_confirm() -> Self {
        Self {
            schema_id: "a2ui/confirm".to_string(),
            title: "Confirmation Prompt".to_string(),
            description: "A simple confirmation dialog with Accept and Cancel (or custom) buttons. The agent receives true/false.".to_string(),
            artifact_type: ArtifactType::Confirm,
            json_schema: serde_json::json!({
                "type": "object",
                "required": ["message"],
                "properties": {
                    "message": { "type": "string", "description": "The question or action to confirm" },
                    "accept_label": { "type": "string", "description": "Accept button label (default: Accept)" },
                    "cancel_label": { "type": "string", "description": "Cancel button label (default: Cancel)" }
                }
            }),
            render_hint: Some("modal".to_string()),
            builtin: true,
        }
    }

    /// `a2ui/select` — single-choice dropdown from a list of options.
    pub fn builtin_select() -> Self {
        Self {
            schema_id: "a2ui/select".to_string(),
            title: "Option Selector".to_string(),
            description: "A dropdown or button list where the user picks one option. The agent receives the selected value.".to_string(),
            artifact_type: ArtifactType::Select,
            json_schema: serde_json::json!({
                "type": "object",
                "required": ["prompt", "options"],
                "properties": {
                    "prompt": { "type": "string", "description": "Instruction shown above the options" },
                    "options": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "required": ["value", "label"],
                            "properties": {
                                "value": { "type": "string" },
                                "label": { "type": "string" },
                                "description": { "type": "string" }
                            }
                        }
                    }
                }
            }),
            render_hint: Some("inline".to_string()),
            builtin: true,
        }
    }

    /// `a2ui/text-input` — single free-text field.
    pub fn builtin_text_input() -> Self {
        Self {
            schema_id: "a2ui/text-input".to_string(),
            title: "Text Input".to_string(),
            description: "A single text field prompting the user for free-form input. Simpler than form for single-value collection.".to_string(),
            artifact_type: ArtifactType::TextInput,
            json_schema: serde_json::json!({
                "type": "object",
                "required": ["prompt"],
                "properties": {
                    "prompt": { "type": "string", "description": "Label or question shown above the input" },
                    "placeholder": { "type": "string", "description": "Placeholder text inside the input" },
                    "multiline": { "type": "boolean", "description": "If true, renders a textarea instead of a single-line input" }
                }
            }),
            render_hint: Some("inline".to_string()),
            builtin: true,
        }
    }

    /// `a2ui/display` — read-only markdown/HTML content display.
    pub fn builtin_display() -> Self {
        Self {
            schema_id: "a2ui/display".to_string(),
            title: "Content Display".to_string(),
            description: "A read-only display of markdown or HTML content. No user response required; the agent continues immediately.".to_string(),
            artifact_type: ArtifactType::Display,
            json_schema: serde_json::json!({
                "type": "object",
                "required": ["content"],
                "properties": {
                    "content": { "type": "string", "description": "Markdown or HTML content to display" },
                    "format": { "type": "string", "enum": ["markdown", "html"], "description": "Content format (default: markdown)" }
                }
            }),
            render_hint: Some("inline".to_string()),
            builtin: true,
        }
    }
}
