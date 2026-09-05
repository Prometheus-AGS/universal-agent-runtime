//! Immutable Presentation contents captured by the trusted run host.
//! Kernels may inspect this data; capture does not expose any mutation facility.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::uar::a2ui::presentation_selection::{
    EffectivePresentationMode, PresentationFallbackReason, PresentationNegotiation,
    PresentationSelection,
};
use crate::uar::a2ui::presentations::Presentation;
use crate::uar::domain::policy::EffectiveRunPolicy;
use crate::uar::persistence::PersistenceLayer;

use super::actor::messages::ActorOwner;

#[cfg(test)]
#[path = "presentations_tests.rs"]
mod tests;

pub(crate) struct RunPresentationSnapshot {
    owner: Option<ActorOwner>,
    templates: BTreeMap<String, Presentation>,
    negotiation: PresentationNegotiation,
    selection: PresentationSelection,
    preparations: std::sync::Mutex<BTreeMap<String, (String, serde_json::Value)>>,
    publications: std::sync::Mutex<BTreeMap<String, u64>>,
    generation_failed: std::sync::atomic::AtomicBool,
}

/// History-owned observation. Template receipts are read from the trusted host
/// snapshot, never from event metadata or model-controlled tool output.
#[derive(Debug, serde::Serialize)]
pub(crate) struct PresentationObservation {
    version: u32,
    requested_mode: Option<crate::uar::a2ui::presentation_selection::PresentationMode>,
    effective_mode: EffectivePresentationMode,
    admission_fallback_reason: Option<PresentationFallbackReason>,
    fallback_reason: Option<PresentationFallbackReason>,
    run_outcome: &'static str,
    eligible_templates: Vec<serde_json::Value>,
    published_templates: Vec<serde_json::Value>,
    surface_published: bool,
    generation_failed: bool,
    receipt_status: &'static str,
    client_display: &'static str,
}

impl PresentationObservation {
    pub(crate) fn new(snapshot: &RunPresentationSnapshot) -> Self {
        Self {
            version: 1,
            requested_mode: snapshot.selection.requested_mode,
            effective_mode: snapshot.selection.effective_mode,
            admission_fallback_reason: snapshot.selection.fallback_reason,
            fallback_reason: snapshot.selection.fallback_reason,
            run_outcome: "running",
            eligible_templates: snapshot.identities(),
            published_templates: Vec::new(),
            surface_published: false,
            generation_failed: false,
            receipt_status: "available",
            client_display: "unconfirmed",
        }
    }

    pub(crate) fn observe(
        &mut self,
        event: &crate::uar::domain::events::NormalizedEvent,
        snapshot: &RunPresentationSnapshot,
    ) {
        use crate::uar::domain::events::NormalizedEvent;
        match event {
            NormalizedEvent::StatePatch { patch, .. }
                if patch.iter().any(|op| {
                    op.path.starts_with("/a2ui/surfaces/")
                        && matches!(op.op.as_str(), "add" | "replace")
                }) =>
            {
                self.surface_published = true
            }
            NormalizedEvent::ArtifactDisplay { artifact, .. }
                if artifact.artifact_type == "a2ui"
                    && artifact
                        .metadata
                        .get("profile")
                        .and_then(serde_json::Value::as_str)
                        == Some(crate::uar::a2ui::protocol::PROFILE) =>
            {
                self.surface_published = true
            }
            NormalizedEvent::RunDone { .. } | NormalizedEvent::RunDoneWithUsage { .. }
                if self.run_outcome == "running" =>
            {
                self.run_outcome = "finished"
            }
            NormalizedEvent::Cancelled { .. } => self.run_outcome = "cancelled",
            NormalizedEvent::Error { .. } => self.run_outcome = "failed",
            _ => {}
        }
        self.generation_failed = snapshot
            .generation_failed
            .load(std::sync::atomic::Ordering::Acquire);
        match snapshot.publications.lock() {
            Ok(publications) => {
                self.published_templates = publications.iter().map(|(id, revision)| {
                    serde_json::json!({ "template_id": id, "revision": revision })
                }).collect();
            }
            Err(_) => self.receipt_status = "unavailable",
        }
        self.fallback_reason = self.admission_fallback_reason;
        if self.run_outcome == "finished"
            && !self.surface_published
            && self.receipt_status == "available"
            && !matches!(
                self.effective_mode,
                EffectivePresentationMode::Text | EffectivePresentationMode::Legacy
            )
        {
            self.fallback_reason = Some(if self.generation_failed {
                PresentationFallbackReason::SurfaceGenerationFailed
            } else {
                PresentationFallbackReason::NoSurfacePublished
            });
        }
    }
}

