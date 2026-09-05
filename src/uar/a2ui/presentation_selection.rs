//! Client rendering negotiation is an output restriction, never resource authority.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::protocol::PROFILE;

/// Requested presentation behavior. Only omission selects legacy compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PresentationMode {
    Auto,
    Text,
    A2ui,
    Hybrid,
}

/// Declarative renderer support; unknown profiles do not grant compatibility.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClientRenderingSupport {
    /// Profiles the client claims it can render, not proof of actual display.
    #[serde(default)]
    pub a2ui_profiles: Vec<String>,
}

/// Optional wire extensions shared by HTTP and host-owned execution requests.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PresentationNegotiation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation_mode: Option<PresentationMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_rendering: Option<ClientRenderingSupport>,
}

/// Admission intent. This does not assert that a surface was published or displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffectivePresentationMode {
    Legacy,
    Auto,
    Text,
    A2ui,
    Hybrid,
}

/// Machine-readable explanation for a narrower output decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PresentationFallbackReason {
    ClientRenderingNotDeclared,
    IncompatibleProfile,
    NoEligibleTemplates,
    ParentTextCeiling,
    NoSurfacePublished,
    SurfaceGenerationFailed,
}

/// Resolved mode and explanation retained independently from publication receipts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PresentationSelection {
    /// None means neither negotiation field was supplied.
    pub requested_mode: Option<PresentationMode>,
    pub effective_mode: EffectivePresentationMode,
    pub fallback_reason: Option<PresentationFallbackReason>,
}

impl PresentationNegotiation {
    /// Resolve support and eligibility already established by the trusted host.
    /// Client profile claims never make an inaccessible template eligible.
    #[must_use]
    pub fn resolve(&self, has_eligible_templates: bool) -> PresentationSelection {
        if self.presentation_mode.is_none() && self.client_rendering.is_none() {
            return PresentationSelection {
                requested_mode: None,
                effective_mode: EffectivePresentationMode::Legacy,
                fallback_reason: None,
            };
        }
        let requested = self.presentation_mode.unwrap_or(PresentationMode::Auto);
        let fallback = if requested == PresentationMode::Text {
            None
        } else {
            match &self.client_rendering {
                None => Some(PresentationFallbackReason::ClientRenderingNotDeclared),
                Some(support)
                    if !support
                        .a2ui_profiles
                        .iter()
                        .any(|profile| profile == PROFILE) =>
                {
                    Some(PresentationFallbackReason::IncompatibleProfile)
                }
                Some(_) if !has_eligible_templates => {
                    Some(PresentationFallbackReason::NoEligibleTemplates)
                }
                Some(_) => None,
            }
        };
        PresentationSelection {
            requested_mode: Some(requested),
            effective_mode: if fallback.is_some() {
                EffectivePresentationMode::Text
            } else {
                match requested {
                    PresentationMode::Auto => EffectivePresentationMode::Auto,
                    PresentationMode::Text => EffectivePresentationMode::Text,
                    PresentationMode::A2ui => EffectivePresentationMode::A2ui,
                    PresentationMode::Hybrid => EffectivePresentationMode::Hybrid,
                }
            },
            fallback_reason: fallback,
        }
    }
}

impl PresentationSelection {
    /// Whether this captured mode permits surface publication under host policy.
    #[must_use]
    pub fn allows_surfaces(&self) -> bool {
        self.effective_mode != EffectivePresentationMode::Text
    }

    /// Constrain descendants without upgrading a legacy or negotiated request.
    pub fn restrict_to_parent(&mut self, parent: &Self) {
        if !parent.allows_surfaces() && self.allows_surfaces() {
            self.effective_mode = EffectivePresentationMode::Text;
            self.fallback_reason = Some(PresentationFallbackReason::ParentTextCeiling);
        }
    }

