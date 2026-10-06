//! Safe failure provenance carried into the existing durable team attempt.
use crate::uar::{
    compiler::collaboration::CollaborationError,
    domain::team_execution::TeamExecutionDiagnostic,
};

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub(crate) enum TeamFailureStage {
    #[error("request-preparation")]
    Preparation,
    #[error("handoff-validation")]
    Validation,
    #[error("provider-opening")]
    Opening,
    #[error("handoff-recording")]
    Recording,
}

/// Context retains the underlying typed provider error and retry policy.
pub(crate) fn at_stage(
    error: anyhow::Error,
    stage: TeamFailureStage,
    profiled: bool,
) -> anyhow::Error {
    if profiled { error.context(stage) } else { error }
}

fn collaboration_code(value: &str) -> Option<String> {
    matches!(value,
        "TEAM_SCOPE_DENIED" | "TEAM_EDGE_DENIED" |
        "TEAM_CONTEXT_REQUIRED_UNSUPPORTED" | "TEAM_CONTEXT_REQUIRED_TOO_LARGE" |
        "TEAM_REVISION_CONFLICT" | "TEAM_EXECUTION_EPOCH_STALE" |
        "TEAM_EXECUTION_OWNER_CONFLICT" | "TEAM_COMMAND_CONFLICT" |
        "TEAM_BUDGET_EXHAUSTED" | "TEAM_PROFILE_UNSUPPORTED" |
        "TEAM_ROUTE_PROFILE_MISMATCH"
    ).then(|| value.to_owned())
}

pub(crate) fn diagnostic(error: &anyhow::Error, streaming: bool) -> TeamExecutionDiagnostic {
    let stage = error.downcast_ref::<TeamFailureStage>();
    let source_stage = stage.map(ToString::to_string)
        .unwrap_or_else(|| if streaming { "provider-stream" } else { "provider-opening" }.into());
    let provider = super::ProviderError::from_anyhow(error);
    let collaboration = error.downcast_ref::<CollaborationError>();
    let category = if let Some(provider) = provider {
        provider.code()
    } else if let Some(collaboration) = collaboration {
        match collaboration {
            CollaborationError::Invalid(_) => "collaboration_invalid",
            CollaborationError::NotFound(_) => "collaboration_not_found",
            CollaborationError::Conflict(_) => "collaboration_conflict",
            CollaborationError::Storage(_) => "collaboration_storage",
        }
    } else if matches!(stage, Some(TeamFailureStage::Preparation)) {
        "request_preparation_failed"
    } else {
        "unclassified"
    };
    let collaboration_code = match collaboration {
        Some(CollaborationError::Conflict(value)) => collaboration_code(value),
        _ if matches!(stage, Some(TeamFailureStage::Preparation)) =>
            collaboration_code(&error.root_cause().to_string()),
        _ => None,
    };
    let code = if collaboration.is_some()
        || matches!(stage, Some(TeamFailureStage::Validation | TeamFailureStage::Recording))
    {
        "TEAM_COLLABORATION_HANDOFF_FAILED"
    } else if matches!(stage, Some(TeamFailureStage::Preparation)) {
        "TEAM_REQUEST_PREPARATION_FAILED"
    } else if streaming {
        "TEAM_PROVIDER_STREAM_FAILED"
    } else {
        "TEAM_PROVIDER_REQUEST_REJECTED"
    };
    TeamExecutionDiagnostic {
        code: code.into(),
        field: None,
        retryable: false,
        action: if code == "TEAM_COLLABORATION_HANDOFF_FAILED" { "reconcile" } else { "change-settings" }.into(),
        protected_diagnostic_ref: Some(uuid::Uuid::new_v4().to_string()),
        source_stage: Some(source_stage),
        category: Some(category.into()),
        http_status: provider.and_then(|error| error.status),
        collaboration_code,
    }
}

/// Only safe machine fields cross the terminal-message compatibility seam.
pub(crate) fn reason(diagnostic: &TeamExecutionDiagnostic) -> String {
    let metadata = serde_json::json!({
        "sourceStage": diagnostic.source_stage,
        "category": diagnostic.category,
        "httpStatus": diagnostic.http_status,
        "collaborationCode": diagnostic.collaboration_code,
    });
    format!("{}; diagnostic reference {}; diagnostic metadata {}",
        diagnostic.code,
        diagnostic.protected_diagnostic_ref.as_deref().unwrap_or_default(),
        metadata)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SafeMetadata {
    source_stage: Option<String>,
    category: Option<String>,
    http_status: Option<u16>,
    collaboration_code: Option<String>,
}

/// Accept legacy receipts and validate additive fields at the event boundary.
pub(crate) fn from_reason(reason: &str) -> Option<TeamExecutionDiagnostic> {
    let (code, tail) = reason.split_once("; diagnostic reference ")?;
    if !matches!(code, "TEAM_PROVIDER_REQUEST_REJECTED" | "TEAM_PROVIDER_STREAM_FAILED" |
        "TEAM_COLLABORATION_HANDOFF_FAILED" | "TEAM_REQUEST_PREPARATION_FAILED") {
        return None;
    }
    let (reference, metadata) = tail.split_once("; diagnostic metadata ")
        .map_or((tail, None), |(reference, data)| (reference, Some(data)));
    let reference = uuid::Uuid::parse_str(reference).ok()?.to_string();
    let mut result = TeamExecutionDiagnostic {
        code: code.into(), field: None, retryable: false,
        action: if code == "TEAM_COLLABORATION_HANDOFF_FAILED" { "reconcile" } else { "change-settings" }.into(),
        protected_diagnostic_ref: Some(reference),
        source_stage: None, category: None, http_status: None, collaboration_code: None,
    };
    if let Some(metadata) = metadata {
        let value: SafeMetadata = serde_json::from_str(metadata).ok()?;
        if value.source_stage.as_deref().is_some_and(|stage| !matches!(stage,
            "request-preparation" | "handoff-validation" | "provider-opening" |
            "handoff-recording" | "provider-stream"))
            || value.category.as_deref().is_some_and(|category| !matches!(category,
                "provider_authentication_failed" | "provider_invalid_request" |
                "provider_rate_limited" | "provider_overloaded" | "provider_timeout" |
                "provider_transport_failed" | "provider_stream_failed" |
                "provider_budget_exceeded" | "provider_external_error" |
                "provider_internal_error" | "collaboration_invalid" |
                "collaboration_not_found" | "collaboration_conflict" |
                "collaboration_storage" | "request_preparation_failed" | "unclassified"))
            || value.http_status.is_some_and(|status| !(100..=599).contains(&status))
            || value.collaboration_code.as_deref().is_some_and(|code| collaboration_code(code).is_none())
        {
            return None;
        }
        result.source_stage = value.source_stage;
        result.category = value.category;
        result.http_status = value.http_status;
        result.collaboration_code = value.collaboration_code;
    }
    Some(result)
}