impl std::fmt::Debug for RunPresentationSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunPresentationSnapshot")
            .field("template_count", &self.templates.len())
            .field("selection", &self.selection)
            .finish_non_exhaustive()
    }
}

impl RunPresentationSnapshot {
    pub(crate) async fn capture(
        persistence: Option<&Arc<dyn PersistenceLayer>>,
        owner: Option<ActorOwner>,
        policy: &EffectiveRunPolicy,
        negotiation: PresentationNegotiation,
    ) -> (Self, Vec<String>) {
        let (mut templates, warnings) =
            crate::uar::persistence::presentations::eligible_presentation_records(
                persistence,
                owner.as_ref(),
            )
            .await;
        templates.retain(|id, _| policy.presentations.ids.contains(id));
        let selection = negotiation.resolve(!templates.is_empty());
        (
            Self {
                owner,
                templates,
                negotiation,
                selection,
                preparations: Default::default(),
                publications: Default::default(),
                generation_failed: Default::default(),
            },
            warnings,
        )
    }

    /// A child reuses admitted contents even if storage has since changed.
    pub(crate) fn narrow(&self, policy: &EffectiveRunPolicy) -> Self {
        let templates: BTreeMap<_, _> = self
            .templates
            .iter()
            .filter(|(id, _)| policy.presentations.ids.contains(id))
            .map(|(id, record)| (id.clone(), record.clone()))
            .collect();
        let mut selection = self.negotiation.resolve(!templates.is_empty());
        selection.restrict_to_parent(&self.selection);
        Self {
            owner: self.owner.clone(),
            templates,
            negotiation: self.negotiation.clone(),
            selection,
            preparations: Default::default(),
            publications: Default::default(),
            generation_failed: Default::default(),
        }
    }

    pub(crate) fn owner(&self) -> Option<&ActorOwner> {
        self.owner.as_ref()
    }
    pub(crate) fn contains(&self, id: &str) -> bool {
        self.templates.contains_key(id)
    }
    pub(crate) fn selection(&self) -> &PresentationSelection {
        &self.selection
    }
    pub(crate) fn negotiation(&self) -> &PresentationNegotiation {
        &self.negotiation
    }

    /// A peer receives an output ceiling, never the source's template contents.
    pub(crate) fn delegation_negotiation(&self) -> Option<PresentationNegotiation> {
        if self.negotiation.presentation_mode.is_none()
            && self.negotiation.client_rendering.is_none()
        {
            return None;
        }
        let mut negotiation = self.negotiation.clone();
        if !self.selection.allows_surfaces() {
            negotiation.presentation_mode =
                Some(crate::uar::a2ui::presentation_selection::PresentationMode::Text);
        }
        Some(negotiation)
    }

    /// Public run context carries identities/revisions, never a display claim.
    pub(crate) fn identities(&self) -> Vec<serde_json::Value> {
        self.templates
            .values()
            .map(|record| {
                serde_json::json!({
                    "presentation_id": record.id,
                    "revision": record.revision,
                })
            })
            .collect()
    }

    pub(crate) fn has_templates(&self) -> bool {
        !self.templates.is_empty()
    }

    pub(crate) fn record_generation_failure(&self) {
        self.generation_failed
            .store(true, std::sync::atomic::Ordering::Release);
    }

