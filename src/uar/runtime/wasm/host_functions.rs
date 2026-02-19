//! Custom host functions exposed to Wasm guests.
//!
//! These functions allow Wasm modules to communicate back to the UAR
//! runtime, e.g. for logging, emitting events, or reading context.

use std::sync::{Arc, Mutex};

use wasmtime::{AsContext, Caller, Linker, Memory};

/// Collected log messages from a Wasm guest execution.
#[derive(Debug, Default, Clone)]
pub struct GuestLogs {
    pub messages: Arc<Mutex<Vec<String>>>,
}

impl GuestLogs {
    /// Create a new, empty log collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all collected log messages.
    pub fn drain(&self) -> Vec<String> {
        let mut msgs = self.messages.lock().expect("log mutex poisoned");
        msgs.drain(..).collect()
    }
}

/// Register UAR-specific host functions into a wasmtime [`Linker`].
///
/// Host functions available to the guest:
///
/// - `uar_log(ptr: i32, len: i32)` — Log a UTF-8 message from guest memory.
/// - `uar_emit_event(ptr: i32, len: i32)` — Emit a JSON event to the UAR event bus.
///
/// Both functions read a UTF-8 string from the guest's linear memory at
/// `[ptr..ptr+len]`.
pub fn register_host_functions(
    linker: &mut Linker<crate::uar::runtime::wasm::sandbox::SandboxState>,
    logs: GuestLogs,
) -> anyhow::Result<()> {
    let logs_clone = logs.clone();

    // uar_log: Write a log message from guest
    linker.func_wrap(
        "uar",
        "uar_log",
        move |mut caller: Caller<'_, crate::uar::runtime::wasm::sandbox::SandboxState>,
              ptr: i32,
              len: i32| {
            if let Some(msg) = read_guest_string(&mut caller, ptr, len) {
                tracing::info!(source = "wasm-guest", "{}", msg);
                if let Ok(mut msgs) = logs_clone.messages.lock() {
                    msgs.push(msg);
                }
            }
        },
    )?;

    let logs_clone2 = logs;

    // uar_emit_event: Emit a JSON event from guest
    linker.func_wrap(
        "uar",
        "uar_emit_event",
        move |mut caller: Caller<'_, crate::uar::runtime::wasm::sandbox::SandboxState>,
              ptr: i32,
              len: i32| {
            if let Some(event_json) = read_guest_string(&mut caller, ptr, len) {
                tracing::debug!(source = "wasm-guest", event = %event_json, "Guest event");
                if let Ok(mut msgs) = logs_clone2.messages.lock() {
                    msgs.push(format!("[event] {event_json}"));
                }
            }
        },
    )?;

    Ok(())
}

/// Read a UTF-8 string from the guest's linear memory.
fn read_guest_string(
    caller: &mut Caller<'_, crate::uar::runtime::wasm::sandbox::SandboxState>,
    ptr: i32,
    len: i32,
) -> Option<String> {
    let memory = caller
        .get_export("memory")
        .and_then(wasmtime::Extern::into_memory)?;

    read_string_from_memory(&memory, &caller.as_context(), ptr, len)
}

/// Read a UTF-8 string from a specific memory instance.
fn read_string_from_memory(
    memory: &Memory,
    store: &impl AsContext,
    ptr: i32,
    len: i32,
) -> Option<String> {
    #[allow(clippy::cast_sign_loss)]
    let ptr = ptr as usize;
    #[allow(clippy::cast_sign_loss)]
    let len = len as usize;
    let data = memory.data(store);

    if ptr.checked_add(len)? > data.len() {
        return None;
    }

    String::from_utf8(data[ptr..ptr + len].to_vec()).ok()
}
