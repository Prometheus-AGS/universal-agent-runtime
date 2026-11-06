//! Conversation thread and session storage.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

use crate::llm::{Message, MessageContent, MessageRole, ToolCall};

/// Default session timeout (30 minutes).
#[allow(dead_code)]
const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(30 * 60);

pub(crate) const ANONYMOUS_SESSION_OWNER: &str = "anonymous";

fn anonymous_session_owner() -> String {
    ANONYMOUS_SESSION_OWNER.to_string()
}

/// A single conversation session.
///
/// Sessions maintain the full message history and provide methods
/// for adding messages and retrieving state.
#[derive(Debug)]
pub struct Session {
    inner: Arc<SessionInner>,
    history_run: Option<(String, Option<String>)>,
}

#[derive(Debug)]
struct SessionInner {
    /// Unique session identifier.
    id: String,
    /// Authenticated user that owns this session.
    owner_id: String,
    /// Conversation messages.
    messages: RwLock<Vec<Message>>,
    message_metadata: RwLock<BTreeMap<usize, super::SessionMessageMetadata>>,
    /// Session creation time.
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    /// Last activity time.
    last_activity: RwLock<DateTime<Utc>>,
    /// Optional system prompt.
    system_prompt: RwLock<Option<String>>,
    /// Host-only baseline. It is not accepted through serialized session input.
    world_state: RwLock<crate::uar::runtime::world_state::contributor::WorldStateBaseline>,
    project_instructions:
        RwLock<Option<crate::uar::runtime::project_instructions::ProjectInstructions>>,
    /// Store-owned activity generation. Standalone deserialized sessions do
    /// not have a retention owner and therefore carry no sender.
    activity_generation: Option<tokio::sync::watch::Sender<u64>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionState {
    pub id: String,
    #[serde(default = "anonymous_session_owner")]
    pub owner_id: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub message_metadata: BTreeMap<usize, super::SessionMessageMetadata>,
    pub created_at: String,    // RFC3339
    pub last_activity: String, // RFC3339
    pub system_prompt: Option<String>,
}

/// Atomic result of attempting to seed a host-restored conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedOutcome {
    Seeded { messages: usize },
    IgnoredWarmSession,
}

impl Serialize for Session {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let state = self.to_state();
        state.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Session {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let state = SessionState::deserialize(deserializer)?;
        Ok(Session::from_state(state))
    }
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            history_run: self.history_run.clone(),
        }
    }
}

impl Session {
    /// Create a new session with the given ID.
    #[cfg(test)]
    fn new(id: String, owner_id: String) -> Self {
        Self::new_with_activity(id, owner_id, None)
    }

