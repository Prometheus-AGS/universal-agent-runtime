//! Transport-free administration services.
//!
//! Every operation here takes the subsystem handles it actually needs
//! (`PersistenceLayer`, `McpRegistry`, `SettingsManager`) rather than an
//! `AppState`, so BOTH containers can call the same code:
//!
//!   * embedded (mobile, macOS) — the SDK `Runtime` calls these in-process
//!   * remote — `uar::api::*` handlers are thin adapters over these
//!
//! Before this existed the admin logic lived only in the axum handlers, so an
//! embedded container had no way to reach it and the control plane had to
//! report these registries as unavailable. Two parallel implementations would
//! have drifted; one shared service cannot.
//!
//! Configuration is stored in the DATABASE. Config files seed it once at boot
//! and the store is authoritative afterwards, which is what lets a runtime API
//! change take effect without a restart or a file-polling loop.

pub mod knowledge;
pub mod pack_sync;
pub mod mcp;
pub mod memory;
pub mod skills;
