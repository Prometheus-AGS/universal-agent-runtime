//! Session and conversation thread management.
//!
//! This module provides in-memory session storage for managing conversation
//! state across multiple requests. Sessions are identified by UUID and contain
//! the full message history.
//!
//! # Architecture
//!
//! - [`Session`]: Represents a single conversation session
//! - [`SessionStore`]: Thread-safe store for all active sessions
//! - [`EncryptedSession`]: Encrypted wrapper for secure communications
//! - [`EncryptedSessionStore`]: Store managing encrypted sessions
//!
//! # Example
//!
//! ```rust
//! use universal_agent_runtime::session::{Session, SessionStore};
//!
//! let store = SessionStore::new();
//! let session = store.create();
//! session.add_user_message("Hello!");
//!
//! let messages = session.messages();
//! assert_eq!(messages.len(), 1);
//! ```

mod encrypted;
mod thread;

pub(crate) use thread::ANONYMOUS_SESSION_OWNER;
#[allow(unused_imports)]
pub use thread::Session;
pub use thread::SessionStore;

// Encrypted session types (optional feature)
pub use encrypted::{
    // Re-export encryption types for convenience
    ConversationId,
    ConversationManager,
    DeviceId,
    EncryptedSession,
    EncryptedSessionStore,
    EncryptionConfig,
    EncryptionType,
    StreamingSession,
    StrongholdKeyManager,
    UserId,
    WebRTCEncryptedSession,
};
