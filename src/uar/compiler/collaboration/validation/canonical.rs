use std::collections::BTreeMap;
use std::fmt::Write as _;

use anyhow::{Result, anyhow, bail};
use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

pub(super) fn parse_unique_json(source: &str) -> Result<Value> {
    let mut deserializer = serde_json::Deserializer::from_str(source);
    let value = UniqueValue::deserialize(&mut deserializer)?.0;
    deserializer.end()?;
    Ok(value)
}

pub(in crate::uar::compiler::collaboration) fn canonical_digest(
    value: &Value,
) -> Result<String> {
    let mut without_digest = value.clone();
    without_digest
        .as_object_mut()
        .ok_or_else(|| anyhow!("collaboration document must be a JSON object"))?
        .remove("contentDigest");
    Ok(digest(&canonical_json(&without_digest)?))
}

pub(super) fn digest(bytes: &[u8]) -> String {
    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

pub(in crate::uar::compiler::collaboration) fn request_digest<T: serde::Serialize>(
    value: &T,
) -> Result<String> {
    let json = serde_json::to_value(value)?;
    Ok(digest(&canonical_json(&json)?))
}

fn canonical_json(value: &Value) -> Result<Vec<u8>> {
    match value {
        Value::Object(object) => {
            let sorted = object
                .iter()
                .map(|(key, value)| Ok((key.clone(), canonical_json_value(value)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            Ok(serde_json::to_vec(&sorted)?)
        }
        _ => Ok(serde_json::to_vec(&canonical_json_value(value)?)?),
    }
}

fn canonical_json_value(value: &Value) -> Result<Value> {
    match value {
        Value::Object(object) => {
            let mut sorted = Map::new();
            let mut entries = object.iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(right.0));
            for (key, value) in entries {
                sorted.insert(key.clone(), canonical_json_value(value)?);
            }
            Ok(Value::Object(sorted))
        }
        Value::Array(values) => values
            .iter()
            .map(canonical_json_value)
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        _ => Ok(value.clone()),
    }
}

pub(super) fn verify_content_digest(document: &Value) -> Result<()> {
    let declared = document
        .get("contentDigest")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("'contentDigest' must be a non-empty string"))?;
    let observed = canonical_digest(document)?;
    if declared != observed {
        bail!("contentDigest mismatch: expected {declared}, observed {observed}");
    }
    Ok(())
}

struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueValueVisitor).map(Self)
    }
}

struct UniqueValueVisitor;

impl<'de> Visitor<'de> for UniqueValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut object = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON key '{key}'"
                )));
            }
            let value = map.next_value::<UniqueValue>()?;
            object.insert(key, value.0);
        }
        Ok(Value::Object(object))
    }
}
