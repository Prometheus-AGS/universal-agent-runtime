//! Trusted supervisor recovery. Nothing in this module is an agent capability.
//!
//! A controller observes an execution domain it contained *before* execution.
//! Transport authentication, an operator-qualified mechanism, immutable domain
//! binding and a durable one-use challenge are all required. Exclusion does not
//! settle issued effects, approvals or accounting reservations.

use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use crate::uar::{domain::team_execution::TeamExecutionFence, persistence::PersistenceLayer};

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("supervisor recovery is not qualified for this execution domain")]
    Unqualified,
    #[error("supervisor exclusion identity or observation is invalid")]
    Invalid,
    #[error("supervisor ownership challenge conflicts with durable state")]
    Conflict,
    #[error("supervisor controller is unavailable")]
    Unavailable,
    #[error("supervisor recovery was cancelled; no exclusion is implied")]
    Cancelled,
    #[error("durable supervisor recovery is unsupported")]
    Unsupported,
}

/// Mechanism-specific identities supplied by the authenticated containment
/// controller. Names/PIDs alone are intentionally insufficient.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExecutionDomain {
    NativeBoot { platform: String, machine_id: String, boot_id: String,
        service_name: String, root_process_identity: String },
    Kubernetes { cluster_id: String, namespace: String, pod_uid: String,
        node_uid: String, node_boot_id: String, sandbox_id: String,
        container_ids: Vec<String> },
}

impl ExecutionDomain {
    fn mechanism(&self) -> &'static str {
        match self {
            Self::NativeBoot { .. } => "native-boot-exclusion-v1",
            Self::Kubernetes { .. } => "kubernetes-cri-containment-v1",
        }
    }
    fn validate(&self) -> Result<(), SupervisorError> {
        let fields: Vec<&str> = match self {
            Self::NativeBoot { platform, machine_id, boot_id, service_name, root_process_identity } => {
                if !matches!((platform.as_str(),service_name.as_str()),
                    ("darwin","com.prometheus.universal-agent-runtime") | ("linux","uar.service")
                    | ("windows","PrometheusUniversalAgentRuntime")) { return Err(SupervisorError::Invalid); }
                vec![platform,machine_id,boot_id,service_name,root_process_identity]
            }
            Self::Kubernetes { cluster_id, namespace, pod_uid, node_uid, node_boot_id, sandbox_id, container_ids } => {
                if container_ids.is_empty() { return Err(SupervisorError::Invalid); }
                let mut ids = container_ids.clone(); ids.sort(); ids.dedup();
                if ids.len() != container_ids.len() { return Err(SupervisorError::Invalid); }
                let mut fields = vec![cluster_id.as_str(), namespace.as_str(), pod_uid.as_str(), node_uid.as_str(), node_boot_id.as_str(), sandbox_id.as_str()];
                fields.extend(container_ids.iter().map(String::as_str)); fields
            }
        };
        if fields.iter().any(|field| field.is_empty() || field.len() > 1024 || field.chars().any(char::is_control)) {
            return Err(SupervisorError::Invalid);
        }
        Ok(())
    }
}

/// Host configuration loaded only from the service operator's private settings.
/// A qualification record is a reference to retained real platform evidence,
/// not a request flag or a controller self-assertion.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SupervisorConfig {
    pub endpoint: String,
    pub controller_id: String,
    pub controller_ca: PathBuf,
    pub client_identity: PathBuf,
    pub qualification_id: String,
    pub qualified_mechanism: String,
    pub deployment_id: String,
    #[serde(default)]
    pub authorized_recovery_deployments: Vec<String>,
    /// Other node controllers explicitly trusted by this installation. Peers
    /// may not contain peers; persisted domains select only this fixed map.
    #[serde(default)]
    pub recovery_peers: Vec<SupervisorConfig>,
}

