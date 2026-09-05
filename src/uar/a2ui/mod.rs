//! A2UI — Agent-to-UI protocol support for UAR.
//!
//! A2UI allows agents to declare UI surfaces in their UAR-AGENT-MD spec (§06) and
//! emit interactive artifacts during a run. The system resolves `a2ui_schema_id`
//! values from this registry, emits `agui.artifact_input_request` SSE events to
//! the browser, renders the appropriate form/confirm/select widget, and routes the
//! user's response back to the paused agent run as a tool result.
//!
//! ## Key types
//! - [`schema::ArtifactSchema`] — definition of one A2UI surface type
//! - [`registry::A2uiRegistry`] — in-memory store of all registered schemas
//! - [`routes`] — REST endpoints for schema listing and user response ingestion

pub mod design_systems;
pub(crate) mod policy_surface;
pub mod presentation_selection;
pub mod presentations;
pub(crate) mod protocol;
pub mod realtime;
pub mod registry;
#[cfg(feature = "server")]
pub mod routes;
pub mod schema;
