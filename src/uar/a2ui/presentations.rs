//! Reusable, owner-scoped Presentation templates and safe surface instantiation.
//!
//! Templates are data. Only the trusted host persists them or publishes their
//! instantiated messages; this module does neither.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use super::protocol::{CATALOG_ID, PROFILE, VERSION, parse_message};

/// Declarative content for exactly one surface, independent of any run ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PresentationTemplate {
    /// Approved A2UI wire version.
    pub version: String,
    /// Approved component catalog; never fetched or executed by the host.
    pub catalog_id: String,
    /// Complete component graph rooted at the component named `root`.
    pub components: Vec<Value>,
    /// Initial root data; invocation data replaces matching top-level keys.
    #[serde(default)]
    pub default_data: Map<String, Value>,
}

impl Default for PresentationTemplate {
    fn default() -> Self {
        Self {
            version: VERSION.to_string(),
            catalog_id: CATALOG_ID.to_string(),
            components: vec![json!({
                "id": "root", "component": "Text", "text": {"path": "/message"}
            })],
            default_data: Map::from_iter([("message".to_string(), json!("Ready"))]),
        }
    }
}

impl PresentationTemplate {
    /// Validate the canonical wire messages and the entire rooted graph.
    ///
    /// # Errors
    /// Returns an error for unsafe protocol content, missing references,
    /// cycles or components unreachable from `root`.
    pub fn validate(&self) -> Result<(), String> {
        // Presentation templates target the first-party production profile;
        // the legacy tool's broader compatibility parser remains unchanged.
        if self.version != VERSION || self.catalog_id != CATALOG_ID {
            return Err("Presentations require v0.9.1 and urn:uar:a2ui:catalog:1".into());
        }
        if self.components.is_empty() || self.components.len() > 500 {
            return Err("Presentations require between 1 and 500 components".into());
        }
        for message in self.messages("template-validation", self.default_data.clone()) {
            parse_message(message)?;
        }
        let mut graph = BTreeMap::new();
        for component in &self.components {
            validate_component(component)?;
            // The protocol parser above established the component's shape.
            let id = component["id"]
                .as_str()
                .filter(|id| !id.trim().is_empty())
                .ok_or("Presentation component IDs must not be blank")?;
            let mut children = Vec::new();
            if let Some(child) = component.get("child").and_then(Value::as_str) {
                children.push(child);
            }
            if let Some(values) = component.get("children").and_then(Value::as_array) {
                for value in values {
                    children.push(value.as_str().ok_or("Invalid component child reference")?);
                }
            }
            graph.insert(id, children);
        }
        let mut referenced = BTreeSet::new();
        for (id, children) in &graph {
            for child in children {
                if !graph.contains_key(child) {
                    return Err(format!(
                        "Presentation component `{id}` references missing `{child}`"
                    ));
                }
                if *child == "root" || !referenced.insert(*child) {
                    return Err(format!(
                        "Presentation component `{child}` must have exactly one parent"
                    ));
                }
            }
        }
        // Iterative traversal avoids recursive calls on an operator-authored graph.
        let mut active = BTreeSet::new();
        let mut complete = BTreeSet::new();
        let mut pending = vec![("root", false)];
        while let Some((id, leaving)) = pending.pop() {
            if leaving {
                active.remove(id);
                complete.insert(id);
                continue;
            }
            if complete.contains(id) {
                continue;
            }
            if !active.insert(id) {
                return Err(format!("Presentation component cycle includes `{id}`"));
            }
            let children = graph
                .get(id)
                .ok_or("Presentation requires a root component")?;
            pending.push((id, true));
            pending.extend(children.iter().rev().map(|child| (*child, false)));
        }
        if let Some(id) = graph.keys().find(|id| !complete.contains(**id)) {
            return Err(format!(
                "Presentation component `{id}` is unreachable from root"
            ));
        }
        validate_data(&Value::Object(self.default_data.clone()))
    }

