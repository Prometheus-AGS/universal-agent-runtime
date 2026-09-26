use chrono::Utc;
use serde_json::Value;

use crate::uar::domain::collaboration::{
    BindingCommandRequest, BindingInstallResponse, BindingPreflightResponse,
    CollaborationCatalogState, CollaborationCommandReceipt, ConversionDiagnostic,
    ConversionDisposition, DeploymentBindingRecord, ImmutableDefinitionRef,
};

use super::service::{
    CollaborationCatalogService, CollaborationError, MAX_CAS_ATTEMPTS, receipt_key,
    validate_owner,
};
use super::validation::{
    canonical_digest, request_digest, validate_common_sections, validate_digest, validate_id,
    validate_semver,
};

impl CollaborationCatalogService {
    pub async fn preflight_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: &BindingCommandRequest,
    ) -> Result<BindingPreflightResponse, CollaborationError> {
        let state = self.load_state().await?;
        let preflight = validate_binding(owner_id, workspace_id, request, &state)?;
        let storage_key = binding_key(owner_id, workspace_id, &preflight.binding_id);
        enforce_binding_revision(
            request.expected_revision,
            state.bindings.get(&storage_key).map(|binding| binding.revision),
        )?;
        Ok(preflight)
    }

    pub async fn install_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        request: BindingCommandRequest,
    ) -> Result<BindingInstallResponse, CollaborationError> {
        validate_owner(owner_id)?;
        let command_digest = request_digest(&request).map_err(CollaborationError::from)?;
        let receipt_key = receipt_key(owner_id, &request.command_id);

        for _ in 0..MAX_CAS_ATTEMPTS {
            let current = self.load_state().await?;
            let preflight = validate_binding(owner_id, workspace_id, &request, &current)?;
            if let Some(receipt) = current.command_receipts.get(&receipt_key) {
                if receipt.operation != "install-binding"
                    || receipt.request_digest != command_digest
                {
                    return Err(CollaborationError::Conflict(format!(
                        "commandId '{}' was already used for a different operation or content",
                        request.command_id
                    )));
                }
                let receipt_revision = receipt.binding_revision.ok_or_else(|| {
                    CollaborationError::Storage(
                        "idempotent binding receipt has no binding revision".to_owned(),
                    )
                })?;
                let binding = current
                    .binding_history
                    .get(&binding_revision_key(
                        owner_id,
                        workspace_id,
                        &receipt.resource_id,
                        receipt_revision,
                    ))
                    .cloned()
                    .ok_or_else(|| {
                        CollaborationError::Storage(
                            "idempotent binding receipt has no matching record".to_owned(),
                        )
                    })?;
                return Ok(BindingInstallResponse {
                    preflight,
                    binding,
                    receipt: receipt.clone(),
                });
            }

            let binding_id = required_string(&request.binding, "id")?.to_owned();
            let storage_key = binding_key(owner_id, workspace_id, &binding_id);
            let existing_revision = current
                .bindings
                .get(&storage_key)
                .map(|binding| binding.revision);
            enforce_binding_revision(request.expected_revision, existing_revision)?;
            let revision = existing_revision.unwrap_or(0).saturating_add(1);
            let declared_revision = request
                .binding
                .get("revision")
                .and_then(Value::as_u64)
                .ok_or_else(|| {
                    CollaborationError::Invalid(
                        "binding revision must be a non-negative integer".to_owned(),
                    )
                })?;
            if declared_revision != revision {
                return Err(CollaborationError::Conflict(format!(
                    "binding document revision {declared_revision} must be next revision {revision}"
                )));
            }
            let package = parse_reference(request.binding.get("package").ok_or_else(|| {
                CollaborationError::Invalid("binding package is missing".to_owned())
            })?)?;
            let binding = DeploymentBindingRecord {
                id: binding_id.clone(),
                owner_id: owner_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
                revision,
                document: request.binding.clone(),
                package,
                activation_supported: preflight.activation_supported,
                preflight_diagnostics: preflight.diagnostics.clone(),
                updated_at: Utc::now(),
            };

            let mut next = current.clone();
            next.generation = current.generation.saturating_add(1);
            next.bindings.insert(storage_key, binding.clone());
            next.binding_history.insert(
                binding_revision_key(owner_id, workspace_id, &binding.id, revision),
                binding.clone(),
            );
            let receipt = CollaborationCommandReceipt {
                command_id: request.command_id.clone(),
                owner_id: owner_id.to_owned(),
                request_digest: command_digest.clone(),
                operation: "install-binding".to_owned(),
                resource_id: binding_id,
                catalog_revision: next.catalog_revision,
                binding_revision: Some(revision),
                committed_at: Utc::now(),
            };
            next.command_receipts
                .insert(receipt_key.clone(), receipt.clone());
            if self.cas(current.generation, &next).await? {
                return Ok(BindingInstallResponse {
                    preflight,
                    binding,
                    receipt,
                });
            }
        }
        Err(CollaborationError::Conflict(
            "catalog changed repeatedly while installing the binding".to_owned(),
        ))
    }

    pub async fn list_bindings(
        &self,
        owner_id: &str,
        workspace_id: &str,
    ) -> Result<Vec<DeploymentBindingRecord>, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        Ok(self
            .load_state()
            .await?
            .bindings
            .into_values()
            .filter(|binding| binding.owner_id == owner_id && binding.workspace_id == workspace_id)
            .collect())
    }

    pub async fn get_binding(
        &self,
        owner_id: &str,
        workspace_id: &str,
        binding_id: &str,
    ) -> Result<DeploymentBindingRecord, CollaborationError> {
        validate_owner(owner_id)?;
        validate_id(workspace_id)?;
        validate_id(binding_id)?;
        self.load_state()
            .await?
            .bindings
            .get(&binding_key(owner_id, workspace_id, binding_id))
            .cloned()
            .ok_or_else(|| CollaborationError::NotFound(binding_id.to_owned()))
    }
}