/// Immutable containment registration, persisted before a runtime advertises
/// recoverability. Runtime epochs and team incarnations remain distinct.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupervisorRegistration {
    pub runtime_epoch: String,
    pub team_incarnation: String,
    pub controller_id: String,
    pub qualification_id: String,
    pub deployment_id: String,
    pub domain: ExecutionDomain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RecoveryTarget {
    CheckpointEpoch { runtime_epoch: String },
    TeamOwner { fence: TeamExecutionFence },
}

/// Nonexpiring challenge, bound to a single predecessor and target. Retrying
/// the same challenge is safe; its durable transition is monotonic and one-use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SupervisorChallenge {
    pub id: String,
    pub registration: SupervisorRegistration,
    pub target: RecoveryTarget,
    pub generation: u64,
}

/// Each variant names an actual exclusion observation, not service status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ExclusionObservation {
    NativeBoot { platform: String, machine_id: String, previous_boot_id: String,
        observed_boot_id: String },
    KubernetesRuntime { node_uid: String, node_boot_id: String, sandbox_id: String,
        stopped_container_ids: Vec<String>, sandbox_stopped: bool,
        descendant_cgroup_empty: bool, restart_inhibited: bool,
        kubelet_unit: String, kubelet_cgroup_empty: bool, kubelet_runtime_masked: bool },

}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControllerExclusion {
    challenge: SupervisorChallenge,
    controller_id: String,
    qualification_id: String,
    observation: ExclusionObservation,
}

/// Sealed proof: no Deserialize or public constructor. Persistence can consume
/// it only after the host validates the authenticated controller response.
/// Clones preserve the same private immutable challenge and observation; the
/// durable generation transition still consumes that challenge atomically.
#[derive(Debug, Clone)]
pub struct VerifiedSupervisorExclusion {
    challenge: SupervisorChallenge,
    observation_json: String,
}
impl VerifiedSupervisorExclusion {
    pub fn challenge(&self) -> &SupervisorChallenge { &self.challenge }
    pub fn observation_json(&self) -> &str { &self.observation_json }
}

#[derive(Debug)]
pub struct SupervisorRecoveryHost {
    persistence: Arc<dyn PersistenceLayer>,
    client: reqwest::Client,
    config: SupervisorConfig,
    endpoint: reqwest::Url,
    peers: BTreeMap<String, SupervisorRecoveryHost>,
}

impl SupervisorRecoveryHost {
    /// Build only from trusted service configuration. Certificates never enter
    /// logs or persisted records; public roots, redirects and proxies are off.
    pub fn new(persistence: Arc<dyn PersistenceLayer>, config: SupervisorConfig) -> anyhow::Result<Self> {
        if config.recovery_peers.len() > 64 || config.recovery_peers.iter().any(|peer| !peer.recovery_peers.is_empty()) {
            return Err(SupervisorError::Invalid.into());
        }
        let endpoint = reqwest::Url::parse(&config.endpoint).map_err(|_| SupervisorError::Invalid)?;
        if endpoint.scheme() != "https" || !endpoint.username().is_empty() || endpoint.password().is_some()
            || endpoint.query().is_some() || endpoint.fragment().is_some()
            || config.controller_id.is_empty() || config.qualification_id.is_empty() || config.deployment_id.is_empty()
            || !["native-boot-exclusion-v1", "kubernetes-cri-containment-v1"].contains(&config.qualified_mechanism.as_str()) {
            return Err(SupervisorError::Unqualified.into());
        }
        let ca = reqwest::Certificate::from_pem(&std::fs::read(&config.controller_ca)?)
            .map_err(|_| SupervisorError::Invalid)?;
        let identity = reqwest::Identity::from_pem(&std::fs::read(&config.client_identity)?)
            .map_err(|_| SupervisorError::Invalid)?;
        let client = reqwest::Client::builder().tls_backend_rustls().tls_certs_only([ca])
            .identity(identity).no_proxy()
            .redirect(reqwest::redirect::Policy::none()).timeout(Duration::from_secs(60))
            .build().map_err(|_| SupervisorError::Unavailable)?;
        let mut peers = BTreeMap::new();
        for peer in &config.recovery_peers {
            if peer.controller_id == config.controller_id || peers.contains_key(&peer.controller_id) {
                return Err(SupervisorError::Invalid.into());
            }
            peers.insert(peer.controller_id.clone(), Self::new(Arc::clone(&persistence), peer.clone())?);
        }
        Ok(Self { persistence, client, config, endpoint, peers })
    }

