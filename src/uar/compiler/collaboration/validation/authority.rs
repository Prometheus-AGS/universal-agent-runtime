use anyhow::{Result, bail};
use serde_json::Value;

use crate::uar::domain::collaboration::{ConversionDisposition, FieldDiagnostic};

use super::diagnostics::{
    PRIVATE_AUTHORITY, RECOGNIZED_SECRET, pointer_child, redacted_diagnostic,
};

const PRIVATE_FIELDS: &[&str] = &[
    "apikey",
    "approvaltoken",
    "authoritytoken",
    "connectionref",
    "consentevidence",
    "consentevidenceref",
    "credential",
    "credentialref",
    "grantid",
    "granteeagentinstanceid",
    "issuerprincipalid",
    "organizationalauthorityevidenceref",
    "ownerid",
    "password",
    "privatekey",
    "rawconsent",
    "representationgrantrefs",
    "runtimeinstanceid",
    "secret",
    "signature",
    "subjectprincipalid",
    "token",
    "workspaceid",
];

pub(super) fn validate_portable_authority(document: &Value) -> Result<()> {
    let diagnostics = portable_authority_diagnostics(document);
    if diagnostics.is_empty() {
        return Ok(());
    }
    let locations = diagnostics
        .iter()
        .map(|diagnostic| format!("{} at {}", diagnostic.reason_code, diagnostic.pointer))
        .collect::<Vec<_>>()
        .join(", ");
    bail!("portable document contains excluded private material: {locations}")
}

pub(super) fn portable_authority_diagnostics(document: &Value) -> Vec<FieldDiagnostic> {
    let mut diagnostics = Vec::new();
    inspect_value(document, "", &mut diagnostics);
    diagnostics
}

fn inspect_value(value: &Value, pointer: &str, diagnostics: &mut Vec<FieldDiagnostic>) {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let child_pointer = pointer_child(pointer, key);
                if PRIVATE_FIELDS.contains(&normalized_key(key).as_str()) {
                    diagnostics.push(redacted_diagnostic(
                        child_pointer.clone(),
                        ConversionDisposition::RequiredUnsupported,
                        PRIVATE_AUTHORITY,
                        "Portable content contains a private authority or binding field.",
                    ));
                }
                inspect_value(child, &child_pointer, diagnostics);
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                inspect_value(
                    child,
                    &pointer_child(pointer, &index.to_string()),
                    diagnostics,
                );
            }
        }
        Value::String(text) if looks_like_recognized_secret(text) => {
            diagnostics.push(redacted_diagnostic(
                if pointer.is_empty() { "/" } else { pointer },
                ConversionDisposition::RequiredUnsupported,
                RECOGNIZED_SECRET,
                "Portable content contains a recognized secret value.",
            ));
        }
        _ => {}
    }
}

fn normalized_key(key: &str) -> String {
    key.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn looks_like_recognized_secret(text: &str) -> bool {
    text.split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, '"' | '\'' | ',' | ';'))
        .any(|token| {
            token.starts_with("example_secret_value_")
                || token.starts_with("sk-live-")
                || token.starts_with("sk_live_")
                || token.starts_with("ghp_")
                || token.starts_with("github_pat_")
                || (token.starts_with("AKIA")
                    && token.len() == 20
                    && token
                        .chars()
                        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()))
        })
}
