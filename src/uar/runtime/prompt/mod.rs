//! Typed, deterministic prompt assembly.

pub mod assemble;
pub mod fragment;
pub mod interrupted;
pub mod manifest;

pub use assemble::{
    PromptSection, PromptTemplateError, PromptTemplateLayout, PromptTemplateProfile,
    PromptTemplateSelector, RenderOptions, render, render_with_options, render_with_template,
};
pub use fragment::{Authority, PromptFragment, PromptRole, Retention};
pub use interrupted::{TurnInterrupted, TurnInterruptionReason};
pub use manifest::{FragmentCounts, ManifestFragment, PromptBudgets, TurnManifest};
