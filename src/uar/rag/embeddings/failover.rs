//! Local embedding primary/backup failover.
//!
//! The runtime admits this only for two byte-identical local asset manifests
//! serving the same declared embedding space. A member is retired through an
//! authenticated host control, drains already admitted inference, and only a
//! subsequent real embed error promotes the backup.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use tokio::sync::{RwLock, oneshot, watch};

use crate::{
    config::LocalEmbeddingFailoverConfig,
    uar::{
        domain::knowledge::EmbeddingFingerprint,
        runtime::lifecycle::ResourceScope,
    },
};

use super::{
    BackendProvenance, EmbeddingBackend, EmbeddingConfig, EmbeddingError, build_local_backend,
};

/// Stable names of the only two members in a local embedding failover pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalEmbeddingMember {
    Primary,
    Backup,
}

/// The event that changed the serving member or its admission state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingFailoverTransition {
    Startup,
    PrimaryRetired,
    Failover,
    Recovery,
}

/// Operator-visible state of the local embedding pair.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EmbeddingFailoverStatus {
    pub active_member: LocalEmbeddingMember,
    pub generation: u64,
    pub active_retired: bool,
    pub space: EmbeddingFingerprint,
    pub active_provenance: BackendProvenance,
    pub primary_provenance: BackendProvenance,
    pub backup_provenance: BackendProvenance,
    pub last_transition: EmbeddingFailoverTransition,
    pub last_error: Option<String>,
}

/// Generation-ordered failover event. Subscribers must check `generation`
/// before accepting a recovery result from an earlier member lifetime.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EmbeddingFailoverEvent {
    pub generation: u64,
    pub transition: EmbeddingFailoverTransition,
    pub status: EmbeddingFailoverStatus,
}

#[derive(Debug)]
struct MemberAdmission {
    retired: AtomicBool,
    active: RwLock<()>,
}

impl MemberAdmission {
    fn new() -> Self {
        Self {
            retired: AtomicBool::new(false),
            active: RwLock::new(()),
        }
    }

    async fn enter(&self, member: LocalEmbeddingMember) -> Result<tokio::sync::RwLockReadGuard<'_, ()>, EmbeddingError> {
        if self.retired.load(Ordering::Acquire) {
            return Err(EmbeddingError::BackendUnavailable(format!(
                "local embedding {member:?} was retired by the trusted host"
            )));
        }
        let guard = self.active.read().await;
        if self.retired.load(Ordering::Acquire) {
            drop(guard);
            return Err(EmbeddingError::BackendUnavailable(format!(
                "local embedding {member:?} was retired by the trusted host"
            )));
        }
        Ok(guard)
    }

    async fn retire(&self) {
        self.retired.store(true, Ordering::Release);
        let drain = self.active.write().await;
        drop(drain);
    }

    fn is_retired(&self) -> bool {
        self.retired.load(Ordering::Acquire)
    }
}

#[derive(Clone)]
struct LocalMember {
    role: LocalEmbeddingMember,
    backend: Arc<dyn EmbeddingBackend>,
    provenance: BackendProvenance,
    admission: Arc<MemberAdmission>,
}

impl LocalMember {
    async fn embed(
        &self,
        texts: &[&str],
        scope: &ResourceScope,
    ) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let texts = texts.iter().map(|text| (*text).to_owned()).collect::<Vec<_>>();
        let member = self.clone();
        let (reply, receiver) = oneshot::channel();
        scope
            .spawn("local-embedding-inference", async move {
                let result = member.embed_admitted(texts).await;
                let _ = reply.send(result);
                Ok(())
            })
            .map_err(|error| {
                EmbeddingError::BackendUnavailable(format!(
                    "local embedding inference was not admitted by the host: {error}"
                ))
            })?;
        receiver.await.map_err(|_| {
            EmbeddingError::BackendUnavailable(
                "local embedding inference owner ended before replying".to_string(),
            )
        })?
    }

    async fn embed_admitted(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let admission = self.admission.enter(self.role).await?;
        let references = texts.iter().map(String::as_str).collect::<Vec<_>>();
        let output = self.backend.embed(&references).await;
        drop(admission);
        output
    }

    async fn retire(&self) -> Result<(), EmbeddingError> {
        self.admission.retire().await;
        self.backend.shutdown().await.map_err(|error| {
            EmbeddingError::BackendUnavailable(format!(
                "local embedding {:?} shutdown failed after admission drained: {error}",
                self.role
            ))
        })
    }
}

