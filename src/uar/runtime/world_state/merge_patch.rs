//! JSON Merge Patch generation and application, per RFC 7386 section 2:
//! <https://www.rfc-editor.org/rfc/rfc7386.html#section-2>.
//! Arrays and scalars are atomic; null object members remove keys.

use serde_json::{Map, Value};

/// A requested object-member value cannot be represented by a merge patch.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MergePatchError {
    #[error("RFC 7386 cannot add or replace object member '{key}' with explicit null")]
    ExplicitNull { key: String },
}

/// Produce only changed object members, or an atomic replacement value.
/// `None` means no change, including unchanged scalar/array/null inputs.
///
/// # Errors
/// Returns an error when a changed object member must contain explicit null;
/// RFC 7386 reserves that representation for deletion. Nulls inside arrays
/// remain representable because arrays are replaced whole.
pub fn generate(before: &Value, after: &Value) -> Result<Option<Value>, MergePatchError> {
    if before == after {
        return Ok(None);
    }
    let Value::Object(after) = after else {
        return Ok(Some(after.clone()));
    };
    let before = before.as_object();
    let mut patch = Map::new();
    if let Some(before) = before {
        for key in before.keys().filter(|key| !after.contains_key(*key)) {
            patch.insert(key.clone(), Value::Null);
        }
    }
    for (key, value) in after {
        let previous = before.and_then(|object| object.get(key));
        if previous == Some(value) {
            continue;
        }
        if value.is_null() {
            return Err(MergePatchError::ExplicitNull { key: key.clone() });
        }
        if let Some(change) = generate(previous.unwrap_or(&Value::Null), value)? {
            patch.insert(key.clone(), change);
        }
    }
    // An empty object still replaces a non-object with an object.
    Ok(Some(Value::Object(patch)))
}

/// Apply a merge patch without modifying either input.
pub fn apply(target: &Value, patch: &Value) -> Value {
    let Value::Object(patch) = patch else {
        return patch.clone();
    };
    let mut result = target.as_object().cloned().unwrap_or_default();
    for (key, change) in patch {
        if change.is_null() {
            result.remove(key);
        } else {
            let updated = apply(result.get(key).unwrap_or(&Value::Null), change);
            result.insert(key.clone(), updated);
        }
    }
    Value::Object(result)
}