    /// Instantiate a surface using a host-selected ID and declarative data.
    ///
    /// # Errors
    /// Rejects invalid templates, empty surface IDs or unsafe invocation data.
    /// No string interpolation or action execution takes place.
    pub fn instantiate(
        &self,
        surface_id: &str,
        data: &Map<String, Value>,
    ) -> Result<Vec<Value>, String> {
        self.validate()?;
        if !identifier(&json!(surface_id)) {
            return Err(
                "Presentation surface ID must match the supported renderer identifier format"
                    .into(),
            );
        }
        let mut merged = self.default_data.clone();
        merged.extend(data.clone());
        validate_data(&Value::Object(merged.clone()))?;
        let messages = self.messages(surface_id, merged);
        for message in &messages {
            parse_message(message.clone())?;
        }
        Ok(messages)
    }

    fn messages(&self, surface_id: &str, data: Map<String, Value>) -> Vec<Value> {
        let mut messages = vec![
            json!({
                "version": self.version, "profile": PROFILE,
                "createSurface": {"surfaceId": surface_id, "catalogId": self.catalog_id}
            }),
            json!({
                "version": self.version, "profile": PROFILE,
                "updateComponents": {"surfaceId": surface_id, "components": self.components}
            }),
        ];
        // The existing client reducer treats `/` as an empty property, not a
        // root replacement. Emit escaped top-level pointers instead.
        for (key, value) in data {
            let path = format!("/{}", key.replace('~', "~0").replace('/', "~1"));
            messages.push(json!({
                "version": self.version, "profile": PROFILE,
                "updateDataModel": {"surfaceId": surface_id, "path": path, "value": value}
            }));
        }
        messages
    }
}

fn bounded_string(value: &Value, min: usize, max: usize) -> bool {
    value
        .as_str()
        .is_some_and(|value| (min..=max).contains(&value.encode_utf16().count()))
}

fn identifier(value: &Value) -> bool {
    value.as_str().is_some_and(|value| {
        !value.is_empty()
            && value.len() <= 128
            && value.as_bytes()[0].is_ascii_alphanumeric()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    })
}

fn reserved_key(value: &str) -> bool {
    matches!(value, "__proto__" | "prototype" | "constructor")
}

fn validate_data(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if reserved_key(key) {
                    return Err(
                        "Presentation data must not contain prototype property names".into(),
                    );
                }
                validate_data(&Value::String(key.clone()))?;
                if key
                    .replace('~', "~0")
                    .replace('/', "~1")
                    .encode_utf16()
                    .count()
                    + 1
                    > 512
                {
                    return Err(
                        "Presentation data keys must fit the client's 512-character pointer limit"
                            .into(),
                    );
                }
                validate_data(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_data(value)?;
            }
        }
        Value::String(value) => {
            let normalized = value.to_ascii_lowercase();
            let compact: String = normalized
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect();
            if (normalized.contains('<') && normalized.contains('>'))
                || compact.contains("javascript:")
                || compact.contains("data:text/html")
                || contains_event_assignment(&normalized)
            {
                return Err(
                    "Executable HTML or JavaScript is not allowed in Presentation data".into(),
                );
            }
        }
        _ => {}
    }
    Ok(())
}

fn contains_event_assignment(value: &str) -> bool {
    let bytes = value.as_bytes();
    for (start, _) in value.match_indices("on") {
        if start > 0 && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_') {
            continue;
        }
        let tail = &value[start + 2..];
        let letters = tail.bytes().take_while(u8::is_ascii_alphabetic).count();
        if letters > 0 && tail[letters..].trim_start().starts_with('=') {
            return true;
        }
    }
    false
}

