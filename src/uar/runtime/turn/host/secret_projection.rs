//! Finite captured-value projection. This is not a general credential detector.
use base64::{
    Engine as _,
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE, URL_SAFE_NO_PAD},
};
use secrecy::{ExposeSecret, SecretString};
use serde_json::{Value, json};

const REPLACEMENT: &str = "[REDACTED]";

#[derive(Clone, Default)]
pub(super) struct SecretProjection {
    patterns: Vec<SecretString>,
}

impl SecretProjection {
    pub(super) fn new(values: Vec<SecretString>) -> Self {
        let mut patterns = Vec::new();
        for value in values {
            let value = value.expose_secret();
            if value.is_empty() {
                continue;
            }
            let quoted = serde_json::to_string(value).expect("string JSON serialization");
            for variant in [
                value.to_owned(),
                quoted[1..quoted.len() - 1].to_owned(),
                percent(value, false),
                percent(value, true),
                STANDARD.encode(value),
                STANDARD_NO_PAD.encode(value),
                URL_SAFE.encode(value),
                URL_SAFE_NO_PAD.encode(value),
            ] {
                patterns.push(SecretString::from(variant));
            }
        }
        let mut projection = Self { patterns };
        projection.normalize();
        projection
    }

    fn normalize(&mut self) {
        self.patterns.sort_by(|a, b| {
            b.expose_secret()
                .len()
                .cmp(&a.expose_secret().len())
                .then_with(|| a.expose_secret().cmp(b.expose_secret()))
        });
        self.patterns
            .dedup_by(|a, b| a.expose_secret() == b.expose_secret());
    }

    pub(super) fn extend(&mut self, other: Self) {
        self.patterns.extend(other.patterns);
        self.normalize();
    }

    /// Consume only complete original-input matches. A possible longer match
    /// at the end stays private until the next fragment or explicit finish.
    pub(super) fn prefix(&self, input: &str, finish: bool) -> (String, usize, bool) {
        let mut output = String::new();
        let mut cursor = 0;
        let mut changed = false;
        while cursor < input.len() {
            let tail = &input[cursor..];
            if !finish
                && self.patterns.iter().any(|pattern| {
                    let pattern = pattern.expose_secret();
                    pattern.len() > tail.len() && pattern.starts_with(tail)
                })
            {
                break;
            }
            if let Some(pattern) = self
                .patterns
                .iter()
                .find(|pattern| tail.starts_with(pattern.expose_secret()))
            {
                output.push_str(REPLACEMENT);
                cursor += pattern.expose_secret().len();
                changed = true;
            } else {
                let character = tail.chars().next().expect("nonempty UTF-8 tail");
                output.push(character);
                cursor += character.len_utf8();
            }
        }
        (output, cursor, changed)
    }

    pub(super) fn text(&self, input: &str) -> (String, bool) {
        let (output, _, changed) = self.prefix(input, true);
        (output, changed)
    }

    pub(super) fn value(&self, value: Value) -> (Value, bool) {
        match value {
            Value::String(value) => {
                let (value, changed) = self.text(&value);
                (Value::String(value), changed)
            }
            Value::Array(values) => {
                let mut changed = false;
                let values = values
                    .into_iter()
                    .map(|value| {
                        let (value, projected) = self.value(value);
                        changed |= projected;
                        value
                    })
                    .collect();
                (Value::Array(values), changed)
            }
            Value::Object(values) => {
                let mut changed = false;
                let mut keys = std::collections::BTreeSet::new();
                let mut collision = false;
                let mut entries = Vec::with_capacity(values.len());
                for (key, value) in values {
                    let (key, key_changed) = self.text(&key);
                    let (value, value_changed) = self.value(value);
                    changed |= key_changed || value_changed;
                    collision |= !keys.insert(key.clone());
                    entries.push((key, value));
                }
                if collision {
                    // An explicit projected representation preserves both values;
                    // it cannot pretend to preserve the original object's shape.
                    (
                        json!({"$uar_projection":"object_key_collision","entries":entries
                        .into_iter().map(|(key,value)| json!({"key":key,"value":value})).collect::<Vec<_>>()}),
                        true,
                    )
                } else {
                    (Value::Object(entries.into_iter().collect()), changed)
                }
            }
            value => (value, false),
        }
    }
}

fn percent(value: &str, lowercase: bool) -> String {
    let digits = if lowercase {
        b"0123456789abcdef"
    } else {
        b"0123456789ABCDEF"
    };
    let mut output = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(digits[usize::from(byte >> 4)]));
            output.push(char::from(digits[usize::from(byte & 15)]));
        }
    }
    output
}
