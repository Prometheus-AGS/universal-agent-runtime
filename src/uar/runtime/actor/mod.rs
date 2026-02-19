//! Actor-model agent collaboration system.
//!
//! Enables agents to run as independently addressable actors that communicate
//! via async message passing, supporting multi-agent collaboration patterns
//! such as delegation, negotiation, and orchestration.
//!
//! Built on the [`ractor`] crate for lightweight, high-performance actor
//! lifecycle management.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────┐
//! │                ActorCollaboration                 │
//! │                                                  │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐      │
//! │  │ AgentActor│  │ AgentActor│  │ AgentActor│     │
//! │  │    (A)    │  │    (B)    │  │    (C)    │     │
//! │  └──────┬───┘  └──────┬───┘  └──────┬───┘      │
//! │         │              │              │          │
//! │         └──────────────┼──────────────┘          │
//! │              Message Passing                     │
//! └──────────────────────────────────────────────────┘
//! ```

pub mod agent_actor;
pub mod messages;
pub mod system;
