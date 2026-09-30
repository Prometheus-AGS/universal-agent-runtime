use crate::uar::domain::collaboration::{
    ConversionDiagnostic, ConversionDisposition, FieldDiagnostic,
};
use serde_json::Value;

pub(super) const PRIVATE_AUTHORITY: &str = "portable.private-authority";
pub(super) const RECOGNIZED_SECRET: &str = "portable.recognized-secret";

pub(super) fn pointer_child(parent: &str, token: &str) -> String {
    let token = token.replace('~', "~0").replace('/', "~1");
    if parent.is_empty() {
        format!("/{token}")
    } else {
        format!("{parent}/{token}")
    }
}

pub(super) fn redacted_diagnostic(
    pointer: impl Into<String>,
    disposition: ConversionDisposition,
    reason_code: impl Into<String>,
    message: impl Into<String>,
) -> FieldDiagnostic {
    FieldDiagnostic {
        pointer: pointer.into(),
        disposition,
        reason_code: reason_code.into(),
        message: message.into(),
        effective_binding_ref: None, source_kind: None, source_definition: None,
    }
}

pub(super) fn upgrade_projection_diagnostic(diagnostic: &ConversionDiagnostic) -> FieldDiagnostic {
    let pointer = legacy_field_pointer(&diagnostic.field);
    let reason_code = match &diagnostic.disposition {
        ConversionDisposition::Exact => "projection.exact",
        ConversionDisposition::Translated => "projection.translated",
        ConversionDisposition::OptionalUnsupported => "projection.optional-unsupported",
        ConversionDisposition::RequiredUnsupported => "projection.required-unsupported",
    };
    redacted_diagnostic(
        pointer,
        diagnostic.disposition.clone(),
        reason_code,
        redacted_projection_message(&diagnostic.disposition),
    )
}

pub(super) fn redact_projection_diagnostics(
    diagnostics: Vec<ConversionDiagnostic>,
) -> Vec<ConversionDiagnostic> {
    diagnostics
        .into_iter()
        .map(|diagnostic| ConversionDiagnostic {
            field: diagnostic.field,
            message: redacted_projection_message(&diagnostic.disposition).to_owned(),
            disposition: diagnostic.disposition,
        })
        .collect()
}

pub(super) fn complete_field_diagnostics(
    document: &Value,
    legacy: &[ConversionDiagnostic],
) -> Vec<FieldDiagnostic> {
    let mut diagnostics = legacy
        .iter()
        .map(upgrade_projection_diagnostic)
        .collect::<Vec<_>>();
    let mut leaf_pointers = Vec::new();
    collect_leaf_pointers(document, "", &mut leaf_pointers);
    for pointer in leaf_pointers {
        if diagnostics.iter().any(|diagnostic| {
            pointer == diagnostic.pointer
                || pointer
                    .strip_prefix(diagnostic.pointer.as_str())
                    .is_some_and(|suffix| suffix.starts_with('/'))
        }) {
            continue;
        }
        diagnostics.push(redacted_diagnostic(
            pointer,
            ConversionDisposition::Exact,
            "source.preserved",
            "The source field is retained exactly.",
        ));
    }
    diagnostics.sort_by(|left, right| left.pointer.cmp(&right.pointer));
    diagnostics
}

fn collect_leaf_pointers(value: &Value, pointer: &str, output: &mut Vec<String>) {
    match value {
        Value::Object(object) if !object.is_empty() => {
            for (key, child) in object {
                collect_leaf_pointers(child, &pointer_child(pointer, key), output);
            }
        }
        Value::Array(values) if !values.is_empty() => {
            for (index, child) in values.iter().enumerate() {
                collect_leaf_pointers(child, &pointer_child(pointer, &index.to_string()), output);
            }
        }
        _ => output.push(if pointer.is_empty() {
            "/".to_owned()
        } else {
            pointer.to_owned()
        }),
    }
}

fn legacy_field_pointer(field: &str) -> String {
    if let Some(extension_name) = field.strip_prefix("extensions.") {
        pointer_child("/extensions", extension_name)
    } else if field.starts_with('/') {
        field.to_owned()
    } else {
        field.split('.').fold(String::new(), |pointer, token| {
            pointer_child(&pointer, token)
        })
    }
}

fn redacted_projection_message(disposition: &ConversionDisposition) -> &'static str {
    match disposition {
        ConversionDisposition::Exact => "The field is retained exactly.",
        ConversionDisposition::Translated => "The field is retained with a documented translation.",
        ConversionDisposition::OptionalUnsupported => {
            "The optional field is preserved without a runtime support claim."
        }
        ConversionDisposition::RequiredUnsupported => {
            "The required field is preserved and blocks activation until it is supported."
        }
    }
}
