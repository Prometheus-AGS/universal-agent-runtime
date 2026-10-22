//! ACP (Agent Communication Protocol) server module.
//!
//! Exposes a JSON-RPC 2.0 endpoint at the configured path (default: `/acp`)
//! for agent introspection, session management, and streaming run execution.
//!
//! Caller-owned tools are an extension of this JSON-RPC surface, not the
//! similarly named Agent Client Protocol. `sessions/create` may persist a
//! `client_tools` declaration; `runs/create` may replace it for one run.
//! `client_tools` contains OpenAI-format `tools`, tagged `tool_choice`, optional
//! `parallel_tool_calls`, and canonical `history`. Tool choice is
//! `{"type":"auto"}`, `{"type":"none"}`, `{"type":"required"}`, or
//! `{"type":"named","name":"function_name"}`.
//!
//! Caller-tool runs return their terminal event projection and, when required,
//! `client_tools` with the exact visible predecessor history. Append a complete
//! batch of `{"role":"tool","tool_call_id":"...","content":...}` results
//! to that history and submit `runs/submit_tool_results` with the same session
//! and declarations. Retain the exact predecessor: results alone do not bind a
//! continuation. Matching completed submissions replay the stored response;
//! interrupted claimed submissions remain unavailable rather than repeat effects.
//! Owner, tenant, agent, session, protocol, schema and batch bindings are enforced.
//!
//! `/stream` emits normalized events followed by an `acp_result` event containing
//! the continuation envelope for caller-tool runs. These runs are collected and
//! durably completed before public replay, so their streaming response is buffered.
//! Durable persistence is required. Session declarations themselves remain
//! process-local under the existing session-store lifetime; a durable batch can
//! restore its exact session on an authenticated result submission after restart.
//!
//! Enable via config: `acp.enabled = true`
//! or CLI: `--acp-enabled` / env: `UAR_ACP__ENABLED=true`

pub mod handler;
pub mod routes;
pub mod types;

pub use handler::AcpSessionStore;
pub use routes::AcpRouter;