    fn new_with_activity(
        id: String,
        owner_id: String,
        activity_generation: Option<tokio::sync::watch::Sender<u64>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            inner: Arc::new(SessionInner {
                id,
                owner_id,
                messages: RwLock::new(Vec::new()),
                message_metadata: RwLock::new(BTreeMap::new()),
                created_at: now,
                last_activity: RwLock::new(now),
                system_prompt: RwLock::new(None),
                world_state: RwLock::new(Default::default()),
                project_instructions: RwLock::new(None),
                activity_generation,
            }),
            history_run: None,
        }
    }

    pub fn to_state(&self) -> SessionState {
        let messages = self.inner.messages.read().unwrap();
        let metadata = self.inner.message_metadata.read().unwrap();
        SessionState {
            id: self.inner.id.clone(),
            owner_id: self.inner.owner_id.clone(),
            messages: messages.clone(),
            message_metadata: metadata.clone(),
            created_at: self.inner.created_at.to_rfc3339(),
            last_activity: self.inner.last_activity.read().unwrap().to_rfc3339(),
            system_prompt: self.inner.system_prompt.read().unwrap().clone(),
        }
    }

    pub fn from_state(mut state: SessionState) -> Self {
        let message_count = state.messages.len();
        state
            .message_metadata
            .retain(|index, _| *index < message_count);
        let created_at = DateTime::parse_from_rfc3339(&state.created_at)
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
            .with_timezone(&Utc);
        let last_activity = DateTime::parse_from_rfc3339(&state.last_activity)
            .unwrap_or_else(|_| DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
            .with_timezone(&Utc);

        Self {
            inner: Arc::new(SessionInner {
                id: state.id,
                owner_id: state.owner_id,
                messages: RwLock::new(state.messages),
                message_metadata: RwLock::new(state.message_metadata),
                created_at,
                last_activity: RwLock::new(last_activity),
                system_prompt: RwLock::new(state.system_prompt),
                world_state: RwLock::new(Default::default()),
                project_instructions: RwLock::new(None),
                activity_generation: None,
            }),
            history_run: None,
        }
    }

    /// A run-local handle to the same conversation. Concurrent runs never share
    /// this append context; it is not serialized or used as authority.
    pub fn with_run_context(&self, run_id: &str, client_turn_id: Option<String>) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            history_run: Some((run_id.to_owned(), client_turn_id)),
        }
    }

    /// Replace redacted message content while preserving existing positional IDs.
    /// Only same-length projection is allowed; this cannot erase pending tools.
    pub fn project_message_content(&self, messages: Vec<Message>) -> anyhow::Result<()> {
        let mut current = self.inner.messages.write().unwrap();
        anyhow::ensure!(
            messages.len() == current.len(),
            "session_projection_length_mismatch"
        );
        *current = messages;
        drop(current);
        self.touch();
        Ok(())
    }

    /// Publish protected replacement history without regenerating its persisted IDs.
    /// The host must already have authorized the source history; ownership cannot move.
    pub fn replace_message_history_from(&self, source: &Self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.id() == source.id() && self.owner_id() == source.owner_id(),
            "session_history_partition_mismatch"
        );
        let source = source.to_state();
        let mut messages = self.inner.messages.write().unwrap();
        let mut identities = self.inner.message_metadata.write().unwrap();
        *messages = source.messages;
        *identities = source.message_metadata;
        drop(identities);
        drop(messages);
        self.touch();
        Ok(())
    }

    /// Get the session ID.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.inner.id
    }

    /// Get the authenticated user that owns this session.
    #[must_use]
    pub fn owner_id(&self) -> &str {
        &self.inner.owner_id
    }

    /// Snapshot the host-only baseline owned by this exact tenant session.
    pub(crate) fn world_state_baseline(
        &self,
    ) -> crate::uar::runtime::world_state::contributor::WorldStateBaseline {
        self.inner.world_state.read().unwrap().clone()
    }

    /// Publish the selected contribution, retaining its messages in session history.
    pub(crate) fn record_world_state(
        &self,
        update: &crate::uar::runtime::world_state::contributor::WorldStateUpdate,
        instructions: crate::uar::runtime::project_instructions::ProjectInstructions,
    ) {
        {
            let mut messages = self.inner.messages.write().unwrap();
            let mut baseline = self.inner.world_state.write().unwrap();
            messages.extend(update.messages.iter().cloned());
            *baseline = update.baseline.clone();
            *self.inner.project_instructions.write().unwrap() = Some(instructions);
        }
        self.touch();
    }

    /// Previously accessed subtrees; never accepted from serialized session data.
    pub(crate) fn project_instructions(
        &self,
    ) -> Option<crate::uar::runtime::project_instructions::ProjectInstructions> {
        self.inner.project_instructions.read().unwrap().clone()
    }

    /// Set the system prompt for this session.
    #[allow(dead_code)]
    pub fn set_system_prompt(&self, prompt: impl Into<String>) {
        let mut guard = self.inner.system_prompt.write().unwrap();
        *guard = Some(prompt.into());
        self.touch();
    }

    /// Get the system prompt if set.
    #[must_use]
    pub fn system_prompt(&self) -> Option<String> {
        self.inner.system_prompt.read().unwrap().clone()
    }

    /// Add a user message to the conversation.
    pub fn add_user_message(&self, content: impl Into<String>) {
        let msg = Message {
            role: MessageRole::User,
            content: MessageContent::text(content),
            tool_call_id: None,
            tool_calls: None,
        };
        self.add_message(msg);
    }

    /// Add an assistant message to the conversation.
    #[allow(dead_code)]
    pub fn add_assistant_message(&self, content: impl Into<String>) {
        let msg = Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(content),
            tool_call_id: None,
            tool_calls: None,
        };
        self.add_message(msg);
    }

    /// Add an assistant message with tool calls.
    #[allow(dead_code)]
    pub fn add_assistant_with_tool_calls(
        &self,
        content: Option<String>,
        tool_calls: Vec<ToolCall>,
    ) {
        let msg = Message {
            role: MessageRole::Assistant,
            content: MessageContent::text(content.unwrap_or_default()),
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        };
        self.add_message(msg);
    }

    /// Add a tool result message.
    #[allow(dead_code)]
    pub fn add_tool_result(&self, tool_call_id: impl Into<String>, content: impl Into<String>) {
        let msg = Message {
            role: MessageRole::Tool,
            content: MessageContent::text(content),
            tool_call_id: Some(tool_call_id.into()),
            tool_calls: None,
        };
        self.add_message(msg);
    }

    /// Add a message to the conversation.
    pub fn add_message(&self, message: Message) {
        let message_id = if self.history_run.is_some() && message.role == MessageRole::Assistant {
            String::new()
        } else {
            Uuid::new_v4().to_string()
        };
        self.add_message_with_metadata(
            message,
            super::SessionMessageMetadata {
                message_id,
                run_id: self.history_run.as_ref().map(|(run, _)| run.clone()),
                client_turn_id: self.history_run.as_ref().and_then(|(_, turn)| turn.clone()),
                created_at: Some(Utc::now().to_rfc3339()),
                status: if self.history_run.is_some() {
                    super::SessionMessageStatus::Running
                } else {
                    super::SessionMessageStatus::Unknown
                },
            },
        );
    }

    /// Append canonical content and its presentation identity under the same lock.
    pub fn add_message_with_metadata(
        &self,
        message: Message,
        mut metadata: super::SessionMessageMetadata,
    ) {
        let mut messages = self.inner.messages.write().unwrap();
        let mut identities = self.inner.message_metadata.write().unwrap();
        if metadata.message_id.is_empty()
            && let Some(run_id) = metadata.run_id.as_deref()
        {
            let ordinal = messages
                .iter()
                .enumerate()
                .filter(|(index, message)| {
                    message.role == MessageRole::Assistant
                        && identities
                            .get(index)
                            .is_some_and(|identity| identity.run_id.as_deref() == Some(run_id))
                })
                .count();
            metadata.message_id = format!("run-{run_id}-assistant-{ordinal}");
        }
        identities.insert(messages.len(), metadata);
        messages.push(message);
        drop(identities);
        drop(messages);
        self.touch();
    }

    /// Update only messages already bound to the exact host run.
    pub fn set_run_history_status(&self, run_id: &str, status: super::SessionMessageStatus) {
        let mut identities = self.inner.message_metadata.write().unwrap();
        for metadata in identities
            .values_mut()
            .filter(|metadata| metadata.run_id.as_deref() == Some(run_id))
        {
            metadata.status = status;
        }
        drop(identities);
        self.touch();
    }

    /// Safe presentation history. Legacy IDs are deterministic and time/outcome stay unknown.
    pub fn history(&self) -> Vec<super::SessionHistoryMessage> {
        let messages = self.inner.messages.read().unwrap();
        let identities = self.inner.message_metadata.read().unwrap();
        messages
            .iter()
            .enumerate()
            .filter(|(_, message)| {
                matches!(message.role, MessageRole::User | MessageRole::Assistant)
            })
            .map(|(index, message)| {
                let metadata = identities.get(&index);
                super::SessionHistoryMessage {
                    message_id: metadata.map_or_else(
                        || super::history::legacy_message_id(self.owner_id(), self.id(), index),
                        |metadata| metadata.message_id.clone(),
                    ),
                    role: message.role.clone(),
                    content: message.content.clone(),
                    run_id: metadata.and_then(|metadata| metadata.run_id.clone()),
                    client_turn_id: metadata.and_then(|metadata| metadata.client_turn_id.clone()),
                    created_at: metadata.and_then(|metadata| metadata.created_at.clone()),
                    status: metadata.map_or(super::SessionMessageStatus::Unknown, |metadata| {
                        metadata.status
                    }),
                    provenance: if metadata.is_some_and(|metadata| metadata.run_id.is_some()) {
                        "available"
                    } else {
                        "unavailable"
                    },
                }
            })
            .collect()
    }

    /// Seed a cold session under one write lock. Concurrent first turns cannot
    /// both observe an empty session and duplicate the host's history.
    pub fn seed_if_empty(&self, messages: &[Message]) -> SeedOutcome {
        let mut guard = self.inner.messages.write().unwrap();
        if !guard.is_empty() {
            return SeedOutcome::IgnoredWarmSession;
        }
        guard.extend(messages.iter().cloned());
        let seeded = messages.len();
        drop(guard);
        self.touch();
        SeedOutcome::Seeded { messages: seeded }
    }

    /// Restore a cold same-owner session including original message metadata.
    /// Concurrent warm sessions retain their current append authority.
    pub fn seed_persisted_history_if_empty(&self, source: &Self) -> anyhow::Result<SeedOutcome> {
        anyhow::ensure!(
            self.id() == source.id() && self.owner_id() == source.owner_id(),
            "session_history_partition_mismatch"
        );
        let source = source.to_state();
        let mut messages = self.inner.messages.write().unwrap();
        if !messages.is_empty() {
            return Ok(SeedOutcome::IgnoredWarmSession);
        }
        let count = source.messages.len();
        let mut identities = self.inner.message_metadata.write().unwrap();
        *messages = source.messages;
        *identities = source.message_metadata;
        drop(identities);
        drop(messages);
        self.touch();
        Ok(SeedOutcome::Seeded { messages: count })
    }

    /// Get all messages in the conversation.
    #[must_use]
    pub fn messages(&self) -> Vec<Message> {
        self.inner.messages.read().unwrap().clone()
    }

    /// Get all messages including the system prompt.
    #[must_use]
    pub fn messages_with_system(&self) -> Vec<Message> {
        let mut result = Vec::new();

        if let Some(prompt) = self.system_prompt() {
            result.push(Message {
                role: MessageRole::System,
                content: MessageContent::text(prompt),
                tool_call_id: None,
                tool_calls: None,
            });
        }

        result.extend(self.messages());
        result
    }

    /// Get the number of messages in the conversation.
    #[must_use]
    pub fn message_count(&self) -> usize {
        self.inner.messages.read().unwrap().len()
    }

    /// Clear all messages from the session.
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut guard = self.inner.messages.write().unwrap();
        guard.clear();
        self.inner.message_metadata.write().unwrap().clear();
        *self.inner.world_state.write().unwrap() = Default::default();
        *self.inner.project_instructions.write().unwrap() = None;
        self.touch();
    }

    /// Update the last activity timestamp.
    pub(crate) fn touch(&self) {
        let mut guard = self.inner.last_activity.write().unwrap();
        *guard = Utc::now();
        drop(guard);
        if let Some(activity_generation) = &self.inner.activity_generation {
            activity_generation.send_modify(|generation| {
                *generation = generation.saturating_add(1);
            });
        }
    }

    /// Check if the session has expired.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_expired(&self) -> bool {
        self.is_expired_with_timeout(DEFAULT_SESSION_TIMEOUT)
    }

    /// Check if the session has expired with a custom timeout.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_expired_with_timeout(&self, timeout: Duration) -> bool {
        let last = *self.inner.last_activity.read().unwrap();
        // Convert Duration to chrono::Duration?
        // chrono subtraction: now - last -> Duration
        // We want (now - last) > timeout
        let now = Utc::now();
        if let Ok(duration) = (now - last).to_std() {
            duration > timeout
        } else {
            // Negative duration means clock skew or "last" is in future.
            false
        }
    }

    /// Get the session age.
    #[must_use]
    #[allow(dead_code)]
    pub fn age(&self) -> Duration {
        let now = Utc::now();
        (now - self.inner.created_at)
            .to_std()
            .unwrap_or(Duration::from_secs(0))
    }
}

