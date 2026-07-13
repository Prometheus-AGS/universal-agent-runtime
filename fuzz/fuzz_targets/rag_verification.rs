#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;
use universal_agent_runtime::uar::rag::verification::verify;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = str::from_utf8(data) else {
        return;
    };

    let parts: Vec<&str> = text.splitn(2, '\0').collect();
    if parts.len() == 2 {
        let _ = verify(parts[0], parts[1]);
    }
});
