use crate::uar::a2ui::realtime::{A2uiReplayBackbone, surface_message_to_state_patch};
use crate::uar::domain::events::{ArtifactPayload, NormalizedEvent, RuntimeEventSink};

/// Last host event boundary for UI projections, including non-tool producers.
/// An absent admission capture is not a legacy compatibility grant.
pub(crate) fn enforce_output_ceiling(
    event: NormalizedEvent,
    snapshot: Option<&super::presentations::RunPresentationSnapshot>,
) -> NormalizedEvent {
    // Only the history-owned projection below this gate may publish provenance.
    if let NormalizedEvent::StatePatch { run_id, patch } = &event
        && patch.iter().any(|op| {
            op.path == "/presentation"
                || op.path.starts_with("/presentation/")
                || (matches!(op.path.as_str(), "" | "/")
                    && (!matches!(op.op.as_str(), "add" | "replace")
                        || !op.value.as_ref().is_some_and(serde_json::Value::is_object)
                        || op
                            .value
                            .as_ref()
                            .is_some_and(|value| value.get("presentation").is_some())))
        })
    {
        return NormalizedEvent::PresentationDiagnostic {
            run_id: run_id.clone(),
            code: "reserved_presentation_provenance".into(),
            message: "Run provenance is owned by the host and cannot be replaced by a producer"
                .into(),
        };
    }
    // These names belong exclusively to the client's artifact projection.
    // Provider tool announcements arrive before execution validation and must
    // never be able to impersonate that projection, even in legacy mode.
    if let NormalizedEvent::ToolStart { run_id, tool, .. } = &event
        && matches!(tool.as_str(), "__a2ui_input__" | "__a2ui_display__")
    {
        return NormalizedEvent::PresentationDiagnostic {
            run_id: run_id.clone(),
            code: "reserved_artifact_projection".into(),
            message: "Tool calls cannot use reserved artifact projection names".into(),
        };
    }
    if snapshot.is_some_and(|snapshot| snapshot.selection().allows_surfaces()) {
        return event;
    }
    let surface_artifact = |artifact: &ArtifactPayload| {
        matches!(artifact.artifact_type.as_str(), "a2ui" | "display")
            || artifact.artifact_type.starts_with("a2ui/")
            || matches!(
                artifact.language.as_deref(),
                Some("a2ui" | "application/a2ui+json" | "application/vnd.uar.a2ui+json")
            )
            || artifact
                .metadata
                .get("profile")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|profile| profile.starts_with("uar.a2ui/"))
    };
    let run_id = match &event {
        NormalizedEvent::Artifact { run_id, artifact }
        | NormalizedEvent::ArtifactDisplay { run_id, artifact }
            if surface_artifact(artifact) =>
        {
            run_id
        }
        NormalizedEvent::ArtifactInputRequest { run_id, .. } => run_id,
        NormalizedEvent::StatePatch { run_id, patch }
            if patch.iter().any(|op| {
                op.path == "/a2ui"
                    || op.path.starts_with("/a2ui/")
                    || (matches!(op.path.as_str(), "" | "/")
                        && op
                            .value
                            .as_ref()
                            .is_some_and(|value| value.get("a2ui").is_some()))
            }) =>
        {
            run_id
        }
        _ => return event,
    };
    NormalizedEvent::PresentationDiagnostic {
        run_id: run_id.clone(),
        code: "presentation_output_ceiling".into(),
        message: "Surface publication is not permitted by this run's captured output mode".into(),
    }
}

/// Publish the validated state and artifact projection produced by the native
/// renderer tools. Callers publish `ToolEnd` only after this returns.
pub(crate) async fn publish_tool_output(
    run_id: &str,
    tool: &str,
    success: bool,
    call_id: &str,
    snapshot: &super::presentations::RunPresentationSnapshot,
    cancellation: &tokio_util::sync::CancellationToken,
    backbone: &dyn A2uiReplayBackbone,
    events: &dyn RuntimeEventSink,
) {
    if !matches!(tool, "a2ui_render" | "presentation_render") {
        return;
    }
    if !success {
        return;
    }

    // Preparation is not a publication grant. Check the immutable output
    // ceiling and validate the entire batch before touching replay or events.
    let mut prepared_on_host = false;
    let admitted = (|| -> anyhow::Result<_> {
        let output = snapshot.take_preparation(call_id, tool)?;
        prepared_on_host = true;
        anyhow::ensure!(
            snapshot.selection().allows_surfaces(),
            "This run permits text output only"
        );
        anyhow::ensure!(
            !cancellation.is_cancelled(),
            "Run cancelled before surface publication"
        );
        let identity = if tool == "presentation_render" {
            Some(snapshot.prepared_identity(&output)?)
        } else {
            None
        };
        let messages = output
            .get("a2uiMessages")
            .and_then(serde_json::Value::as_array)
            .filter(|messages| !messages.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("Renderer output must contain a nonempty a2uiMessages array")
            })?;
        let messages = messages
            .iter()
            .cloned()
            .map(crate::uar::a2ui::protocol::parse_message)
            .collect::<Result<Vec<_>, _>>()
            .map_err(anyhow::Error::msg)?;
        Ok((identity, messages))
    })();
    let (identity, messages) = match admitted {
        Ok(value) => value,
        Err(error) => {
            if prepared_on_host {
                snapshot.record_generation_failure();
            }
            events
                .emit(NormalizedEvent::PresentationDiagnostic {
                    run_id: run_id.to_owned(),
                    code: "a2ui_publication_rejected".into(),
                    message: error.to_string(),
                })
                .await;
            return;
        }
    };

    let mut source = Vec::new();
    let mut surface_ids = Vec::new();
    for message in messages {
        if !surface_ids.contains(&message.surface_id) {
            surface_ids.push(message.surface_id.clone());
        }
        let op = surface_message_to_state_patch(&message.surface_id, message.kind, message.payload);
        backbone.publish(run_id, op.clone());
        events
            .emit(NormalizedEvent::StatePatch {
                run_id: run_id.to_string(),
                patch: vec![op],
            })
            .await;
        source.push(message.raw.to_string());
    }

    if !source.is_empty() {
        if let Some(identity) = &identity
            && let Err(error) = snapshot.record_template_publication(identity)
        {
            events
                .emit(NormalizedEvent::PresentationDiagnostic {
                    run_id: run_id.to_owned(),
                    code: "presentation_receipt_unavailable".into(),
                    message: error.to_string(),
                })
                .await;
        }
        events
            .emit(NormalizedEvent::ArtifactDisplay {
                run_id: run_id.to_string(),
                artifact: ArtifactPayload {
                    artifact_id: format!("a2ui:{}", uuid::Uuid::new_v4()),
                    artifact_type: "a2ui".to_string(),
                    title: "Interactive UI".to_string(),
                    content: source.join("\n"),
                    language: Some("application/a2ui+json".to_string()),
                    metadata: serde_json::json!({
                        "profile": "uar.a2ui/1",
                        "surfaceIds": surface_ids,
                        "sourceTool": tool,
                        "presentation": identity,
                        "publication_status": "published",
                    }),
                },
            })
            .await;
    }
}
