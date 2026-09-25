use serde::{Deserialize, Deserializer, Serializer};

use crate::uar::tools::descriptor::ApprovalClass;

pub(super) fn serialize<S>(value: &ApprovalClass, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(match value {
        ApprovalClass::NotRequired => "not_required",
        ApprovalClass::Required => "required",
    })
}

pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<ApprovalClass, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_str() {
        "not_required" => Ok(ApprovalClass::NotRequired),
        "required" => Ok(ApprovalClass::Required),
        value => Err(serde::de::Error::unknown_variant(
            value,
            &["not_required", "required"],
        )),
    }
}
