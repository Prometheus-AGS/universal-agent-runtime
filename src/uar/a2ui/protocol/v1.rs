//! Supported subset of the pinned A2UI 1.0 wire and Basic Catalog schemas.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde_json::{Map, Value};

use super::{A2uiWireKind, BASIC_CATALOG_ID_1_0, CATALOG_ID, ValidatedA2uiMessage};

pub(super) const FUNCTIONS_UNSUPPORTED: &str = "A2UI function calls are not supported";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CreateSurface {
    surface_id: String,
    catalog_id: String,
    components: Option<Vec<Value>>,
    data_model: Option<Map<String, Value>>,
    send_data_model: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateComponents {
    surface_id: String,
    components: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateDataModel {
    surface_id: String,
    path: Option<String>,
    value: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DeleteSurface {
    surface_id: String,
}

pub(super) fn parse(raw: Value) -> Result<ValidatedA2uiMessage, String> {
    let fields = raw.as_object().ok_or("expected A2UI 1.0 object")?;
    if fields.len() != 2 {
        return Err("A2UI 1.0 requires version and exactly one message payload".into());
    }
    let (key, payload) = fields.iter().find(|(key, _)| *key != "version")
        .ok_or("missing A2UI 1.0 message payload")?;
    for field in ["components", "dataModel", "sendDataModel", "path"] {
        if payload.get(field).is_some_and(Value::is_null) {
            return Err(format!("A2UI {field} must not be null"));
        }
    }
    let (surface_id, kind) = match key.as_str() {
        "createSurface" => {
            let create: CreateSurface = decode(payload)?;
            if !approved_catalog(&create.catalog_id) {
                return Err("unapproved A2UI catalog".into());
            }
            if create.send_data_model == Some(true) {
                return Err("A2UI sendDataModel is not supported".into());
            }
            if create.data_model.is_some() {
                validate_data(&payload["dataModel"], false)?;
            }
            if let Some(components) = &create.components {
                validate_components(components)?;
            }
            (create.surface_id, A2uiWireKind::CreateSurface)
        }
        "updateComponents" => {
            let update: UpdateComponents = decode(payload)?;
            validate_components(&update.components)?;
            (update.surface_id, A2uiWireKind::UpdateComponents)
        }
        "updateDataModel" => {
            let update: UpdateDataModel = decode(payload)?;
            if let Some(path) = &update.path {
                validate_pointer(path)?;
            }
            validate_data(&update.value, false)?;
            (update.surface_id, A2uiWireKind::UpdateDataModel)
        }
        "deleteSurface" => {
            let delete: DeleteSurface = decode(payload)?;
            (delete.surface_id, A2uiWireKind::DeleteSurface)
        }
        _ => return Err("unsupported A2UI 1.0 message type".into()),
    };
    if surface_id.trim().is_empty() || surface_id.encode_utf16().count() > 128 {
        return Err("A2UI surface IDs must contain between 1 and 128 characters".into());
    }
    Ok(ValidatedA2uiMessage { surface_id, kind, payload: payload.clone(), raw })
}

fn decode<T: serde::de::DeserializeOwned>(value: &Value) -> Result<T, String> {
    serde_json::from_value(value.clone()).map_err(|error| error.to_string())
}

fn approved_catalog(catalog: &str) -> bool {
    matches!(catalog, CATALOG_ID | BASIC_CATALOG_ID_1_0)
}

fn validate_components(values: &[Value]) -> Result<(), String> {
    if values.is_empty() || values.len() > 500 {
        return Err("A2UI updates require between 1 and 500 components".into());
    }
    let mut ids = HashSet::new();
    for component in values {
        let object = component.as_object().ok_or("invalid A2UI component")?;
        let id = component["id"].as_str().filter(|id| component_id(id))
            .ok_or("invalid A2UI component ID")?;
        if !ids.insert(id) {
            return Err("A2UI component IDs must be unique within an update".into());
        }
        reject_functions(component)?;
        if let Some(catalog) = object.get("@catalogId")
            && !catalog.as_str().is_some_and(approved_catalog)
        {
            return Err("unapproved A2UI catalog".into());
        }
        validate_component(object)?;
    }
    Ok(())
}

fn validate_component(fields: &Map<String, Value>) -> Result<(), String> {
    let kind = fields.get("component").and_then(Value::as_str).ok_or("missing A2UI component type")?;
    let (required, optional): (&[&str], &[&str]) = match kind {
        "Text" => (&["text"], &["variant"]),
        "Button" => (&["child", "action"], &["variant"]),
        "TextField" => (&["label", "value"], &["placeholder", "variant"]),
        "CheckBox" => (&["label", "value"], &[]),
        "ChoicePicker" => (&["value", "options"], &["label", "variant"]),
        "Row" | "Column" => (&["children"], &["justify", "align"]),
        "Card" => (&["child"], &[]),
        "Divider" => (&[], &["axis"]),
        _ => return Err(format!("unsupported A2UI component {kind}")),
    };
    if required.iter().any(|field| !fields.contains_key(*field)) {
        return Err(format!("missing required A2UI {kind} properties"));
    }
    for (field, value) in fields {
        let field = field.as_str();
        if !(matches!(field, "id" | "component" | "@catalogId")
            || required.contains(&field)
            || optional.contains(&field))
        {
            return Err(format!("unsupported A2UI {kind} property {field}"));
        }
        match field {
            "text" | "label" | "placeholder" => dynamic(value, dynamic_text)?,
            "value" if kind == "TextField" => dynamic(value, dynamic_text)?,
            "value" if kind == "CheckBox" => dynamic(value, Value::is_boolean)?,
            "value" => dynamic(value, string_list)?,
            "child" if !value.as_str().is_some_and(component_id) => {
                return Err("invalid A2UI child reference".into());
            }
            "children" if !value.as_array().is_some_and(|ids| {
                ids.len() <= 200 && ids.iter().all(|id| id.as_str().is_some_and(component_id))
            }) => return Err("invalid A2UI child references".into()),
            "options" => validate_options(value)?,
            "action" => validate_action(value)?,
            "axis" => one_of(value, &["horizontal", "vertical"])?,
            "align" => one_of(value, &["start", "center", "end", "stretch"])?,
            "justify" => one_of(value, &["start", "center", "end", "spaceBetween", "spaceAround", "spaceEvenly", "stretch"])?,
            "variant" => one_of(
                value,
                match kind {
                    "Text" => &["body", "caption"],
                    "Button" => &["default", "primary", "borderless"],
                    "TextField" => &["shortText", "longText", "number", "obscured"],
                    "ChoicePicker" => &["mutuallyExclusive", "multipleSelection"],
                    _ => &[],
                },
            )?,
            _ => {}
        }
    }
    Ok(())
}

fn string_list(value: &Value) -> bool {
    value.as_array().is_some_and(|items| {
        items.len() <= 100 && items.iter().all(|item| text_limit(item, 1024))
    })
}

fn text_limit(value: &Value, limit: usize) -> bool {
    value.as_str().is_some_and(|value| value.encode_utf16().count() <= limit)
}

fn dynamic_text(value: &Value) -> bool {
    text_limit(value, 16_384)
}

fn component_id(id: &str) -> bool {
    let mut chars = id.chars();
    id.encode_utf16().count() <= 128
        && chars.next().is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
        && chars.all(unicode_ident::is_xid_continue)
}

fn one_of(value: &Value, allowed: &[&str]) -> Result<(), String> {
    if value.as_str().is_some_and(|value| allowed.contains(&value)) {
        return Ok(());
    }
    Err("invalid A2UI component property value".into())
}

fn validate_options(value: &Value) -> Result<(), String> {
    let options = value.as_array().ok_or("invalid A2UI choice options")?;
    if options.len() > 100 {
        return Err("A2UI choice pickers support up to 100 options".into());
    }
    for option in options {
        if !option.as_object().is_some_and(|fields| fields.len() == 2)
            || !text_limit(&option["value"], 256)
        {
            return Err("invalid A2UI choice option".into());
        }
        dynamic(&option["label"], dynamic_text)?;
    }
    Ok(())
}

fn validate_action(value: &Value) -> Result<(), String> {
    let fields = value.as_object().filter(|fields| fields.len() == 1)
        .ok_or("invalid A2UI event action")?;
    if fields.contains_key("functionCall") {
        return Err(FUNCTIONS_UNSUPPORTED.into());
    }
    let event = fields.get("event").and_then(Value::as_object).ok_or("invalid A2UI event action")?;
    if !event.get("name").and_then(Value::as_str).is_some_and(|name| {
        !name.trim().is_empty() && name.encode_utf16().count() <= 128
    }) {
        return Err("invalid A2UI event name".into());
    }
    for (field, value) in event {
        match field.as_str() {
            "name" => {}
            "userMessage" => dynamic(value, dynamic_text)?,
            "context" => {
                let context = value.as_object().ok_or("invalid A2UI event context")?;
                for (key, value) in context {
                    validate_key(key)?;
                    dynamic(value, |value| !value.is_null())?;
                }
            }
            _ => return Err(format!("unsupported A2UI event property {field}")),
        }
    }
    Ok(())
}

fn dynamic(value: &Value, literal: impl Fn(&Value) -> bool) -> Result<(), String> {
    if let Some(path) = value.get("@path") {
        if !value.as_object().is_some_and(|fields| fields.len() == 1) {
            return Err("invalid A2UI data binding".into());
        }
        return validate_pointer(path.as_str().ok_or("invalid A2UI data binding")?);
    }
    if !literal(value) {
        return Err("invalid A2UI literal or data binding".into());
    }
    validate_data(value, true)
}

fn validate_key(key: &str) -> Result<(), String> {
    if matches!(key, "__proto__" | "constructor" | "prototype") {
        return Err("reserved A2UI data key".into());
    }
    Ok(())
}

fn validate_data(value: &Value, dynamic_literal: bool) -> Result<(), String> {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                validate_key(key)?;
                if dynamic_literal && key.starts_with('@') && !key.starts_with("@@") {
                    return Err("unescaped reserved A2UI data key".into());
                }
                validate_data(value, dynamic_literal)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                validate_data(item, dynamic_literal)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_pointer(path: &str) -> Result<(), String> {
    if matches!(path, "" | "/") {
        return Ok(());
    }
    if path.encode_utf16().count() > 512 || !path.starts_with('/') {
        return Err("invalid A2UI JSON pointer".into());
    }
    for segment in path.split('/').skip(1) {
        if segment.is_empty() {
            return Err("A2UI JSON pointer segments must not be empty".into());
        }
        let mut bytes = segment.bytes();
        while let Some(byte) = bytes.next() {
            if byte == b'~' && !matches!(bytes.next(), Some(b'0' | b'1')) {
                return Err("invalid A2UI JSON pointer escape".into());
            }
        }
        validate_key(&segment.replace("~1", "/").replace("~0", "~"))?;
    }
    Ok(())
}

fn reject_functions(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(fields) => {
            if fields.contains_key("@call") {
                return Err(FUNCTIONS_UNSUPPORTED.into());
            }
            for value in fields.values() {
                reject_functions(value)?;
            }
        }
        Value::Array(items) => {
            for item in items {
                reject_functions(item)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Validate batch-local ordering while allowing a continuation to update a
/// surface that was already created in an earlier publication.
pub(super) fn validate_sequence(messages: &[ValidatedA2uiMessage]) -> Result<(), String> {
    let mut surfaces = HashMap::new();
    for message in messages {
        if message.raw["version"] != super::VERSION_1_0 {
            continue;
        }
        let state = surfaces.entry(message.surface_id.as_str()).or_insert(None);
        match (message.kind, *state) {
            (A2uiWireKind::CreateSurface, Some(true)) => {
                return Err("A2UI creation follows an existing surface update or creation".into());
            }
            (A2uiWireKind::DeleteSurface, Some(false)) => {
                return Err("A2UI surface is already deleted".into());
            }
            (A2uiWireKind::DeleteSurface, _) => *state = Some(false),
            (A2uiWireKind::CreateSurface, _) => *state = Some(true),
            (_, Some(false)) => return Err("A2UI surface update follows deletion".into()),
            _ => *state = Some(true),
        }
    }
    Ok(())
}
