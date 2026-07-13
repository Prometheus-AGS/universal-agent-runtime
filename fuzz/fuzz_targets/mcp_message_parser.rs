#![no_main]

use libfuzzer_sys::fuzz_target;
use serde_json::Value;
use universal_agent_runtime::uar::api::acp::types::JsonRpcRequest;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    if let Ok(value) = serde_json::from_str::<Value>(text) {
        let _ = serde_json::from_value::<JsonRpcRequest>(value);
    }
});
