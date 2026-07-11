//! Validated UAR production profile for A2UI v0.9.1 messages.

#![expect(
    dead_code,
    reason = "profile DTO fields are consumed by serde validation and retained for Rust/TypeScript contract parity"
)]

use serde::Deserialize;

pub(crate) const PROFILE: &str = "uar.a2ui/1";
pub(crate) const VERSION: &str = "v0.9.1";
pub(crate) const CATALOG_ID: &str = "urn:uar:a2ui:catalog:1";

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum A2uiMessage {
    Create(CreateMessage),
    Components(ComponentsMessage),
    Data(DataMessage),
    Delete(DeleteMessage),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateMessage {
    pub version: String,
    pub profile: String,
    pub create_surface: CreateSurface,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateSurface {
    pub surface_id: String,
    pub catalog_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ComponentsMessage {
    pub version: String,
    pub profile: String,
    pub update_components: UpdateComponents,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateComponents {
    pub surface_id: String,
    pub components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "component")]
pub(crate) enum Component {
    Text(TextComponent),
    Button(ButtonComponent),
    TextField(TextFieldComponent),
    CheckBox(CheckBoxComponent),
    ChoicePicker(ChoicePickerComponent),
    Row(ContainerComponent),
    Column(ContainerComponent),
    Card(CardComponent),
    Divider(DividerComponent),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TextComponent {
    pub id: String,
    pub text: DynamicString,
    pub variant: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ButtonComponent {
    pub id: String,
    pub child: String,
    pub variant: Option<String>,
    pub action: Action,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Action {
    pub event: ActionEvent,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ActionEvent {
    pub name: String,
    pub context: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TextFieldComponent {
    pub id: String,
    pub label: String,
    pub value: DynamicString,
    pub variant: Option<String>,
    pub placeholder: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CheckBoxComponent {
    pub id: String,
    pub label: String,
    pub value: DynamicBoolean,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ChoicePickerComponent {
    pub id: String,
    pub label: String,
    pub value: DynamicStringList,
    pub variant: String,
    pub options: Vec<ChoiceOption>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChoiceOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ContainerComponent {
    pub id: String,
    pub children: Vec<String>,
    pub justify: Option<String>,
    pub align: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CardComponent {
    pub id: String,
    pub child: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DividerComponent {
    pub id: String,
    pub axis: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum DynamicString {
    Literal(String),
    Path(DataPath),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum DynamicBoolean {
    Literal(bool),
    Path(DataPath),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum DynamicStringList {
    Literal(Vec<String>),
    Path(DataPath),
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DataPath {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DataMessage {
    pub version: String,
    pub profile: String,
    pub update_data_model: UpdateDataModel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateDataModel {
    pub surface_id: String,
    pub path: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeleteMessage {
    pub version: String,
    pub profile: String,
    pub delete_surface: DeleteSurface,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DeleteSurface {
    pub surface_id: String,
}

pub(crate) fn parse_message(value: serde_json::Value) -> Result<A2uiMessage, String> {
    fn contains_executable(value: &serde_json::Value) -> bool {
        match value {
            serde_json::Value::String(value) => {
                let normalized = value.to_ascii_lowercase();
                (normalized.contains('<') && normalized.contains('>'))
                    || normalized.contains("javascript:")
                    || normalized.contains("data:text/html")
                    || normalized.contains("onerror=")
                    || normalized.contains("onclick=")
            }
            serde_json::Value::Array(values) => values.iter().any(contains_executable),
            serde_json::Value::Object(values) => values.values().any(contains_executable),
            _ => false,
        }
    }
    if contains_executable(&value) {
        return Err("executable HTML or JavaScript is not allowed in A2UI data".to_string());
    }
    let message: A2uiMessage = serde_json::from_value(value).map_err(|error| error.to_string())?;
    let (version, profile, catalog) = match &message {
        A2uiMessage::Create(value) => (
            value.version.as_str(),
            value.profile.as_str(),
            Some(value.create_surface.catalog_id.as_str()),
        ),
        A2uiMessage::Components(value) => (value.version.as_str(), value.profile.as_str(), None),
        A2uiMessage::Data(value) => (value.version.as_str(), value.profile.as_str(), None),
        A2uiMessage::Delete(value) => (value.version.as_str(), value.profile.as_str(), None),
    };
    if version != VERSION || profile != PROFILE {
        return Err("unsupported A2UI version or profile".to_string());
    }
    if catalog.is_some_and(|value| value != CATALOG_ID) {
        return Err("unapproved A2UI catalog".to_string());
    }
    if let A2uiMessage::Components(value) = &message {
        let mut ids = std::collections::HashSet::new();
        for component in &value.update_components.components {
            let id = match component {
                Component::Text(value) => &value.id,
                Component::Button(value) => &value.id,
                Component::TextField(value) => &value.id,
                Component::CheckBox(value) => &value.id,
                Component::ChoicePicker(value) => &value.id,
                Component::Row(value) | Component::Column(value) => &value.id,
                Component::Card(value) => &value.id,
                Component::Divider(value) => &value.id,
            };
            if !ids.insert(id) {
                return Err("A2UI component IDs must be unique within an update".to_string());
            }
        }
    }
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::parse_message;

    #[test]
    fn accepts_profile_message_with_approved_component() {
        let message = serde_json::json!({
            "version": "v0.9.1", "profile": "uar.a2ui/1",
            "updateComponents": { "surfaceId": "surface-1", "components": [
                { "id": "root", "component": "Text", "text": "Ready", "variant": "body" }
            ]}
        });
        assert!(parse_message(message).is_ok());
    }

    #[test]
    fn rejects_unknown_component() {
        let message = serde_json::json!({
            "version": "v0.9.1", "profile": "uar.a2ui/1",
            "updateComponents": { "surfaceId": "surface-1", "components": [
                { "id": "root", "component": "Script", "source": "alert(1)" }
            ]}
        });
        assert!(parse_message(message).is_err());
    }

    #[test]
    fn rejects_executable_content_and_duplicate_ids() {
        let executable = serde_json::json!({
            "version": "v0.9.1", "profile": "uar.a2ui/1",
            "updateComponents": { "surfaceId": "surface-1", "components": [
                { "id": "root", "component": "Text", "text": "<script>alert(1)</script>" }
            ]}
        });
        assert!(parse_message(executable).is_err());

        let duplicate = serde_json::json!({
            "version": "v0.9.1", "profile": "uar.a2ui/1",
            "updateComponents": { "surfaceId": "surface-1", "components": [
                { "id": "root", "component": "Text", "text": "one" },
                { "id": "root", "component": "Text", "text": "two" }
            ]}
        });
        assert!(parse_message(duplicate).is_err());
    }
}