fn validate_component(component: &Value) -> Result<(), String> {
    let id = component["id"].as_str().unwrap_or("");
    let invalid = || {
        format!(
            "Presentation component `{id}` does not match the supported renderer; check its identifiers, variant, labels and bindings"
        )
    };
    if !identifier(&component["id"]) {
        return Err(invalid());
    }
    for field in ["child"] {
        if let Some(value) = component.get(field) {
            if !identifier(value) {
                return Err(invalid());
            }
        }
    }
    if let Some(children) = component.get("children").and_then(Value::as_array) {
        if children.len() > 200 || children.iter().any(|value| !identifier(value)) {
            return Err(invalid());
        }
    }
    let kind = component["component"].as_str().unwrap_or("");
    let variants: &[&str] = match kind {
        "Text" => &["h1", "h2", "h3", "body", "caption"],
        "Button" => &["primary", "secondary", "borderless"],
        "TextField" => &["shortText", "longText", "email", "number"],
        "ChoicePicker" => &["mutuallyExclusive", "multipleSelection"],
        _ => &[],
    };
    if let Some(value) = component.get("variant") {
        if !value
            .as_str()
            .is_some_and(|value| variants.contains(&value))
        {
            return Err(invalid());
        }
    }
    for (field, options) in [
        ("align", &["start", "center", "end", "stretch"][..]),
        ("justify", &["start", "center", "end", "spaceBetween"][..]),
        ("axis", &["horizontal", "vertical"][..]),
    ] {
        if let Some(value) = component.get(field) {
            if !value.as_str().is_some_and(|value| options.contains(&value)) {
                return Err(invalid());
            }
        }
    }
    if kind == "Column" && component.get("justify").is_some() {
        return Err(invalid());
    }
    for (field, min, max) in [("label", 1, 256), ("placeholder", 0, 512)] {
        if let Some(value) = component.get(field) {
            if !bounded_string(value, min, max) {
                return Err(invalid());
            }
        }
    }
    for field in ["text", "value"] {
        if let Some(value) = component.get(field) {
            if let Some(path) = value.get("path") {
                if !bounded_string(path, 1, 512)
                    || !path.as_str().is_some_and(|path| {
                        path.starts_with('/')
                            && !path.split('/').any(|part| {
                                reserved_key(&part.replace("~1", "/").replace("~0", "~"))
                            })
                    })
                {
                    return Err(invalid());
                }
            } else if value.is_string() && !bounded_string(value, 0, 16_384) {
                return Err(invalid());
            } else if let Some(values) = value.as_array() {
                if values.len() > 100 || values.iter().any(|value| !bounded_string(value, 0, 1024))
                {
                    return Err(invalid());
                }
            }
        }
    }
    if let Some(options) = component.get("options").and_then(Value::as_array) {
        if options.is_empty()
            || options.len() > 100
            || options.iter().any(|option| {
                !bounded_string(&option["value"], 1, 256)
                    || !bounded_string(&option["label"], 1, 256)
            })
        {
            return Err(invalid());
        }
    }
    if let Some(action) = component.get("action") {
        if !identifier(&action["event"]["name"]) {
            return Err(invalid());
        }
        if action["event"]
            .get("context")
            .is_some_and(|context| !context.is_object())
        {
            return Err(invalid());
        }
    }
    validate_data(component)
}

/// Editable fields; identity, owner and revision are assigned by the host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PresentationDraft {
    /// Human-readable template name.
    pub title: String,
    /// What this template presents and when an agent should choose it.
    #[serde(default)]
    pub description: String,
    /// Whether future runs may select this template.
    pub enabled: bool,
    /// Safe declarative content.
    pub template: PresentationTemplate,
}

impl PresentationDraft {
    /// Validate editable fields before crossing a persistence boundary.
    ///
    /// # Errors
    /// Rejects a blank title or invalid template.
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Presentation title must not be blank".to_string());
        }
        self.template.validate()
    }
}

/// Durable record whose owner and revision are managed by the trusted host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Presentation {
    /// Stable host-generated identifier.
    pub id: String,
    /// Verified owner partition; never accepted from an editor draft.
    pub owner_id: String,
    /// Monotonic version used for optimistic concurrency and run provenance.
    pub revision: u64,
    /// Editable content, retained in full by admitted run snapshots.
    pub content: PresentationDraft,
    /// Host creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last accepted mutation.
    pub updated_at: DateTime<Utc>,
}