    async fn request<T: serde::de::DeserializeOwned>(&self, path: &str, body: &impl Serialize,
        cancellation: &CancellationToken) -> anyhow::Result<T> {
        let url = self.endpoint.join(path).map_err(|_| SupervisorError::Invalid)?;
        let operation = async {
            let mut response = self.client.post(url).json(body).send().await.map_err(|_| SupervisorError::Unavailable)?;
            if !response.status().is_success() { return Err(SupervisorError::Unavailable.into()); }
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|_| SupervisorError::Unavailable)? {
                if bytes.len().saturating_add(chunk.len()) > 65536 { return Err(SupervisorError::Invalid.into()); }
                bytes.extend_from_slice(&chunk);
            }
            serde_json::from_slice(&bytes).map_err(|_| SupervisorError::Invalid.into())
        };
        tokio::select! { biased;
            _ = cancellation.cancelled() => Err(SupervisorError::Cancelled.into()),
            result = operation => result,
        }
    }

    /// Register the caller's epoch against an already-contained process. The
    /// controller must verify this client's authenticated deployment identity,
    /// root process start identity, and its own launch-time containment journal.
    pub async fn register(&self, runtime_epoch: &str, team_incarnation: &str,
        cancellation: &CancellationToken) -> anyhow::Result<SupervisorRegistration> {
        if uuid::Uuid::parse_str(runtime_epoch).is_err() || uuid::Uuid::parse_str(team_incarnation).is_err() {
            return Err(SupervisorError::Invalid.into());
        }
        let registration: SupervisorRegistration = self.request("v1/containment/register", &serde_json::json!({
            "runtime_epoch": runtime_epoch, "team_incarnation": team_incarnation,
            "process_id": std::process::id(), "deployment_id": self.config.deployment_id,
            "qualification_id": self.config.qualification_id,
        }), cancellation).await?;
        self.validate_registration(&registration)?;
        if registration.runtime_epoch != runtime_epoch || registration.team_incarnation != team_incarnation
            || registration.deployment_id != self.config.deployment_id
            || registration.controller_id != self.config.controller_id {
            return Err(SupervisorError::Invalid.into());
        }
        self.persistence.register_supervisor_domain(&registration).await?;
        Ok(registration)
    }

    fn validate_registration(&self, registration: &SupervisorRegistration) -> anyhow::Result<()> {
        let controller = self.controller(&registration.controller_id)?;
        registration.domain.validate()?;
        if registration.qualification_id != controller.config.qualification_id
            || (registration.deployment_id != controller.config.deployment_id
                && !controller.config.authorized_recovery_deployments.contains(&registration.deployment_id))
            || registration.domain.mechanism() != controller.config.qualified_mechanism {
            return Err(SupervisorError::Unqualified.into());
        }
        Ok(())
    }

    fn controller(&self, controller_id: &str) -> Result<&Self, SupervisorError> {
        if controller_id == self.config.controller_id { return Ok(self); }
        self.peers.get(controller_id).ok_or(SupervisorError::Unqualified)
    }

    /// Operator authorization is checked by the host API before this method;
    /// neither agent-supplied metadata nor an uploaded receipt reaches it.
    pub async fn challenge(&self, runtime_epoch: &str, target: RecoveryTarget) -> anyhow::Result<SupervisorChallenge> {
        let registration = self.persistence.load_supervisor_domain(runtime_epoch).await?
            .ok_or(SupervisorError::Unqualified)?;
        self.validate_registration(&registration)?;
        match &target {
            RecoveryTarget::CheckpointEpoch { runtime_epoch } if runtime_epoch == &registration.runtime_epoch => {},
            RecoveryTarget::TeamOwner { fence } if fence.incarnation_id == registration.team_incarnation && fence.epoch > 0 => {},
            _ => return Err(SupervisorError::Invalid.into()),
        }
        let challenge = SupervisorChallenge { id: uuid::Uuid::new_v4().to_string(), registration,
            target, generation: 1 };
        self.persistence.create_supervisor_challenge(&challenge).await?;
        Ok(challenge)
    }

    /// Ask the controller to inhibit replacement/restart, terminate the exact
    /// registered domain, then independently observe complete exclusion.
    pub async fn exclude(&self, challenge_id: &str, cancellation: &CancellationToken)
        -> anyhow::Result<VerifiedSupervisorExclusion> {
        let challenge = self.persistence.load_supervisor_challenge(challenge_id).await?
            .ok_or(SupervisorError::Conflict)?;
        self.validate_registration(&challenge.registration)?;
        let controller = self.controller(&challenge.registration.controller_id)?;
        let receipt: ControllerExclusion = controller.request("v1/containment/exclude", &challenge, cancellation).await?;
        if receipt.challenge != challenge || receipt.controller_id != controller.config.controller_id
            || receipt.qualification_id != controller.config.qualification_id {
            return Err(SupervisorError::Invalid.into());
        }
        validate_observation(&challenge.registration.domain, &receipt.observation)?;
        Ok(VerifiedSupervisorExclusion { challenge, observation_json: serde_json::to_string(&receipt.observation)? })
    }

    /// Consume a team challenge and close the runtime epoch before catalog
    /// staging. A crash between these stores keeps authority blocked; a fresh
    /// challenge may reuse the controller's immutable domain observation.
    pub async fn commit_team_exclusion(&self, proof: &VerifiedSupervisorExclusion) -> anyhow::Result<()> {
        if !matches!(&proof.challenge.target, RecoveryTarget::TeamOwner { .. }) {
            return Err(SupervisorError::Invalid.into());
        }
        self.persistence.commit_supervisor_checkpoint_exclusion(proof).await
    }

    /// Atomically consumes the challenge and closes the predecessor epoch. It
    /// does not replay a claimed continuation or release uncertain liabilities.
    pub async fn commit_checkpoint_exclusion(&self, proof: &VerifiedSupervisorExclusion) -> anyhow::Result<()> {
        if !matches!(&proof.challenge.target, RecoveryTarget::CheckpointEpoch { .. }) {
            return Err(SupervisorError::Invalid.into());
        }
        self.persistence.commit_supervisor_checkpoint_exclusion(proof).await
    }
}

