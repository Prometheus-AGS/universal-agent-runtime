//! WebAssembly sandbox runtime for executing Wasm agents.
//!
//! This module is **feature-gated** behind `wasm-runtime`.
//! Enable it with: `cargo build --features wasm-runtime`
//!
//! # Architecture
//!
//! - [`config::WasmConfig`] — Sandbox configuration (memory limits, timeouts, ACLs)
//! - [`sandbox::WasmSandbox`] — Core execution engine wrapping `wasmtime`
//! - [`host_functions`] — Custom host functions exposed to Wasm guests

pub mod config;
pub mod host_functions;
pub mod sandbox;
