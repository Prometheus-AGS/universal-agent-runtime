#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    let Ok(value) = serde_json::from_str::<Value>(text) else {
        return;
    };

    let schema = serde_json::json!({ "type": "object" });
    if let Ok(validator) = jsonschema::validator_for(&schema) {
        let _ = validator.validate(&value);
    }
});