#[derive(Clone)]
struct FailoverState {
    primary: LocalMember,
    backup: LocalMember,
    active: LocalEmbeddingMember,
    generation: u64,
    transition: EmbeddingFailoverTransition,
    last_error: Option<String>,
}

/// A compatibility-pinned local primary/backup embedding backend.
///
/// It exposes the original member fingerprint so existing knowledge bases keep
/// their durable `{backend, model, dimension}` binding. The backup is accepted
/// only after the identity and exact local asset manifest match the primary.
pub struct LocalEmbeddingFailover {
    primary_config: EmbeddingConfig,
    scope: ResourceScope,
    fingerprint: EmbeddingFingerprint,
    state: Mutex<FailoverState>,
    events: watch::Sender<EmbeddingFailoverEvent>,
}

impl std::fmt::Debug for LocalEmbeddingFailover {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LocalEmbeddingFailover")
            .field("status", &self.status())
            .finish()
    }
}

impl LocalEmbeddingFailover {
    /// Build a local primary/backup pair from the configured primary and its
    /// explicit backup member. Both members are constructed before publication.
    pub async fn build(
        primary: EmbeddingConfig,
        failover: &LocalEmbeddingFailoverConfig,
        scope: ResourceScope,
    ) -> Result<Arc<Self>, EmbeddingError> {
        Self::ensure_local_primary(&primary)?;
        let primary_config = primary.strict_local_member();
        let backup_config = failover.backup.clone();
        let primary_member = Self::build_member(
            LocalEmbeddingMember::Primary,
            primary_config.clone(),
            scope.clone(),
        )
        .await?;
        let backup_member = match Self::build_member(
            LocalEmbeddingMember::Backup,
            EmbeddingConfig::from_local_member(&backup_config),
            scope.clone(),
        )
        .await
        {
            Ok(member) => member,
            Err(error) => {
                return match primary_member.retire().await {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(Self::with_cleanup_failure(
                        "backup construction failed",
                        error,
                        cleanup,
                    )),
                };
            }
        };
        let fingerprint = primary_member.backend.fingerprint();
        if let Err(error) = Self::ensure_compatible(&fingerprint, &primary_member, &backup_member) {
            return match Self::retire_pair(&primary_member, &backup_member).await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(Self::with_cleanup_failure(
                    "local embedding pair compatibility validation failed",
                    error,
                    cleanup,
                )),
            };
        }

