//! Boot-effective governance posture and live enforcement state.
//!
//! This module owns the network/authentication trust-boundary proof used to
//! decide whether an operator may disable tool governance. It deliberately
//! keeps mutation authority separate from the read-only gate and status
//! handles consumed by runtime and API code.

use std::{
    collections::{BTreeMap, BTreeSet},
    net::SocketAddr,
    sync::{Arc, Once, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Boot/runtime phase of the governance authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceRuntimePhase {
    /// Boot posture or durable preference has not been finalized. Gates On.
    Initializing,
    /// Governance is finalized and enforced.
    On,
    /// Governance is finalized and bypassed for the eligible local posture.
    Off,
}

/// Operator-facing effective state derived from one coherent snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceEffectiveState {
    /// Boot is incomplete or a projection could not be trusted.
    Unknown,
    /// Governance gates On and the boot posture cannot be disabled.
    Required,
    /// Governance is eligible to be disabled but is currently enforced.
    On,
    /// Governance is disabled in a verified local-only posture.
    Off,
}

/// Closed reason-code vocabulary for mandatory or unavailable states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceStatusReason {
    InitializationIncomplete,
    ConfiguredHostNotAllowed,
    AuthenticationUnverified,
    JwtRequired,
    IngressInventoryUnsealed,
    IngressProofMissing,
    BoundIngressNotLoopback,
    PersistenceUnavailable,
}

/// Coherent status consumed by the tool gate and settings surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceRuntimeSnapshot {
    pub boot_instance_id: String,
    pub revision: u64,
    pub phase: GovernanceRuntimePhase,
    pub effective_state: GovernanceEffectiveState,
    pub effective_enabled: bool,
    pub may_disable: bool,
    pub mutation_available: bool,
    pub configured_host: String,
    pub bound_addresses: Vec<SocketAddr>,
    pub jwt_required: Option<bool>,
    pub reasons: Vec<GovernanceStatusReason>,
}

impl GovernanceRuntimeSnapshot {
    /// Reject combinations that cannot be produced by the runtime authority.
    pub fn validate(&self) -> Result<(), &'static str> {
        let persistence_unavailable = self
            .reasons
            .contains(&GovernanceStatusReason::PersistenceUnavailable);
        let has_mandatory_reason = self.reasons.iter().any(|reason| {
            matches!(
                reason,
                GovernanceStatusReason::ConfiguredHostNotAllowed
                    | GovernanceStatusReason::AuthenticationUnverified
                    | GovernanceStatusReason::JwtRequired
                    | GovernanceStatusReason::IngressInventoryUnsealed
                    | GovernanceStatusReason::IngressProofMissing
                    | GovernanceStatusReason::BoundIngressNotLoopback
            )
        });
        match self.effective_state {
            GovernanceEffectiveState::Unknown
                if self.phase == GovernanceRuntimePhase::Initializing
                    && self.effective_enabled
                    && !self.mutation_available => {}
            GovernanceEffectiveState::Required
                if self.phase == GovernanceRuntimePhase::On
                    && self.effective_enabled
                    && !self.may_disable
                    && has_mandatory_reason
                    && !self
                        .reasons
                        .contains(&GovernanceStatusReason::InitializationIncomplete)
                    && self.mutation_available != persistence_unavailable => {}
            GovernanceEffectiveState::On
                if self.phase == GovernanceRuntimePhase::On
                    && self.effective_enabled
                    && self.may_disable
                    && ((self.mutation_available && self.reasons.is_empty())
                        || (!self.mutation_available
                            && self.reasons
                                == [GovernanceStatusReason::PersistenceUnavailable])) => {}
            GovernanceEffectiveState::Off
                if self.phase == GovernanceRuntimePhase::Off
                    && !self.effective_enabled
                    && self.may_disable
                    && self.mutation_available
                    && self.reasons.is_empty() => {}
            _ => return Err("incoherent governance runtime snapshot"),
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovernancePreferenceWrite {
    None,
    Seed(bool),
    Normalize(bool),
}

/// Durable action required before the runtime may finalize governance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernancePreferencePlan {
    pub write: GovernancePreferenceWrite,
    pub target_enabled: bool,
    boot_instance_id: String,
    sealed_revision: u64,
}

/// Result of finalizing a durable preference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceFinalization {
    pub snapshot: GovernanceRuntimeSnapshot,
    pub warning_emitted: bool,
}