    /// Called after the host has published a consumed preparation's validated
    /// message batch. Generic event metadata cannot add a template receipt.
    pub(crate) fn record_template_publication(
        &self,
        identity: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let id = identity["template_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Publication omitted template identity"))?;
        let record = self
            .templates
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Published template is outside the run snapshot"))?;
        anyhow::ensure!(
            identity["revision"].as_u64() == Some(record.revision),
            "Published template revision does not match the run snapshot"
        );
        self.publications
            .lock()
            .map_err(|_| anyhow::anyhow!("Publication receipts unavailable"))?
            .insert(id.to_owned(), record.revision);
        Ok(())
    }

    /// Called only by the trusted native execution boundary, before history
    /// formatting. Models never supply this receipt through a ToolResult.
    pub(crate) fn retain_preparation(
        &self,
        call_id: &str,
        tool: &str,
        output: serde_json::Value,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            !call_id.is_empty(),
            "Surface preparation has no tool call identity"
        );
        let mut receipts = self
            .preparations
            .lock()
            .map_err(|_| anyhow::anyhow!("Surface preparation receipts unavailable"))?;
        anyhow::ensure!(
            !receipts.contains_key(call_id),
            "Duplicate surface preparation call identity"
        );
        receipts.insert(call_id.to_owned(), (tool.to_owned(), output));
        Ok(())
    }

    pub(crate) fn take_preparation(
        &self,
        call_id: &str,
        tool: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let mut receipts = self
            .preparations
            .lock()
            .map_err(|_| anyhow::anyhow!("Surface preparation receipts unavailable"))?;
        let (source, output) = receipts
            .remove(call_id)
            .ok_or_else(|| anyhow::anyhow!("No host preparation receipt for this tool call"))?;
        anyhow::ensure!(source == tool, "Surface preparation tool identity changed");
        Ok(output)
    }

    /// Resolve a preparation receipt against the host's frozen identity, not
    /// against current storage or model-supplied revision claims.
    pub(crate) fn prepared_identity(
        &self,
        output: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let identity = &output["presentation"];
        let id = identity["template_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Template preparation omitted its identity"))?;
        let record = self
            .templates
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Prepared template is outside the run snapshot"))?;
        anyhow::ensure!(
            identity["revision"].as_u64() == Some(record.revision),
            "Prepared template revision does not match the run snapshot"
        );
        Ok(serde_json::json!({ "template_id": record.id, "revision": record.revision }))
    }

    /// Owner-authored labels remain data, separate from host output instructions.
    pub(crate) fn catalog(&self) -> serde_json::Value {
        serde_json::Value::Array(self.templates.values().map(|record| {
            let template = &record.content.template;
            let defaults = serde_json::Value::Object(template.default_data.clone());
            let paths: BTreeSet<_> = template.components.iter().flat_map(|component| {
                ["text", "value"].into_iter().filter_map(move |field| {
                    component.get(field)?.get("path")?.as_str()
                })
            }).collect();
            let bindings: Vec<_> = paths.into_iter().map(|path| {
                let mut binding = serde_json::json!({ "path": path });
                if let Some(value) = defaults.pointer(path) {
                    // Large defaults stay in the frozen template, not in the
                    // prompt. The host still applies them when data is omitted.
                    if value.to_string().len() <= 1024 {
                        binding["default"] = value.clone();
                    } else {
                        binding["default_omitted"] = serde_json::json!(true);
                    }
                }
                binding
            }).collect();
            serde_json::json!({
                "template_id": record.id, "revision": record.revision,
                "title": record.content.title, "description": record.content.description,
                "bindings": bindings,
                "data_merge": "Supply a JSON object; supplied top-level keys replace template defaults. Binding paths are JSON pointers into that object. Omitted top-level keys retain their defaults.",
            })
        }).collect())
    }

    /// Prepare declarative messages only; publication belongs to the host.
    pub(crate) fn prepare(
        &self,
        template_id: &str,
        data: &serde_json::Map<String, serde_json::Value>,
    ) -> anyhow::Result<serde_json::Value> {
        anyhow::ensure!(
            self.selection.allows_surfaces(),
            "This run permits text output only"
        );
        let record = self
            .templates
            .get(template_id)
            .ok_or_else(|| anyhow::anyhow!("Template is not in this run's eligible snapshot"))?;
        let surface_id = format!("presentation-{}", uuid::Uuid::new_v4());
        let messages = record
            .content
            .template
            .instantiate(&surface_id, data)
            .map_err(anyhow::Error::msg)?;
        Ok(serde_json::json!({
            "status": "prepared",
            "terminal": true,
            "instruction": "The template messages are prepared for host publication. Do not repeat this render request. Describe the content without claiming the client displayed it.",
            "presentation": { "template_id": record.id, "revision": record.revision },
            "a2uiMessages": messages,
        }))
    }
}