/// Thread-safe store for sessions.
///
/// Provides methods for creating, retrieving, and cleaning up sessions.
#[derive(Debug, Clone)]
pub struct SessionStore {
    inner: Arc<SessionStoreInner>,
}

#[derive(Debug)]
struct SessionStoreInner {
    sessions: RwLock<HashMap<(String, String), Session>>,
    activity_generation: tokio::sync::watch::Sender<u64>,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    /// Create a new session store.
    #[must_use]
    pub fn new() -> Self {
        let (activity_generation, _) = tokio::sync::watch::channel(0);
        Self {
            inner: Arc::new(SessionStoreInner {
                sessions: RwLock::new(HashMap::new()),
                activity_generation,
            }),
        }
    }

    fn notify_activity(&self) {
        self.inner.activity_generation.send_modify(|generation| {
            *generation = generation.saturating_add(1);
        });
    }

    /// Retained generation for session creation, attachment, and removal.
    /// Consumers inspect the authoritative session state after every change.
    #[must_use]
    pub(crate) fn subscribe_activity(&self) -> tokio::sync::watch::Receiver<u64> {
        self.inner.activity_generation.subscribe()
    }

    /// Earliest local idle boundary for sessions that are not protected by an
    /// active run. This is a business-time wakeup; `cleanup_bounded` remains
    /// the atomic expiry decision when the wakeup fires.
    pub(crate) fn next_idle_expiry(
        &self,
        idle_timeout: Option<Duration>,
        protected: &HashSet<(String, String)>,
    ) -> Option<tokio::time::Instant> {
        let timeout = idle_timeout?;
        let now = Utc::now();
        self.inner
            .sessions
            .read()
            .unwrap()
            .iter()
            .filter(|(key, _)| !protected.contains(*key))
            .filter_map(|(_, session)| {
                let last = *session.inner.last_activity.read().unwrap();
                let deadline = last + chrono::Duration::from_std(timeout).ok()?;
                // An already-expired session is processed by the current
                // authoritative cleanup pass. Returning it as an immediate
                // timer would turn a failed or unchanged cleanup into a
                // zero-delay scan loop; later activity supplies the next
                // event-driven wakeup.
                if deadline <= now {
                    return None;
                }
                let remaining = deadline.signed_duration_since(now).to_std().ok()?;
                Some(tokio::time::Instant::now() + remaining)
            })
            .min()
    }