/// Opaque proof returned only after a declared ingress records its bound address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngressRegistrationProof {
    ingress_id: String,
    boot_instance_id: String,
    proof_id: Uuid,
}

/// Admission capability for one registered ingress.
#[derive(Debug, Clone)]
pub struct GovernanceAdmissionToken {
    shared: Arc<Shared>,
    ingress_id: String,
    proof_id: Uuid,
}

impl GovernanceAdmissionToken {
    #[must_use]
    pub fn ingress_id(&self) -> &str {
        &self.ingress_id
    }

    /// Whether this ingress may enter its serve/admission path.
    #[must_use]
    pub fn is_active(&self) -> bool {
        let state = read_state(&self.shared);
        state.admission_active
            && state.phase != GovernanceRuntimePhase::Initializing
            && state
                .registrations
                .get(&self.ingress_id)
                .is_some_and(|registration| registration.proof_id == self.proof_id)
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GovernanceRuntimeError {
    #[error("governance ingress inventory is already sealed")]
    InventorySealed,
    #[error("governance ingress inventory is not sealed")]
    InventoryNotSealed,
    #[error("governance ingress '{0}' was not declared")]
    UndeclaredIngress(String),
    #[error("governance ingress '{0}' was declared more than once")]
    DuplicateIngress(String),
    #[error("governance ingress '{0}' registered more than once")]
    DuplicateRegistration(String),
    #[error("no tool-capable ingress was declared")]
    NoDeclaredIngress,
    #[error("installed authentication mode was not recorded")]
    AuthenticationUnverified,
    #[error("one or more declared ingress registration proofs are missing or invalid")]
    InvalidIngressProofs,
    #[error("governance preference plan is stale or belongs to another boot")]
    StalePreferencePlan,
    #[error("governance runtime has already been finalized")]
    AlreadyFinalized,
    #[error("governance runtime is not finalized")]
    NotFinalized,
    #[error("governance mutation is unavailable")]
    MutationUnavailable,
    #[error("governance is mandatory for the active boot posture")]
    GovernanceRequired,
}

#[derive(Debug, Clone)]
struct Registration {
    address: SocketAddr,
    proof_id: Uuid,
}

#[derive(Debug)]
struct State {
    boot_instance_id: String,
    revision: u64,
    phase: GovernanceRuntimePhase,
    effective_state: GovernanceEffectiveState,
    effective_enabled: bool,
    may_disable: bool,
    mutation_available: bool,
    configured_host: String,
    jwt_required: Option<bool>,
    declared_ingresses: BTreeSet<String>,
    registrations: BTreeMap<String, Registration>,
    sealed: bool,
    admission_active: bool,
    reasons: BTreeSet<GovernanceStatusReason>,
}

impl State {
    fn snapshot(&self) -> GovernanceRuntimeSnapshot {
        GovernanceRuntimeSnapshot {
            boot_instance_id: self.boot_instance_id.clone(),
            revision: self.revision,
            phase: self.phase,
            effective_state: self.effective_state,
            effective_enabled: self.effective_enabled,
            may_disable: self.may_disable,
            mutation_available: self.mutation_available,
            configured_host: self.configured_host.clone(),
            bound_addresses: self
                .registrations
                .values()
                .map(|registration| registration.address)
                .collect(),
            jwt_required: self.jwt_required,
            reasons: self.reasons.iter().copied().collect(),
        }
    }
}

#[derive(Debug)]
struct Shared {
    state: RwLock<State>,
    inactive_warning: Once,
}

fn read_state(shared: &Shared) -> RwLockReadGuard<'_, State> {
    shared
        .state
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_state(shared: &Shared) -> RwLockWriteGuard<'_, State> {
    shared
        .state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Trusted-host mutation handle. Do not expose this to agent kernels.
#[derive(Debug, Clone)]
pub struct GovernanceMutationHandle {
    shared: Arc<Shared>,
}

/// Read-only hot-path gate handle.
#[derive(Debug, Clone)]
pub struct GovernanceGateHandle {
    shared: Arc<Shared>,
}

/// Read-only API/status handle.
#[derive(Debug, Clone)]
pub struct GovernanceStatusHandle {
    shared: Arc<Shared>,
}

/// Construct separate authority handles for one process boot.
#[must_use]
pub fn governance_runtime_handles(
    configured_host: impl Into<String>,
) -> (
    GovernanceMutationHandle,
    GovernanceGateHandle,
    GovernanceStatusHandle,
) {
    let shared = Arc::new(Shared {
        state: RwLock::new(State {
            boot_instance_id: Uuid::new_v4().to_string(),
            revision: 0,
            phase: GovernanceRuntimePhase::Initializing,
            effective_state: GovernanceEffectiveState::Unknown,
            effective_enabled: true,
            may_disable: false,
            mutation_available: false,
            configured_host: configured_host.into(),
            jwt_required: None,
            declared_ingresses: BTreeSet::new(),
            registrations: BTreeMap::new(),
            sealed: false,
            admission_active: false,
            reasons: BTreeSet::from([GovernanceStatusReason::InitializationIncomplete]),
        }),
        inactive_warning: Once::new(),
    });
    (
        GovernanceMutationHandle {
            shared: Arc::clone(&shared),
        },
        GovernanceGateHandle {
            shared: Arc::clone(&shared),
        },
        GovernanceStatusHandle { shared },
    )
}

impl GovernanceGateHandle {
    /// Initializing and unavailable states intentionally return enabled.
    #[must_use]
    pub fn effective_enabled(&self) -> bool {
        read_state(&self.shared).effective_enabled
    }

    #[must_use]
    pub fn snapshot(&self) -> GovernanceRuntimeSnapshot {
        read_state(&self.shared).snapshot()
    }
}

impl GovernanceStatusHandle {
    #[must_use]
    pub fn snapshot(&self) -> GovernanceRuntimeSnapshot {
        read_state(&self.shared).snapshot()
    }
}

impl GovernanceMutationHandle {
    /// Record the authentication mode actually installed for this boot.
    pub fn record_installed_authentication(&self, jwt_required: bool) {
        let mut state = write_state(&self.shared);
        state.jwt_required = Some(jwt_required);
        state.revision = state.revision.saturating_add(1);
    }

    /// Declare a tool-capable network ingress before binding it.
    pub fn declare_ingress(
        &self,
        ingress_id: impl Into<String>,
    ) -> Result<(), GovernanceRuntimeError> {
        let mut state = write_state(&self.shared);
        if state.sealed {
            return Err(GovernanceRuntimeError::InventorySealed);
        }
        let ingress_id = ingress_id.into();
        if !state.declared_ingresses.insert(ingress_id.clone()) {
            return Err(GovernanceRuntimeError::DuplicateIngress(ingress_id));
        }
        state.revision = state.revision.saturating_add(1);
        Ok(())
    }

    /// Register the successfully bound address for a declared ingress.
    pub fn register_bound_ingress(
        &self,
        ingress_id: impl Into<String>,
        address: SocketAddr,
    ) -> Result<IngressRegistrationProof, GovernanceRuntimeError> {
        let mut state = write_state(&self.shared);
        if state.sealed {
            return Err(GovernanceRuntimeError::InventorySealed);
        }
        let ingress_id = ingress_id.into();
        if !state.declared_ingresses.contains(&ingress_id) {
            return Err(GovernanceRuntimeError::UndeclaredIngress(ingress_id));
        }
        if state.registrations.contains_key(&ingress_id) {
            return Err(GovernanceRuntimeError::DuplicateRegistration(ingress_id));
        }
        let proof_id = Uuid::new_v4();
        state
            .registrations
            .insert(ingress_id.clone(), Registration { address, proof_id });
        state.revision = state.revision.saturating_add(1);
        Ok(IngressRegistrationProof {
            ingress_id,
            boot_instance_id: state.boot_instance_id.clone(),
            proof_id,
        })
    }

    /// Seal the complete ingress inventory and derive boot eligibility.
    pub fn seal_ingress_inventory(
        &self,
        proofs: &[IngressRegistrationProof],
    ) -> Result<Vec<GovernanceAdmissionToken>, GovernanceRuntimeError> {
        let mut state = write_state(&self.shared);
        if state.sealed {
            return Err(GovernanceRuntimeError::InventorySealed);
        }
        if state.declared_ingresses.is_empty() {
            return Err(GovernanceRuntimeError::NoDeclaredIngress);
        }
        let Some(jwt_required) = state.jwt_required else {
            return Err(GovernanceRuntimeError::AuthenticationUnverified);
        };
        let valid_proofs = proofs
            .iter()
            .filter(|proof| proof.boot_instance_id == state.boot_instance_id)
            .map(|proof| (proof.ingress_id.as_str(), proof.proof_id))
            .collect::<BTreeSet<_>>();
        let expected_proofs = state
            .registrations
            .iter()
            .map(|(ingress_id, registration)| (ingress_id.as_str(), registration.proof_id))
            .collect::<BTreeSet<_>>();
        if state.declared_ingresses.len() != state.registrations.len()
            || valid_proofs != expected_proofs
        {
            return Err(GovernanceRuntimeError::InvalidIngressProofs);
        }

        state.sealed = true;
        state.reasons.clear();
        if !is_allowed_configured_host(&state.configured_host) {
            state
                .reasons
                .insert(GovernanceStatusReason::ConfiguredHostNotAllowed);
        }
        if jwt_required {
            state.reasons.insert(GovernanceStatusReason::JwtRequired);
        }
        if state
            .registrations
            .values()
            .any(|registration| !registration.address.ip().is_loopback())
        {
            state
                .reasons
                .insert(GovernanceStatusReason::BoundIngressNotLoopback);
        }
        state.may_disable = state.reasons.is_empty();
        state.revision = state.revision.saturating_add(1);

        Ok(state
            .registrations
            .iter()
            .map(|(ingress_id, registration)| GovernanceAdmissionToken {
                shared: Arc::clone(&self.shared),
                ingress_id: ingress_id.clone(),
                proof_id: registration.proof_id,
            })
            .collect())
    }

    /// Decide whether durable storage must seed or normalize the preference.
    pub fn preference_plan(
        &self,
        persisted_enabled: Option<bool>,
    ) -> Result<GovernancePreferencePlan, GovernanceRuntimeError> {
        let state = read_state(&self.shared);
        if !state.sealed {
            return Err(GovernanceRuntimeError::InventoryNotSealed);
        }
        let (write, target_enabled) = match (state.may_disable, persisted_enabled) {
            (true, Some(value)) => (GovernancePreferenceWrite::None, value),
            (true, None) => (GovernancePreferenceWrite::Seed(false), false),
            (false, Some(false)) => (GovernancePreferenceWrite::Normalize(true), true),
            (false, Some(true)) => (GovernancePreferenceWrite::None, true),
            (false, None) => (GovernancePreferenceWrite::Seed(true), true),
        };
        Ok(GovernancePreferencePlan {
            write,
            target_enabled,
            boot_instance_id: state.boot_instance_id.clone(),
            sealed_revision: state.revision,
        })
    }

    /// Finalize from a durable preference plan after its required write succeeds.
    pub fn finalize_preference(
        &self,
        plan: &GovernancePreferencePlan,
    ) -> Result<GovernanceFinalization, GovernanceRuntimeError> {
        {
            let mut state = write_state(&self.shared);
            if state.phase != GovernanceRuntimePhase::Initializing {
                return Err(GovernanceRuntimeError::AlreadyFinalized);
            }
            if !state.sealed
                || state.boot_instance_id != plan.boot_instance_id
                || state.revision != plan.sealed_revision
            {
                return Err(GovernanceRuntimeError::StalePreferencePlan);
            }
            state.mutation_available = true;
            state.effective_enabled = !state.may_disable || plan.target_enabled;
            state.phase = if state.effective_enabled {
                GovernanceRuntimePhase::On
            } else {
                GovernanceRuntimePhase::Off
            };
            state.effective_state = if !state.may_disable {
                GovernanceEffectiveState::Required
            } else if state.effective_enabled {
                GovernanceEffectiveState::On
            } else {
                GovernanceEffectiveState::Off
            };
            state.revision = state.revision.saturating_add(1);
        }
        let warning_emitted = self.emit_inactive_warning_once();
        Ok(GovernanceFinalization {
            snapshot: read_state(&self.shared).snapshot(),
            warning_emitted,
        })
    }

    /// Finalize fail-closed when durable preference resolution is unavailable.
    pub fn finalize_mutation_unavailable(
        &self,
    ) -> Result<GovernanceFinalization, GovernanceRuntimeError> {
        {
            let mut state = write_state(&self.shared);
            if state.phase != GovernanceRuntimePhase::Initializing {
                return Err(GovernanceRuntimeError::AlreadyFinalized);
            }
            state.phase = GovernanceRuntimePhase::On;
            state.effective_state = if state.may_disable {
                GovernanceEffectiveState::On
            } else {
                GovernanceEffectiveState::Required
            };
            state.effective_enabled = true;
            state.mutation_available = false;
            state
                .reasons
                .insert(GovernanceStatusReason::PersistenceUnavailable);
            state.revision = state.revision.saturating_add(1);
        }
        Ok(GovernanceFinalization {
            snapshot: read_state(&self.shared).snapshot(),
            warning_emitted: false,
        })
    }

    /// Publish a preference only after the trusted settings layer commits it.
    pub fn publish_committed_preference(
        &self,
        enabled: bool,
    ) -> Result<GovernanceFinalization, GovernanceRuntimeError> {
        {
            let mut state = write_state(&self.shared);
            if state.phase == GovernanceRuntimePhase::Initializing {
                return Err(GovernanceRuntimeError::NotFinalized);
            }
            if !state.mutation_available {
                return Err(GovernanceRuntimeError::MutationUnavailable);
            }
            if !enabled && !state.may_disable {
                return Err(GovernanceRuntimeError::GovernanceRequired);
            }
            state.effective_enabled = enabled;
            state.phase = if enabled {
                GovernanceRuntimePhase::On
            } else {
                GovernanceRuntimePhase::Off
            };
            state.effective_state = if !state.may_disable {
                GovernanceEffectiveState::Required
            } else if enabled {
                GovernanceEffectiveState::On
            } else {
                GovernanceEffectiveState::Off
            };
            state.revision = state.revision.saturating_add(1);
        }
        let warning_emitted = self.emit_inactive_warning_once();
        Ok(GovernanceFinalization {
            snapshot: read_state(&self.shared).snapshot(),
            warning_emitted,
        })
    }

    /// Activate all previously issued admission tokens after finalization.
    pub fn activate_admission_tokens(&self) -> Result<(), GovernanceRuntimeError> {
        let mut state = write_state(&self.shared);
        if state.phase == GovernanceRuntimePhase::Initializing {
            return Err(GovernanceRuntimeError::NotFinalized);
        }
        state.admission_active = true;
        state.revision = state.revision.saturating_add(1);
        Ok(())
    }

    fn emit_inactive_warning_once(&self) -> bool {
        let snapshot = read_state(&self.shared).snapshot();
        if snapshot.phase != GovernanceRuntimePhase::Off {
            return false;
        }
        let emitted = std::sync::atomic::AtomicBool::new(false);
        self.shared.inactive_warning.call_once(|| {
            emitted.store(true, std::sync::atomic::Ordering::Relaxed);
            let bound_addresses = snapshot
                .bound_addresses
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            tracing::warn!(
                name: "governance.inactive_local_mode",
                event = "governance.inactive_local_mode",
                boot_instance_id = %snapshot.boot_instance_id,
                configured_host = %snapshot.configured_host,
                bound_addresses = %bound_addresses,
                jwt_required = false,
                effective_enabled = false,
                bypassed_gates = "cedar,run_policy,risk_approval",
                "Tool governance is inactive for this loopback-only process"
            );
        });
        emitted.load(std::sync::atomic::Ordering::Relaxed)
    }
}

fn is_allowed_configured_host(configured_host: &str) -> bool {
    matches!(configured_host, "localhost" | "127.0.0.1")
}

#[cfg(test)]
mod tests {
    use super::{
        GovernanceEffectiveState, GovernanceRuntimeError, GovernanceRuntimePhase,
        GovernanceStatusReason, governance_runtime_handles,
    };
    use serial_test::serial;
    use std::{
        io::{self, Write},
        net::SocketAddr,
        sync::{Arc, Mutex},
    };
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct LogBuffer(Arc<Mutex<Vec<u8>>>);

    struct LogWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for LogWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("log buffer remains available")
                .extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for LogBuffer {
        type Writer = LogWriter;

        fn make_writer(&'a self) -> Self::Writer {
            LogWriter(Arc::clone(&self.0))
        }
    }

    impl LogBuffer {
        fn contents(&self) -> String {
            String::from_utf8(self.0.lock().expect("log buffer remains available").clone())
                .expect("captured tracing output is UTF-8")
        }
    }

    fn address(value: &str) -> SocketAddr {
        value.parse().expect("test socket address must parse")
    }

    fn sealed_runtime(
        configured_host: &str,
        jwt_required: bool,
        bound_address: &str,
    ) -> (
        super::GovernanceMutationHandle,
        Vec<super::GovernanceAdmissionToken>,
    ) {
        let (mutation, _, _) = governance_runtime_handles(configured_host);
        mutation.record_installed_authentication(jwt_required);
        mutation
            .declare_ingress("primary-http")
            .expect("declaration succeeds");
        let proof = mutation
            .register_bound_ingress("primary-http", address(bound_address))
            .expect("registration succeeds");
        let tokens = mutation
            .seal_ingress_inventory(&[proof])
            .expect("inventory seals");
        (mutation, tokens)
    }

    #[test]
    #[serial]
    fn initializing_snapshot_is_coherent_and_gates_on() {
        let (_, gate, status) = governance_runtime_handles("localhost");
        let snapshot = status.snapshot();
        assert_eq!(snapshot.phase, GovernanceRuntimePhase::Initializing);
        assert_eq!(snapshot.effective_state, GovernanceEffectiveState::Unknown);
        assert!(snapshot.effective_enabled);
        assert!(!snapshot.may_disable);
        assert!(!snapshot.mutation_available);
        assert_eq!(
            snapshot.reasons,
            vec![GovernanceStatusReason::InitializationIncomplete]
        );
        assert!(gate.effective_enabled());
    }

    #[test]
    #[serial]
    fn exact_configured_literals_are_required() {
        for host in ["localhost", "127.0.0.1"] {
            let (mutation, _) = sealed_runtime(host, false, "127.0.0.1:1906");
            let plan = mutation.preference_plan(None).expect("plan succeeds");
            let finalized = mutation
                .finalize_preference(&plan)
                .expect("finalization succeeds");
            assert_eq!(
                finalized.snapshot.effective_state,
                GovernanceEffectiveState::Off
            );
        }
        for host in ["::1", "0.0.0.0", "LOCALHOST", "127.0.0.2"] {
            let (mutation, _) = sealed_runtime(host, false, "127.0.0.1:1906");
            let plan = mutation.preference_plan(None).expect("plan succeeds");
            let finalized = mutation
                .finalize_preference(&plan)
                .expect("finalization succeeds");
            assert_eq!(
                finalized.snapshot.effective_state,
                GovernanceEffectiveState::Required,
                "configured host {host} must remain mandatory"
            );
        }
    }

    #[test]
    #[serial]
    fn bound_ipv6_loopback_is_valid_for_an_allowed_configured_literal() {
        let (mutation, _) = sealed_runtime("localhost", false, "[::1]:1906");
        let plan = mutation.preference_plan(None).expect("plan succeeds");
        let finalized = mutation
            .finalize_preference(&plan)
            .expect("finalization succeeds");
        assert_eq!(
            finalized.snapshot.effective_state,
            GovernanceEffectiveState::Off
        );
        assert_eq!(
            finalized.snapshot.bound_addresses,
            vec![address("[::1]:1906")]
        );
    }

    #[test]
    #[serial]
    fn jwt_or_non_loopback_binding_makes_governance_required() {
        for (jwt_required, bound) in [(true, "127.0.0.1:1906"), (false, "10.0.0.2:1906")] {
            let (mutation, _) = sealed_runtime("localhost", jwt_required, bound);
            let plan = mutation
                .preference_plan(Some(false))
                .expect("plan succeeds");
            assert!(plan.target_enabled);
            let finalized = mutation
                .finalize_preference(&plan)
                .expect("finalization succeeds");
            assert_eq!(
                finalized.snapshot.effective_state,
                GovernanceEffectiveState::Required
            );
            assert!(finalized.snapshot.effective_enabled);
        }
    }

    #[test]
    #[serial]
    fn publishing_enabled_preserves_required_posture() {
        let (mutation, _) = sealed_runtime("localhost", true, "127.0.0.1:1906");
        let plan = mutation.preference_plan(Some(true)).expect("plan succeeds");
        mutation
            .finalize_preference(&plan)
            .expect("finalization succeeds");

        let published = mutation
            .publish_committed_preference(true)
            .expect("idempotent enabled publication succeeds");

        assert_eq!(
            published.snapshot.effective_state,
            GovernanceEffectiveState::Required
        );
        assert!(published.snapshot.effective_enabled);
        assert!(!published.snapshot.may_disable);
        published
            .snapshot
            .validate()
            .expect("snapshot remains coherent");
    }

    #[test]
    #[serial]
    fn proofs_are_complete_and_late_registration_is_rejected() {
        let (mutation, _, _) = governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        mutation
            .declare_ingress("primary-http")
            .expect("declare primary");
        mutation.declare_ingress("a2a-grpc").expect("declare grpc");
        let primary = mutation
            .register_bound_ingress("primary-http", address("127.0.0.1:1906"))
            .expect("register primary");
        assert!(matches!(
            mutation.seal_ingress_inventory(&[primary.clone()]),
            Err(GovernanceRuntimeError::InvalidIngressProofs)
        ));
        let grpc = mutation
            .register_bound_ingress("a2a-grpc", address("127.0.0.1:50051"))
            .expect("register grpc");
        mutation
            .seal_ingress_inventory(&[primary, grpc])
            .expect("complete inventory seals");
        assert_eq!(
            mutation.declare_ingress("late-http"),
            Err(GovernanceRuntimeError::InventorySealed)
        );
    }

    #[test]
    #[serial]
    fn admission_tokens_remain_inactive_until_finalization_activation() {
        let (mutation, tokens) = sealed_runtime("localhost", false, "127.0.0.1:1906");
        assert!(tokens.iter().all(|token| !token.is_active()));
        let plan = mutation.preference_plan(Some(true)).expect("plan succeeds");
        mutation
            .finalize_preference(&plan)
            .expect("finalization succeeds");
        assert!(tokens.iter().all(|token| !token.is_active()));
        mutation
            .activate_admission_tokens()
            .expect("activation succeeds");
        assert!(
            tokens
                .iter()
                .all(super::GovernanceAdmissionToken::is_active)
        );
    }

    #[test]
    #[serial]
    fn every_real_ingress_token_stays_inactive_until_finalization() {
        let (mutation, _, _) = governance_runtime_handles("localhost");
        mutation.record_installed_authentication(false);
        let registrations = [
            ("primary-http", "127.0.0.1:1906"),
            ("companion-http", "[::1]:1906"),
            ("a2a-grpc", "127.0.0.1:50051"),
        ];
        let mut proofs = Vec::new();
        for (ingress, bound) in registrations {
            mutation.declare_ingress(ingress).expect("declare ingress");
            proofs.push(
                mutation
                    .register_bound_ingress(ingress, address(bound))
                    .expect("register ingress"),
            );
        }
        let tokens = mutation
            .seal_ingress_inventory(&proofs)
            .expect("inventory seals");
        assert_eq!(tokens.len(), 3);
        assert!(tokens.iter().all(|token| !token.is_active()));
        let plan = mutation.preference_plan(None).expect("plan succeeds");
        mutation
            .finalize_preference(&plan)
            .expect("finalization succeeds");
        assert!(tokens.iter().all(|token| !token.is_active()));
        mutation
            .activate_admission_tokens()
            .expect("tokens activate");
        assert!(
            tokens
                .iter()
                .all(super::GovernanceAdmissionToken::is_active)
        );
    }

    #[test]
    #[serial]
    fn inactive_warning_is_emitted_once_per_process() {
        let logs = LogBuffer::default();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .without_time()
            .with_ansi(false)
            .with_writer(logs.clone())
            .finish();
        let _subscriber_guard = tracing::subscriber::set_default(subscriber);
        let (mutation, _) = sealed_runtime("localhost", false, "127.0.0.1:1906");
        let off = mutation
            .preference_plan(Some(false))
            .expect("plan succeeds");
        let first = mutation
            .finalize_preference(&off)
            .expect("finalization succeeds");
        assert!(first.warning_emitted);
        mutation
            .publish_committed_preference(true)
            .expect("turning on succeeds");
        let second = mutation
            .publish_committed_preference(false)
            .expect("turning off succeeds");
        assert!(!second.warning_emitted);
        let output = logs.contents();
        assert_eq!(output.matches("governance.inactive_local_mode").count(), 1);
        for field in [
            "boot_instance_id",
            "configured_host=localhost",
            "bound_addresses=127.0.0.1:1906",
            "jwt_required=false",
            "effective_enabled=false",
            "bypassed_gates=\"cedar,run_policy,risk_approval\"",
        ] {
            assert!(
                output.contains(field),
                "missing warning field {field}: {output}"
            );
        }
    }

    #[test]
    #[serial]
    fn persistence_failure_never_finalizes_off_or_warns() {
        let (mutation, _) = sealed_runtime("localhost", false, "127.0.0.1:1906");
        let finalized = mutation
            .finalize_mutation_unavailable()
            .expect("fail-closed finalization succeeds");
        assert_eq!(
            finalized.snapshot.effective_state,
            GovernanceEffectiveState::On
        );
        assert!(finalized.snapshot.effective_enabled);
        assert!(!finalized.snapshot.mutation_available);
        assert!(!finalized.warning_emitted);
        assert!(
            finalized
                .snapshot
                .reasons
                .contains(&GovernanceStatusReason::PersistenceUnavailable)
        );
    }

    #[test]
    #[serial]
    fn ineligible_persistence_failure_remains_required_and_unavailable() {
        let (mutation, _) = sealed_runtime("0.0.0.0", true, "10.0.0.2:1906");
        let finalized = mutation
            .finalize_mutation_unavailable()
            .expect("fail-closed finalization succeeds");
        assert_eq!(
            finalized.snapshot.effective_state,
            GovernanceEffectiveState::Required
        );
        assert!(!finalized.snapshot.may_disable);
        assert!(!finalized.snapshot.mutation_available);
        assert!(finalized.snapshot.validate().is_ok());
    }

    #[test]
    #[serial]
    fn snapshot_validation_rejects_impossible_state_and_preserves_multiple_reasons() {
        let (mutation, _) = sealed_runtime("0.0.0.0", true, "10.0.0.2:1906");
        let plan = mutation.preference_plan(Some(true)).expect("plan succeeds");
        let finalized = mutation
            .finalize_preference(&plan)
            .expect("finalization succeeds");
        assert_eq!(
            finalized.snapshot.reasons,
            vec![
                GovernanceStatusReason::ConfiguredHostNotAllowed,
                GovernanceStatusReason::JwtRequired,
                GovernanceStatusReason::BoundIngressNotLoopback,
            ]
        );
        assert!(finalized.snapshot.validate().is_ok());

        let mut impossible = finalized.snapshot;
        impossible.effective_enabled = false;
        assert_eq!(
            impossible.validate(),
            Err("incoherent governance runtime snapshot")
        );

        impossible.effective_enabled = true;
        impossible.mutation_available = true;
        impossible
            .reasons
            .insert(0, GovernanceStatusReason::PersistenceUnavailable);
        assert_eq!(
            impossible.validate(),
            Err("incoherent governance runtime snapshot")
        );
    }
}