        let state = FailoverState {
            primary: primary_member,
            backup: backup_member,
            active: LocalEmbeddingMember::Primary,
            generation: 1,
            transition: EmbeddingFailoverTransition::Startup,
            last_error: None,
        };
        let initial_status = Self::status_from_state(&state, &fingerprint);
        let (events, _) = watch::channel(EmbeddingFailoverEvent {
            generation: 1,
            transition: EmbeddingFailoverTransition::Startup,
            status: initial_status,
        });
        Ok(Arc::new(Self {
            primary_config,
            scope,
            fingerprint,
            state: Mutex::new(state),
            events,
        }))
    }

    fn ensure_local_primary(config: &EmbeddingConfig) -> Result<(), EmbeddingError> {
        if config.backend != "fastembed"
            || config.api_key.is_some()
            || config.api_key_env.is_some()
            || config.base_url.is_some()
        {
            return Err(EmbeddingError::InvalidInput(
                "local embedding failover requires credential-free fastembed primary assets".to_string(),
            ));
        }
        Ok(())
    }

    async fn build_member(
        role: LocalEmbeddingMember,
        config: EmbeddingConfig,
        scope: ResourceScope,
    ) -> Result<LocalMember, EmbeddingError> {
        let backend = build_local_backend(&config, scope)?;
        let candidate = async {
            if backend.backend_name() != config.backend
                || backend.model_id() != config.model
                || backend.vector_dimension() != config.vector_dimension
            {
                return Err(EmbeddingError::BackendUnavailable(format!(
                    "local embedding {role:?} does not honor its configured backend/model/dimension"
                )));
            }
            let provenance = backend.provenance().ok_or_else(|| {
                EmbeddingError::BackendUnavailable(format!(
                    "local embedding {role:?} did not report asset provenance"
                ))
            })?;
            let readiness = backend.embed(&["embedding failover readiness"]).await?;
            let Some(vector) = readiness.first() else {
                return Err(EmbeddingError::EmptyResponse);
            };
            if readiness.len() != 1 || vector.len() != config.vector_dimension {
                return Err(EmbeddingError::DimensionMismatch {
                    expected: config.vector_dimension,
                    actual: vector.len(),
                });
            }
            Ok(LocalMember {
                role,
                backend: Arc::clone(&backend),
                provenance,
                admission: Arc::new(MemberAdmission::new()),
            })
        }
        .await;
        match candidate {
            Ok(member) => Ok(member),
            Err(error) => match backend.shutdown().await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(Self::with_cleanup_failure(
                    "unpublished local embedding member initialization failed",
                    error,
                    cleanup,
                )),
            },
        }
    }

    fn with_cleanup_failure(
        context: &str,
        original: EmbeddingError,
        cleanup: EmbeddingError,
    ) -> EmbeddingError {
        EmbeddingError::BackendUnavailable(format!(
            "{context}: {original}; cleanup also failed: {cleanup}"
        ))
    }

    fn record_cleanup_failure(&self, context: &str, error: &EmbeddingError) {
        self.scope
            .record_failure(format!("local embedding {context}: {error}"));
    }

    async fn retire_pair(primary: &LocalMember, backup: &LocalMember) -> Result<(), EmbeddingError> {
        let mut failures = Vec::new();
        if let Err(error) = primary.retire().await {
            failures.push(error.to_string());
        }
        if let Err(error) = backup.retire().await {
            failures.push(error.to_string());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(EmbeddingError::BackendUnavailable(format!(
                "local embedding member cleanup failed: {}",
                failures.join("; ")
            )))
        }
    }

    fn ensure_compatible(
        primary: &EmbeddingFingerprint,
        primary_member: &LocalMember,
        backup_member: &LocalMember,
    ) -> Result<(), EmbeddingError> {
        let backup = backup_member.backend.fingerprint();
        if &backup != primary {
            return Err(EmbeddingError::InvalidInput(format!(
                "local embedding backup space {backup} does not match primary space {primary}"
            )));
        }
        if backup_member.provenance.asset_manifest_sha256
            != primary_member.provenance.asset_manifest_sha256
        {
            return Err(EmbeddingError::InvalidInput(
                "local embedding backup assets differ from the primary manifest; refusing to blend spaces"
                    .to_string(),
            ));
        }
        Ok(())
    }

    fn member(state: &FailoverState) -> LocalMember {
        match state.active {
            LocalEmbeddingMember::Primary => state.primary.clone(),
            LocalEmbeddingMember::Backup => state.backup.clone(),
        }
    }

    fn status_from_state(
        state: &FailoverState,
        fingerprint: &EmbeddingFingerprint,
    ) -> EmbeddingFailoverStatus {
        let active = Self::member(state);
        EmbeddingFailoverStatus {
            active_member: state.active,
            generation: state.generation,
            active_retired: active.admission.is_retired(),
            space: fingerprint.clone(),
            active_provenance: active.provenance,
            primary_provenance: state.primary.provenance.clone(),
            backup_provenance: state.backup.provenance.clone(),
            last_transition: state.transition,
            last_error: state.last_error.clone(),
        }
    }

    fn publish_locked(&self, state: &FailoverState) {
        let status = Self::status_from_state(state, &self.fingerprint);
        self.events.send_replace(EmbeddingFailoverEvent {
            generation: state.generation,
            transition: state.transition,
            status,
        });
    }

    /// Observe state transitions. A receiver must compare the event generation
    /// to the generation it initiated before treating recovery as successful.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<EmbeddingFailoverEvent> {
        self.events.subscribe()
    }

    /// Return the truthfully active member and its generation.
    #[must_use]
    pub fn status(&self) -> EmbeddingFailoverStatus {
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::status_from_state(&state, &self.fingerprint)
    }

    /// Retire the active primary through the trusted host. This closes new
    /// member admission and awaits every accepted inference before returning;
    /// the next embed receives a real retirement error and atomically promotes
    /// the compatible backup.
    pub async fn retire_primary(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> Result<EmbeddingFailoverStatus, EmbeddingError> {
        self.submit_retire_primary(expected_generation)?
            .await
            .map_err(|_| {
                EmbeddingError::BackendUnavailable(
                    "embedding retirement owner ended before replying".to_string(),
                )
            })?
    }

    async fn retire_primary_inner(
        &self,
        expected_generation: u64,
    ) -> Result<EmbeddingFailoverStatus, EmbeddingError> {
        let primary = {
            let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.generation != expected_generation {
                return Err(EmbeddingError::InvalidInput(format!(
                    "stale embedding failover generation {expected_generation}; active generation is {}",
                    state.generation
                )));
            }
            if state.active != LocalEmbeddingMember::Primary {
                return Err(EmbeddingError::InvalidInput(
                    "the primary is not active and cannot be retired".to_string(),
                ));
            }
            state.primary.clone()
        };
        let cleanup = primary.retire().await;
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.active == LocalEmbeddingMember::Primary && state.generation == expected_generation {
            state.transition = EmbeddingFailoverTransition::PrimaryRetired;
            state.last_error = Some(match &cleanup {
                Ok(()) => "primary retired by trusted host; next embed promotes backup".to_string(),
                Err(error) => format!("primary admission retired but native cleanup failed: {error}"),
            });
            self.publish_locked(&state);
        }
        if let Err(error) = &cleanup {
            self.record_cleanup_failure("primary retirement cleanup failed", error);
        }
        cleanup?;
        Ok(Self::status_from_state(&state, &self.fingerprint))
    }

    async fn promote_backup_after_primary_error(
        &self,
        observed_generation: u64,
        error: &EmbeddingError,
    ) -> Result<LocalMember, EmbeddingError> {
        let primary = {
            let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.active != LocalEmbeddingMember::Primary || state.generation != observed_generation {
                return Ok(Self::member(&state));
            }
            state.primary.clone()
        };
        if let Err(error) = primary.retire().await {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.active == LocalEmbeddingMember::Primary && state.generation == observed_generation {
                state.transition = EmbeddingFailoverTransition::PrimaryRetired;
                state.last_error = Some(format!(
                    "primary admission retired after inference error, but native cleanup failed: {error}"
                ));
                self.publish_locked(&state);
            }
            self.record_cleanup_failure("primary failover cleanup failed", &error);
            return Err(error);
        }
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.active == LocalEmbeddingMember::Primary && state.generation == observed_generation {
            state.active = LocalEmbeddingMember::Backup;
            state.generation = state.generation.saturating_add(1);
            state.transition = EmbeddingFailoverTransition::Failover;
            state.last_error = Some(error.to_string());
            self.publish_locked(&state);
        }
        Ok(Self::member(&state))
    }

    /// Rebuild and activate the primary after an operator has repaired it.
    /// Recovery is generation-checked and leaves the validated backup live as
    /// the next standby, so a later primary fault can fail over again.
    pub async fn recover_primary(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> Result<EmbeddingFailoverStatus, EmbeddingError> {
        self.submit_recover_primary(expected_generation)?
            .await
            .map_err(|_| {
                EmbeddingError::BackendUnavailable(
                    "embedding recovery owner ended before replying".to_string(),
                )
            })?
    }

    async fn recover_primary_inner(
        &self,
        expected_generation: u64,
    ) -> Result<EmbeddingFailoverStatus, EmbeddingError> {
        let candidate = Self::build_member(
            LocalEmbeddingMember::Primary,
            self.primary_config.clone(),
            self.scope.clone(),
        )
        .await?;
        match {
            let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.generation != expected_generation {
                Err(EmbeddingError::InvalidInput(format!(
                    "stale embedding failover generation {expected_generation}; active generation is {}",
                    state.generation
                )))
            } else if state.active != LocalEmbeddingMember::Backup {
                Err(EmbeddingError::InvalidInput(
                    "recovery requires the compatible backup to be active".to_string(),
                ))
            } else if let Err(error) = Self::ensure_compatible(&self.fingerprint, &candidate, &state.backup) {
                Err(error)
            } else {
                Ok(())
            }
        } {
            Ok(()) => (),
            Err(error) => {
                return match candidate.retire().await {
                    Ok(()) => Err(error),
                    Err(cleanup) => Err(Self::with_cleanup_failure(
                        "unpublished primary recovery candidate was rejected",
                        error,
                        cleanup,
                    )),
                };
            }
        };
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.generation != expected_generation || state.active != LocalEmbeddingMember::Backup {
            let error = EmbeddingError::InvalidInput(
            "embedding failover state changed before recovery could publish the primary".to_string(),
            );
            drop(state);
            let candidate_cleanup = candidate.retire().await;
            if let Err(cleanup) = &candidate_cleanup {
                self.record_cleanup_failure(
                    "primary recovery candidate cleanup failed after a stale publish check",
                    cleanup,
                );
            }
            return match candidate_cleanup {
                Ok(()) => Err(error),
                Err(cleanup) => Err(Self::with_cleanup_failure(
                    "primary recovery candidate could not be published",
                    error,
                    cleanup,
                )),
            };
        }
        state.primary = candidate;
        state.active = LocalEmbeddingMember::Primary;
        state.generation = state.generation.saturating_add(1);
        state.transition = EmbeddingFailoverTransition::Recovery;
        state.last_error = None;
        self.publish_locked(&state);
        Ok(Self::status_from_state(&state, &self.fingerprint))
    }

    /// Finalize both members after the host scope has stopped new control and
    /// inference work. Dropping cached engines leaves retained facades inert.
    pub async fn shutdown(&self) -> Result<(), EmbeddingError> {
        let (primary, backup) = {
            let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            (state.primary.clone(), state.backup.clone())
        };
        Self::retire_pair(&primary, &backup).await
    }

    /// Submit retirement into the host-owned scope. The command continues
    /// draining after an HTTP client abandons its reply wait.
    pub fn submit_retire_primary(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> Result<oneshot::Receiver<Result<EmbeddingFailoverStatus, EmbeddingError>>, EmbeddingError> {
        let (reply, receiver) = oneshot::channel();
        let failover = Arc::clone(self);
        self.scope
            .spawn("embedding-retire-primary", async move {
                let result = failover.retire_primary_inner(expected_generation).await;
                let _ = reply.send(result);
                Ok(())
            })
            .map_err(|error| {
                EmbeddingError::BackendUnavailable(format!(
                    "embedding retirement was not admitted by the host: {error}"
                ))
            })?;
        Ok(receiver)
    }

    /// Submit primary recovery into the host-owned scope. The generation CAS
    /// is evaluated by the retained task, not by a cancellable HTTP waiter.
    pub fn submit_recover_primary(
        self: &Arc<Self>,
        expected_generation: u64,
    ) -> Result<oneshot::Receiver<Result<EmbeddingFailoverStatus, EmbeddingError>>, EmbeddingError> {
        let (reply, receiver) = oneshot::channel();
        let failover = Arc::clone(self);
        self.scope
            .spawn("embedding-recover-primary", async move {
                let result = failover.recover_primary_inner(expected_generation).await;
                let _ = reply.send(result);
                Ok(())
            })
            .map_err(|error| {
                EmbeddingError::BackendUnavailable(format!(
                    "embedding recovery was not admitted by the host: {error}"
                ))
            })?;
        Ok(receiver)
    }
}

#[async_trait]
impl EmbeddingBackend for LocalEmbeddingFailover {
    fn backend_name(&self) -> &str {
        &self.fingerprint.backend
    }

    fn vector_dimension(&self) -> usize {
        self.fingerprint.dimension
    }

    fn model_id(&self) -> &str {
        &self.fingerprint.model
    }

    fn provenance(&self) -> Option<BackendProvenance> {
        Some(self.status().active_provenance)
    }

    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let (member, generation) = {
            let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            (Self::member(&state), state.generation)
        };
        match member.embed(texts, &self.scope).await {
            Ok(vectors) => Ok(vectors),
            Err(error) if member.role == LocalEmbeddingMember::Primary => {
                let backup = self
                    .promote_backup_after_primary_error(generation, &error)
                    .await?;
                backup.embed(texts, &self.scope).await
            }
            Err(error) => Err(error),
        }
    }
}