fn validate_binding(
    owner_id: &str,
    workspace_id: &str,
    request: &BindingCommandRequest,
    state: &CollaborationCatalogState,
) -> Result<BindingPreflightResponse, CollaborationError> {
    validate_owner(owner_id)?;
    validate_id(workspace_id)?;
    if request.command_id.trim().is_empty() {
        return Err(CollaborationError::Invalid(
            "commandId must not be empty".to_owned(),
        ));
    }
    let document = &request.binding;
    for field in [
        "profile",
        "kind",
        "id",
        "version",
        "contentDigest",
        "provenance",
        "requiredCapabilities",
        "extensions",
        "exportClass",
        "package",
        "ownerId",
        "workspaceId",
        "runtimeInstanceId",
        "revision",
        "modelBindings",
        "skillBindings",
        "storage",
        "policyRevision",
        "effectiveLimits",
        "effectiveBudget",
        "contextGrants",
        "representationGrantRefs",
        "status",
    ] {
        if document.get(field).is_none() {
            return Err(CollaborationError::Invalid(format!(
                "binding required field '{field}' is missing"
            )));
        }
    }
    if required_string(document, "profile")?
        != crate::uar::domain::collaboration::COLLABORATION_PROFILE
        || required_string(document, "kind")? != "DeploymentBinding"
        || required_string(document, "exportClass")? != "private-installed-state"
    {
        return Err(CollaborationError::Invalid(
            "unsupported deployment binding profile, kind, or exportClass".to_owned(),
        ));
    }
    validate_id(required_string(document, "id")?)?;
    validate_semver(required_string(document, "version")?)?;
    validate_digest(required_string(document, "contentDigest")?)?;
    validate_common_sections(document)?;
    let observed_digest = canonical_digest(document)?;
    if observed_digest != required_string(document, "contentDigest")? {
        return Err(CollaborationError::Invalid(format!(
            "binding contentDigest mismatch: observed {observed_digest}"
        )));
    }
    if required_string(document, "ownerId")? != owner_id
        || required_string(document, "workspaceId")? != workspace_id
    {
        return Err(CollaborationError::Invalid(
            "binding ownerId/workspaceId must match authenticated request scope".to_owned(),
        ));
    }
    validate_private_references(document)?;
    reject_binding_secrets(document, "$")?;
    let package = parse_reference(document.get("package").ok_or_else(|| {
        CollaborationError::Invalid("binding package is missing".to_owned())
    })?)?;
    if !state.packages.contains_key(&package.storage_key()) {
        return Err(CollaborationError::NotFound(format!(
            "package {} {} {}",
            package.id, package.version, package.digest
        )));
    }
    let diagnostics = vec![ConversionDiagnostic {
        field: "status".to_owned(),
        disposition: ConversionDisposition::RequiredUnsupported,
        message: "binding is installed, but TeamInstance activation is unavailable until I2"
            .to_owned(),
    }];
    Ok(BindingPreflightResponse {
        binding_id: required_string(document, "id")?.to_owned(),
        package,
        diagnostics,
        activation_supported: false,
        request_digest: request_digest(request).map_err(CollaborationError::from)?,
    })
}

