//! Identity-bearing description and admission for this UAR process.
//!
//! The trusted host owns instance inventory, credentials, placement, and
//! process lifecycle. This module describes and enforces only the configured
//! identity of the process in which it runs.

use serde::{Deserialize, Serialize};

use crate::config::{ServiceInstanceConfig, ServiceOwnership, WorkspaceLocation};

pub const SERVICE_PROFILE: &str = "uar.service-instance/1";
pub const SERVICE_INSTANCE_CAPABILITY: &str = "service_instance_placement_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceInstanceIdentity {
    pub id: String,
    pub profile: &'static str,
    pub workspace_location: WorkspaceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceEndpointRoles {
    pub runtime: String,
    pub administration: String,
    pub models: String,
    pub console: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceInstanceReferences {
    pub lifecycle_owner: Option<String>,
    pub credential: Option<String>,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlacementSupport {
    pub new: bool,
    pub reattach: bool,
    pub migrate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceInstanceDescriptor {
    pub instance: ServiceInstanceIdentity,
    pub endpoints: ServiceEndpointRoles,
    pub ownership: ServiceOwnership,
    pub references: ServiceInstanceReferences,
    pub capabilities: Vec<String>,
    pub placement: PlacementSupport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlacementIntent {
    New,
    Reattach,
    Migrate,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServicePlacementExpectation {
    pub intent: PlacementIntent,
    pub expected_instance_id: String,
    pub expected_profile: String,
    #[serde(default)]
    pub expected_workspace_location: Option<WorkspaceLocation>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    #[serde(default)]
    pub expected_endpoints: Option<ServiceEndpointRoles>,
    #[serde(default)]
    pub binding_id: Option<String>,
    #[serde(default)]
    pub binding_revision: Option<u64>,
    #[serde(default)]
    pub credential_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectiveServiceBinding {
    pub instance_id: String,
    pub profile: String,
    pub workspace_location: WorkspaceLocation,
    pub endpoints: ServiceEndpointRoles,
    pub capabilities: Vec<String>,
    pub intent: PlacementIntent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_revision: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityDiagnostic {
    pub field: &'static str,
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityResponse {
    pub compatible: bool,
    pub diagnostics: Vec<CompatibilityDiagnostic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_binding: Option<EffectiveServiceBinding>,
}

#[derive(Debug, Clone)]
pub struct ServiceInstanceAuthority {
    descriptor: ServiceInstanceDescriptor,
}

impl ServiceInstanceAuthority {
    #[must_use]
    pub fn new(
        config: &ServiceInstanceConfig,
        a2a_instance_id: &str,
        bound_origin: &str,
        sidecar_mode: bool,
        runtime_capabilities: &[&str],
    ) -> Self {
        let id = if config.instance_id.trim().is_empty() {
            a2a_instance_id.to_owned()
        } else {
            config.instance_id.clone()
        };
        let endpoint = |configured: &Option<String>, path: &str| {
            configured
                .clone()
                .unwrap_or_else(|| format!("{bound_origin}{path}"))
        };
        let endpoints = ServiceEndpointRoles {
            runtime: endpoint(&config.runtime_endpoint, ""),
            administration: endpoint(&config.administration_endpoint, "/api/uar"),
            models: endpoint(&config.models_endpoint, "/v1"),
            console: if sidecar_mode {
                None
            } else {
                Some(endpoint(&config.console_endpoint, "/admin"))
            },
        };
        let mut capabilities = runtime_capabilities
            .iter()
            .filter(|capability| {
                **capability != SERVICE_INSTANCE_CAPABILITY || !id.trim().is_empty()
            })
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        capabilities.sort_unstable();
        capabilities.dedup();
        Self {
            descriptor: ServiceInstanceDescriptor {
                instance: ServiceInstanceIdentity {
                    id,
                    profile: SERVICE_PROFILE,
                    workspace_location: config.workspace_location,
                },
                endpoints,
                ownership: config.ownership,
                references: ServiceInstanceReferences {
                    lifecycle_owner: config.lifecycle_owner_ref.clone(),
                    credential: config.credential_ref.clone(),
                    workspace: config.workspace_ref.clone(),
                },
                capabilities,
                placement: PlacementSupport {
                    new: true,
                    reattach: true,
                    migrate: false,
                },
            },
        }
    }

    #[must_use]
    pub fn descriptor(&self) -> &ServiceInstanceDescriptor {
        &self.descriptor
    }

    #[must_use]
    pub fn evaluate(
        &self,
        expectation: &ServicePlacementExpectation,
        allowed_intent: PlacementIntent,
    ) -> CompatibilityResponse {
        let mut diagnostics = Vec::new();
        if self.descriptor.instance.id.trim().is_empty() {
            diagnostics.push(diagnostic(
                "expectedInstanceId",
                "instance.identity-unconfigured",
                "This runtime has no stable configured instance identity.",
            ));
        }
        if expectation.intent == PlacementIntent::Migrate {
            diagnostics.push(diagnostic(
                "intent",
                "placement.migration-unsupported",
                "Live run migration is not supported by this UAR profile.",
            ));
        } else if expectation.intent != allowed_intent {
            diagnostics.push(diagnostic(
                "intent",
                "placement.intent-mismatch",
                format!("This operation requires placement intent {allowed_intent:?}."),
            ));
        }
        if expectation.expected_instance_id != self.descriptor.instance.id {
            diagnostics.push(diagnostic(
                "expectedInstanceId",
                "instance.identity-mismatch",
                "The selected runtime instance is not the process that answered.",
            ));
        }
        if expectation.expected_profile != self.descriptor.instance.profile {
            diagnostics.push(diagnostic(
                "expectedProfile",
                "instance.profile-unsupported",
                "The selected service profile is not supported by this runtime.",
            ));
        }
        if expectation
            .expected_workspace_location
            .is_some_and(|expected| expected != self.descriptor.instance.workspace_location)
        {
            diagnostics.push(diagnostic(
                "expectedWorkspaceLocation",
                "instance.workspace-location-mismatch",
                "The selected workspace location does not match this runtime.",
            ));
        }
        if expectation
            .expected_endpoints
            .as_ref()
            .is_some_and(|expected| expected != &self.descriptor.endpoints)
        {
            diagnostics.push(diagnostic(
                "expectedEndpoints",
                "instance.endpoint-mismatch",
                "The selected endpoint roles do not match this runtime.",
            ));
        }
        for capability in &expectation.required_capabilities {
            if !self.descriptor.capabilities.contains(capability) {
                diagnostics.push(diagnostic(
                    "requiredCapabilities",
                    "instance.capability-unsupported",
                    format!("Required capability '{capability}' is not implemented."),
                ));
            }
        }
        if let Some(reference) = expectation.credential_ref.as_ref() {
            if reference.chars().any(char::is_whitespace) || !reference.contains("://") {
                diagnostics.push(diagnostic(
                    "credentialRef",
                    "instance.credential-reference-invalid",
                    "The runtime credential reference must be an opaque host-store reference.",
                ));
            } else if self.descriptor.references.credential.as_ref() != Some(reference) {
                diagnostics.push(diagnostic(
                    "credentialRef",
                    "instance.credential-reference-mismatch",
                    "The runtime credential reference does not match this service instance.",
                ));
            }
        }
        let compatible = diagnostics.is_empty();
        CompatibilityResponse {
            compatible,
            diagnostics,
            effective_binding: compatible.then(|| EffectiveServiceBinding {
                instance_id: self.descriptor.instance.id.clone(),
                profile: self.descriptor.instance.profile.to_owned(),
                workspace_location: self.descriptor.instance.workspace_location,
                endpoints: self.descriptor.endpoints.clone(),
                capabilities: self.descriptor.capabilities.clone(),
                intent: expectation.intent,
                binding_id: expectation.binding_id.clone(),
                binding_revision: expectation.binding_revision,
                credential_ref: expectation.credential_ref.clone(),
            }),
        }
    }

    pub fn revalidate(
        &self,
        binding: &EffectiveServiceBinding,
    ) -> Result<(), CompatibilityResponse> {
        let response = self.evaluate(
            &ServicePlacementExpectation {
                intent: binding.intent,
                expected_instance_id: binding.instance_id.clone(),
                expected_profile: binding.profile.clone(),
                expected_workspace_location: Some(binding.workspace_location),
                required_capabilities: binding.capabilities.clone(),
                expected_endpoints: Some(binding.endpoints.clone()),
                binding_id: binding.binding_id.clone(),
                binding_revision: binding.binding_revision,
                credential_ref: binding.credential_ref.clone(),
            },
            binding.intent,
        );
        if response.compatible {
            Ok(())
        } else {
            Err(response)
        }
    }
}

fn diagnostic(
    field: &'static str,
    code: &'static str,
    message: impl Into<String>,
) -> CompatibilityDiagnostic {
    CompatibilityDiagnostic {
        field,
        code,
        message: message.into(),
    }
}