fn validate_observation(domain: &ExecutionDomain, observation: &ExclusionObservation) -> Result<(), SupervisorError> {
    let excluded = match (domain, observation) {
        (ExecutionDomain::NativeBoot { platform, machine_id, boot_id, .. },
            ExclusionObservation::NativeBoot { platform: observed_platform, machine_id: observed_machine,
                previous_boot_id, observed_boot_id }) => platform == observed_platform && machine_id == observed_machine
                    && boot_id == previous_boot_id && !observed_boot_id.is_empty() && boot_id != observed_boot_id,
        (ExecutionDomain::Kubernetes { node_uid: node, node_boot_id: boot, sandbox_id: sandbox, container_ids, .. },
            ExclusionObservation::KubernetesRuntime { node_uid, node_boot_id, sandbox_id, stopped_container_ids, sandbox_stopped, descendant_cgroup_empty, restart_inhibited, kubelet_unit, kubelet_cgroup_empty, kubelet_runtime_masked }) => {
            let mut expected = container_ids.clone(); expected.sort();
            let mut actual = stopped_container_ids.clone(); actual.sort();
            node == node_uid && boot == node_boot_id && sandbox == sandbox_id && expected == actual
                && *sandbox_stopped && *descendant_cgroup_empty && *restart_inhibited
                && kubelet_unit == "kubelet.service" && *kubelet_cgroup_empty && *kubelet_runtime_masked
        }
        _ => false,
    };
    if !excluded { return Err(SupervisorError::Invalid); }
    Ok(())
}