    /// Output instructions are host intent, not evidence that the model complied.
    #[must_use]
    pub fn output_guidance(&self) -> Option<&'static str> {
        match self.effective_mode {
            EffectivePresentationMode::Legacy => None,
            EffectivePresentationMode::Text => {
                Some("Respond in readable text. Do not generate or publish UI surfaces.")
            }
            EffectivePresentationMode::Auto => Some(
                "Use an eligible UI template when useful; otherwise respond in readable text. Do not claim a surface was displayed.",
            ),
            EffectivePresentationMode::A2ui => Some(
                "Prefer an eligible UI surface and include a brief accessible textual summary. If no surface can be published, provide a readable text answer. Do not claim client display.",
            ),
            EffectivePresentationMode::Hybrid => Some(
                "Provide substantive readable text and an eligible UI surface. If no surface can be published, preserve the text answer. Do not claim client display.",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_two_omitted_fields_select_legacy() {
        for eligible in [false, true] {
            let legacy = PresentationNegotiation::default().resolve(eligible);
            assert_eq!(legacy.requested_mode, None);
            assert_eq!(legacy.effective_mode, EffectivePresentationMode::Legacy);
            assert_eq!(legacy.output_guidance(), None);
        }
        let support_only = PresentationNegotiation {
            client_rendering: Some(ClientRenderingSupport {
                a2ui_profiles: vec![PROFILE.into()],
            }),
            ..Default::default()
        }
        .resolve(true);
        assert_eq!(support_only.requested_mode, Some(PresentationMode::Auto));
        assert_eq!(support_only.effective_mode, EffectivePresentationMode::Auto);
    }

    #[test]
    fn negotiated_mode_support_and_eligibility_matrix() {
        for (mode, effective) in [
            (PresentationMode::Auto, EffectivePresentationMode::Auto),
            (PresentationMode::A2ui, EffectivePresentationMode::A2ui),
            (PresentationMode::Hybrid, EffectivePresentationMode::Hybrid),
        ] {
            for (support, fallback) in [
                (
                    None,
                    Some(PresentationFallbackReason::ClientRenderingNotDeclared),
                ),
                (
                    Some(ClientRenderingSupport::default()),
                    Some(PresentationFallbackReason::IncompatibleProfile),
                ),
                (
                    Some(ClientRenderingSupport {
                        a2ui_profiles: vec!["uar.a2ui/unknown".into()],
                    }),
                    Some(PresentationFallbackReason::IncompatibleProfile),
                ),
                (
                    Some(ClientRenderingSupport {
                        a2ui_profiles: vec![PROFILE.into()],
                    }),
                    None,
                ),
            ] {
                for eligible in [false, true] {
                    let selection = PresentationNegotiation {
                        presentation_mode: Some(mode),
                        client_rendering: support.clone(),
                    }
                    .resolve(eligible);
                    let expected_fallback = fallback
                        .or((!eligible).then_some(PresentationFallbackReason::NoEligibleTemplates));
                    assert_eq!(selection.fallback_reason, expected_fallback);
                    assert_eq!(
                        selection.effective_mode,
                        if expected_fallback.is_some() {
                            EffectivePresentationMode::Text
                        } else {
                            effective
                        }
                    );
                    assert_eq!(selection.requested_mode, Some(mode));
                }
            }
        }
    }

    #[test]
    fn explicit_text_and_parent_ceiling_cannot_be_widened() {
        let text = PresentationNegotiation {
            presentation_mode: Some(PresentationMode::Text),
            client_rendering: None,
        }
        .resolve(true);
        assert_eq!(text.effective_mode, EffectivePresentationMode::Text);
        assert_eq!(text.fallback_reason, None);
        for negotiation in [
            PresentationNegotiation::default(),
            PresentationNegotiation {
                presentation_mode: Some(PresentationMode::Hybrid),
                client_rendering: Some(ClientRenderingSupport {
                    a2ui_profiles: vec![PROFILE.into()],
                }),
            },
        ] {
            let mut child = negotiation.resolve(true);
            child.restrict_to_parent(&text);
            assert!(!child.allows_surfaces());
            assert_eq!(
                child.fallback_reason,
                Some(PresentationFallbackReason::ParentTextCeiling)
            );
        }
    }
}