fn enforce_binding_revision(
    expected: Option<u64>,
    actual: Option<u64>,
) -> Result<(), CollaborationError> {
    match (expected, actual) {
        (None | Some(0), None) => Ok(()),
        (Some(expected), Some(actual)) if expected == actual => Ok(()),
        (expected, actual) => Err(CollaborationError::Conflict(format!(
            "binding expectedRevision {expected:?} does not match {actual:?}"
        ))),
    }
}

fn validate_private_references(document: &Value) -> Result<(), CollaborationError> {
    for binding in document
        .get("modelBindings")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let reference = required_string(binding, "credentialRef")?;
        if reference.chars().any(char::is_whitespace) || !reference.contains("://") {
            return Err(CollaborationError::Invalid(
                "credentialRef must be an opaque protected-store reference".to_owned(),
            ));
        }
    }
    let storage = document
        .get("storage")
        .ok_or_else(|| CollaborationError::Invalid("binding storage is missing".to_owned()))?;
    let connection = required_string(storage, "connectionRef")?;
    if connection.chars().any(char::is_whitespace) || !connection.contains("://") {
        return Err(CollaborationError::Invalid(
            "connectionRef must be an opaque protected-store reference".to_owned(),
        ));
    }
    if storage.get("durableTransactions").and_then(Value::as_bool) != Some(true) {
        return Err(CollaborationError::Invalid(
            "binding storage must provide durableTransactions".to_owned(),
        ));
    }
    Ok(())
}

fn reject_binding_secrets(value: &Value, path: &str) -> Result<(), CollaborationError> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                let normalized = key.to_ascii_lowercase().replace(['-', '_'], "");
                if matches!(
                    normalized.as_str(),
                    "apikey" | "password" | "secret" | "token" | "credential"
                ) {
                    return Err(CollaborationError::Invalid(format!(
                        "binding contains secret value field '{path}.{key}'; store it behind credentialRef"
                    )));
                }
                reject_binding_secrets(child, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_binding_secrets(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_reference(value: &Value) -> Result<ImmutableDefinitionRef, CollaborationError> {
    let reference: ImmutableDefinitionRef = serde_json::from_value(value.clone())
        .map_err(|error| CollaborationError::Invalid(error.to_string()))?;
    validate_id(&reference.id)?;
    validate_semver(&reference.version)?;
    validate_digest(&reference.digest)?;
    Ok(reference)
}

fn required_string<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, CollaborationError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| {
            CollaborationError::Invalid(format!("'{field}' must be a non-empty string"))
        })
}

fn binding_key(owner_id: &str, workspace_id: &str, binding_id: &str) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{binding_id}")
}

fn binding_revision_key(
    owner_id: &str,
    workspace_id: &str,
    binding_id: &str,
    revision: u64,
) -> String {
    format!("{owner_id}\u{1f}{workspace_id}\u{1f}{binding_id}\u{1f}{revision}")
}