    /// Create a new session and return it.
    #[must_use]
    pub fn create(&self) -> Session {
        self.create_for_user(ANONYMOUS_SESSION_OWNER)
    }

    /// Create a new session owned by `owner_id`.
    #[must_use]
    pub fn create_for_user(&self, owner_id: &str) -> Session {
        let id = Uuid::new_v4().to_string();
        self.create_with_id_for_user(id, owner_id)
    }

    /// Create a new session with a specific ID.
    #[must_use]
    pub fn create_with_id(&self, id: impl Into<String>) -> Session {
        self.create_with_id_for_user(id, ANONYMOUS_SESSION_OWNER)
    }

    /// Create a new session with a specific ID for `owner_id`.
    #[must_use]
    pub fn create_with_id_for_user(&self, id: impl Into<String>, owner_id: &str) -> Session {
        let id = id.into();
        let owner_id = owner_id.to_string();
        let session = Session::new_with_activity(
            id.clone(),
            owner_id.clone(),
            Some(self.inner.activity_generation.clone()),
        );
        let mut guard = self.inner.sessions.write().unwrap();
        guard.insert((owner_id, id), session.clone());
        #[allow(clippy::cast_precision_loss)]
        crate::uar::telemetry::metrics::set_active_sessions(guard.len() as f64);
        drop(guard);
        self.notify_activity();
        session
    }

