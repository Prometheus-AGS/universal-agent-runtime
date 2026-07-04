//! UAR-AGENT-MD Compiler
//!
//! This module implements the PMPO-driven compilation pipeline that transforms
//! UAR-AGENT-MD Markdown specification documents into signed, runnable agent
//! descriptors. It supports two operating modes:
//!
//! - **Single-shot**: A complete document is parsed and compiled in one call
//!   via the [`CompilerAgentSkill`] NativeSkill (`uar.compile`).
//! - **Conversational**: A multi-turn session incrementally builds a partial IR,
//!   determines completeness, and compiles when ready via session tools:
//!   - `uar.session.update_section`
//!   - `uar.session.check_completeness`
//!   - `uar.session.compile`

pub mod compiler_skill;
pub mod completeness;
pub mod conformance;
pub mod conversational;
pub mod error;
pub mod ir;
pub mod parser;
pub mod pipeline;
pub mod registries;
pub mod report;
pub mod service;
pub mod session;
pub mod signing;
pub mod stages;
pub mod storage;
pub mod to_artifact;

// Re-exports for convenience
pub use compiler_skill::CompilerAgentSkill;
pub use conformance::{CheckResult, ConformanceReport, check_conformance};
pub use conversational::{
    CheckCompletenessTool, CompileSessionTool, CompilerSessionStore, UpdateSectionTool,
};
pub use error::CompileError;
pub use ir::{AgentDescriptorIR, PartialAgentDescriptorIR};
pub use parser::parse;
pub use pipeline::{CompileContext, CompileOutput, compile};
pub use report::CompileReport;
pub use service::CompilerService;
pub use session::CompilerSession;
pub use storage::{InMemorySpecStorage, SpecStorage};
