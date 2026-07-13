#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;
use universal_agent_runtime::uar::rag::chunking::{Chunker, ChunkingStrategy};

fuzz_target!(|data: &[u8]| {
    let Ok(text) = str::from_utf8(data) else {
        return;
    };

    let size = text.len().max(1).min(1024);
    let chunker = Chunker::new(ChunkingStrategy::FixedSize { size }, None);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _ = rt.block_on(chunker.chunk(text));
});