    /// Get a session by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<Session> {
        self.get_for_user(id, ANONYMOUS_SESSION_OWNER)
    }

    /// Get a session only when it belongs to `owner_id`.
    #[must_use]
    pub fn get_for_user(&self, id: &str, owner_id: &str) -> Option<Session> {
        let guard = self.inner.sessions.read().unwrap();
        guard.get(&(owner_id.to_string(), id.to_string())).cloned()
    }

    /// Get a session by ID, creating it if it doesn't exist.
    #[must_use]
    pub fn get_or_create(&self, id: &str) -> Session {
        self.get_or_create_for_user(id, ANONYMOUS_SESSION_OWNER)
    }

    /// Get or create a session scoped to `owner_id`.
    #[must_use]
    pub fn get_or_create_for_user(&self, id: &str, owner_id: &str) -> Session {
        // Try read-only first
        {
            let guard = self.inner.sessions.read().unwrap();
            if let Some(session) = guard.get(&(owner_id.to_string(), id.to_string())) {
                let session = session.clone();
                session.touch();
                drop(guard);
                self.notify_activity();
                return session;
            }
        }

        // Re-check under the write lock. Concurrent first turns of one new
        // conversation must share a single session; a replaced session would
        // let the later save overwrite the earlier turn.
        let mut guard = self.inner.sessions.write().unwrap();
        let session = guard
            .entry((owner_id.to_string(), id.to_string()))
            .or_insert_with(|| {
                Session::new_with_activity(
                    id.to_string(),
                    owner_id.to_string(),
                    Some(self.inner.activity_generation.clone()),
                )
            })
            .clone();
        session.touch();
        #[allow(clippy::cast_precision_loss)]
        crate::uar::telemetry::metrics::set_active_sessions(guard.len() as f64);
        drop(guard);
        self.notify_activity();
        session
    }

    /// Remove a session by ID.
    pub fn remove(&self, id: &str) -> Option<Session> {
        self.remove_for_user(id, ANONYMOUS_SESSION_OWNER)
    }

    /// Remove a session only when it belongs to `owner_id`.
    pub fn remove_for_user(&self, id: &str, owner_id: &str) -> Option<Session> {
        let mut guard = self.inner.sessions.write().unwrap();
        let removed = guard.remove(&(owner_id.to_string(), id.to_string()));
        #[allow(clippy::cast_precision_loss)]
        crate::uar::telemetry::metrics::set_active_sessions(guard.len() as f64);
        drop(guard);
        if removed.is_some() {
            self.notify_activity();
        }
        removed
    }

    /// Get the number of active sessions.
    #[must_use]
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.inner.sessions.read().unwrap().len()
    }

    /// Check if there are no sessions.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all expired sessions.
    ///
    /// Returns the number of sessions removed.
    #[allow(dead_code)]
    pub fn cleanup_expired(&self) -> usize {
        self.cleanup_expired_with_timeout(DEFAULT_SESSION_TIMEOUT)
    }

    /// Remove sessions that have been inactive longer than the timeout.
    #[allow(dead_code)]
    pub fn cleanup_expired_with_timeout(&self, timeout: Duration) -> usize {
        let mut guard = self.inner.sessions.write().unwrap();
        let before = guard.len();
        guard.retain(|_, session| !session.is_expired_with_timeout(timeout));
        let removed = before - guard.len();
        if removed > 0 {
            #[allow(clippy::cast_precision_loss)]
            crate::uar::telemetry::metrics::set_active_sessions(guard.len() as f64);
            drop(guard);
            self.notify_activity();
            return removed;
        }
        removed
    }

    /// Remove idle or over-cap sessions that have no live run.
    ///
    /// `protected` is captured from the run manager immediately before this
    /// lock is taken. The lock-held expiration and ordering checks make the
    /// store mutation one atomic operation.
    pub(crate) fn cleanup_bounded(
        &self,
        idle_timeout: Option<Duration>,
        max_retained: usize,
        protected: &HashSet<(String, String)>,
    ) -> Vec<(String, String)> {
        let mut guard = self.inner.sessions.write().unwrap();
        let mut removed = Vec::new();

        if let Some(timeout) = idle_timeout {
            let idle = guard
                .iter()
                .filter(|(key, session)| {
                    !protected.contains(*key) && session.is_expired_with_timeout(timeout)
                })
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>();
            for key in idle {
                if guard.remove(&key).is_some() {
                    removed.push(key);
                }
            }
        }

        if max_retained > 0 && guard.len() > max_retained {
            let remove_count = guard.len() - max_retained;
            let mut eligible = guard
                .iter()
                .filter(|(key, _)| !protected.contains(*key))
                .map(|(key, session)| (key.clone(), *session.inner.last_activity.read().unwrap()))
                .collect::<Vec<_>>();
            eligible.sort_by_key(|(_, last_activity)| *last_activity);
            for (key, _) in eligible.into_iter().take(remove_count) {
                if guard.remove(&key).is_some() {
                    removed.push(key);
                }
            }
        }

        if !removed.is_empty() {
            #[allow(clippy::cast_precision_loss)]
            crate::uar::telemetry::metrics::set_active_sessions(guard.len() as f64);
        }
        drop(guard);
        if !removed.is_empty() {
            self.notify_activity();
        }
        removed
    }

    /// List all session IDs.
    #[must_use]
    pub fn list_ids(&self) -> Vec<String> {
        self.list_ids_for_user(ANONYMOUS_SESSION_OWNER)
    }

    /// List session IDs owned by `owner_id`.
    #[must_use]
    pub fn list_ids_for_user(&self, owner_id: &str) -> Vec<String> {
        self.inner
            .sessions
            .read()
            .unwrap()
            .iter()
            .filter(|((stored_owner, _), _)| stored_owner == owner_id)
            .map(|((_, id), _)| id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_identity_and_typed_content_survive_history_roundtrip() {
        let session = Session::new("rich-history".into(), "owner".into());
        let content = MessageContent::Parts {
            content: vec![
                crate::llm::ContentPart::text("original text"),
                crate::llm::ContentPart::image_url("https://example.invalid/original.png"),
            ],
        };
        session.add_message_with_metadata(
            Message {
                role: MessageRole::User,
                content,
                tool_call_id: None,
                tool_calls: None,
            },
            super::super::SessionMessageMetadata::for_run(
                "original-user-message".into(),
                "original-run".into(),
                Some("original-turn".into()),
                super::super::SessionMessageStatus::Running,
            ),
        );
        let restored: Session =
            serde_json::from_value(serde_json::to_value(&session).unwrap()).unwrap();
        let history = restored.history();
        assert_eq!(history[0].message_id, "original-user-message");
        assert_eq!(history[0].run_id.as_deref(), Some("original-run"));
        assert_eq!(
            serde_json::to_value(&history[0].content).unwrap(),
            serde_json::to_value(&session.messages()[0].content).unwrap()
        );
        assert_eq!(history[0].created_at, session.history()[0].created_at);
    }

    #[test]
    fn legacy_history_has_repeatable_ids_without_invented_terminal_or_time() {
        let session = Session::new("legacy-history".into(), "owner".into());
        session.seed_if_empty(&[Message {
            role: MessageRole::Assistant,
            content: MessageContent::text("legacy partial"),
            tool_call_id: None,
            tool_calls: None,
        }]);
        let first = session.history();
        let restored = Session::from_state(session.to_state());
        let second = restored.history();
        assert_eq!(first[0].message_id, second[0].message_id);
        assert_eq!(
            second[0].status,
            super::super::SessionMessageStatus::Unknown
        );
        assert_eq!(second[0].created_at, None);
        assert_eq!(second[0].provenance, "unavailable");
    }

    #[test]
    fn run_handles_keep_distinct_identity_and_actual_terminal_outcomes() {
        let session = Session::new("shared".into(), "owner".into());
        let first = session.with_run_context("run-first", Some("turn-first".into()));
        let second = session.with_run_context("run-second", Some("turn-second".into()));
        first.add_assistant_message("cancelled partial");
        second.add_assistant_message("failed partial");
        first.set_run_history_status("run-first", super::super::SessionMessageStatus::Cancelled);
        second.set_run_history_status("run-second", super::super::SessionMessageStatus::Error);
        let history = Session::from_state(session.to_state()).history();
        assert_eq!(history[0].message_id, "run-run-first-assistant-0");
        assert_eq!(
            history[0].status,
            super::super::SessionMessageStatus::Cancelled
        );
        assert_eq!(history[1].status, super::super::SessionMessageStatus::Error);
        assert_eq!(history[1].client_turn_id.as_deref(), Some("turn-second"));
    }

    #[test]
    fn content_projection_preserves_ids_and_does_not_drop_pending_tool_messages() {
        let session = Session::new("projection".into(), "owner".into());
        session.add_user_message("source");
        session.add_tool_result("pending-tool", "result");
        let before = session.history()[0].message_id.clone();
        let mut messages = session.messages();
        messages[0].content = MessageContent::text("redacted");
        session.project_message_content(messages).unwrap();
        assert_eq!(session.history()[0].message_id, before);
        assert_eq!(
            session.messages()[1].tool_call_id.as_deref(),
            Some("pending-tool")
        );
        assert!(session.project_message_content(Vec::new()).is_err());
        assert_eq!(session.message_count(), 2);
    }

    #[test]
    fn test_session_lifecycle() {
        let session = Session::new("test-123".to_string(), ANONYMOUS_SESSION_OWNER.to_string());

        assert_eq!(session.id(), "test-123");
        assert_eq!(session.message_count(), 0);

        session.add_user_message("Hello");
        assert_eq!(session.message_count(), 1);

        session.add_assistant_message("Hi there!");
        assert_eq!(session.message_count(), 2);

        let messages = session.messages();
        assert_eq!(messages[0].role, MessageRole::User);
        assert_eq!(messages[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_session_store() {
        let store = SessionStore::new();

        assert!(store.is_empty());

        let session = store.create();
        assert_eq!(store.len(), 1);

        let retrieved = store.get(session.id()).unwrap();
        assert_eq!(retrieved.id(), session.id());

        store.remove(session.id());
        assert!(store.is_empty());
    }

    #[test]
    fn session_store_partitions_identical_ids_by_owner() {
        let store = SessionStore::new();
        let alice = store.create_with_id_for_user("shared-id", "alice");
        let bob = store.create_with_id_for_user("shared-id", "bob");
        alice.add_user_message("alice secret");
        bob.add_user_message("bob secret");

        assert_eq!(
            store
                .get_for_user("shared-id", "alice")
                .expect("alice session")
                .messages()[0]
                .content
                .as_text(),
            Some("alice secret")
        );
        assert_eq!(
            store
                .get_for_user("shared-id", "bob")
                .expect("bob session")
                .messages()[0]
                .content
                .as_text(),
            Some("bob secret")
        );
        assert!(store.get("shared-id").is_none());
        assert_eq!(store.list_ids_for_user("alice"), vec!["shared-id"]);
        assert!(store.remove_for_user("shared-id", "bob").is_some());
        assert!(store.get_for_user("shared-id", "alice").is_some());
    }

    #[test]
    fn test_system_prompt() {
        let session = Session::new("test".to_string(), ANONYMOUS_SESSION_OWNER.to_string());

        assert!(session.system_prompt().is_none());

        session.set_system_prompt("You are a helpful assistant.");
        assert_eq!(
            session.system_prompt().unwrap(),
            "You are a helpful assistant."
        );

        let messages = session.messages_with_system();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::System);
    }
}
