//! Domain types for the UAR A2UI design-system bridge.
//!
//! Ported from `flint-forge/crates/fdb-app/src/a2ui/types.rs` (component
//! registry use-case types) and the schema introduced by
//! `flint-forge/migrations/0009_flint_a2ui_design_systems.sql` (design system
//! import provenance + per-design-system component overrides). Renamed to fit
//! UAR's persistence-layer conventions (`DesignSystemStore`, see `store.rs`)
//! and to drop the `flint_a2ui.` schema prefix, since UAR hosts these tables
//! under its own `design_systems` / `components` / `component_overrides`
//! tables rather than reusing flint-forge's Postgres schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Which SDK renderers support a given component.
///
/// A component with `flutter: false` is excluded from the Flutter SDK
/// catalog. A component with `htmx: false` is excluded from the HTMX
/// renderer template set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Renderers {
    pub react: bool,
    pub flutter: bool,
    pub htmx: bool,
}

impl Default for Renderers {
    fn default() -> Self {
        Self {
            react: true,
            flutter: true,
            htmx: true,
        }
    }
}

/// A single W3C Design Tokens Community Group 2024 token value.
///
/// Reference: <https://design-tokens.org/schema/2024>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignToken {
    #[serde(rename = "$value")]
    pub value: String,
    #[serde(rename = "$type")]
    pub token_type: String,
}

/// The full design token map for a design system, keyed by group then token
/// name, e.g. `{ "color": { "primary": "oklch(68% 0.21 250)" } }`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesignTokenMap(pub serde_json::Value);

impl DesignTokenMap {
    #[must_use]
    pub fn empty() -> Self {
        Self(serde_json::Value::Object(serde_json::Map::default()))
    }
}

/// How a design system's tokens/overrides were produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFormat {
    /// Parsed from a Flint `DESIGN.md` document (see `design_md_parser`).
    DesignMd,
    /// Parsed from a raw W3C Design Tokens Community Group 2024 JSON document.
    W3cTokens,
    /// Imported from Figma token export JSON. Not yet parsed by this module;
    /// reserved for a follow-up change (kept for schema/API parity with the
    /// flint-forge source).
    FigmaTokens,
    /// Hand-authored, not imported from any structured source.
    Manual,
}

impl SourceFormat {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DesignMd => "design_md",
            Self::W3cTokens => "w3c_tokens",
            Self::FigmaTokens => "figma_tokens",
            Self::Manual => "manual",
        }
    }

    #[must_use]
    pub fn from_str_lenient(s: &str) -> Self {
        match s {
            "design_md" => Self::DesignMd,
            "w3c_tokens" => Self::W3cTokens,
            "figma_tokens" => Self::FigmaTokens,
            _ => Self::Manual,
        }
    }
}

/// A design system record: a named token set with import provenance.
///
/// Mirrors `flint_a2ui.design_systems` plus the `source_format` /
/// `source_content` / `imported_at` columns added by migration `0009` in
/// flint-forge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignSystem {
    pub id: String,
    pub name: String,
    pub tokens: serde_json::Value,
    pub source_format: SourceFormat,
    /// Raw imported content (DESIGN.md text or token JSON), if any.
    pub source_content: Option<String>,
    pub imported_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A component in the base registry that design systems can override.
///
/// This is a minimal, self-contained catalog row — UAR does not yet vendor
/// flint-forge's full component catalog (that is separately scoped future
/// work; see the proposal's "Out of scope" section). It carries just enough
/// fields to host per-design-system overrides and embeddings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub slug: String,
    pub primitive_type: String,
    pub category: String,
    pub schema: serde_json::Value,
    pub description: Option<String>,
    pub usage_examples: Option<serde_json::Value>,
    #[serde(default)]
    pub renderers: Renderers,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Per-design-system override of a base component's props/CSS/renderer
/// wiring. Mirrors `flint_a2ui.component_overrides` from migration `0009`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOverrideRecord {
    pub id: String,
    pub design_system_id: String,
    pub component_id: String,
    pub prop_defaults: serde_json::Value,
    pub css_vars: serde_json::Value,
    pub react_component: Option<String>,
    pub flutter_widget: Option<String>,
    pub htmx_template: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A component definition merged with any per-design-system override,
/// equivalent to flint-forge's `flint_a2ui.resolve_components_with_overrides()`
/// SQL function result row (`fdb-app`'s `ResolvedComponent`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedComponent {
    pub slug: String,
    pub primitive_type: String,
    pub category: String,
    pub schema: serde_json::Value,
    pub description: Option<String>,
    pub renderers: Renderers,
    /// Merged prop defaults from `component_overrides` (empty object if none).
    pub prop_defaults: serde_json::Value,
    /// Merged CSS variables from `component_overrides` (empty object if none).
    pub css_vars: serde_json::Value,
    /// Overridden React component import path (None = use SDK default).
    pub react_component: Option<String>,
    /// Overridden Flutter widget class name (None = use SDK default).
    pub flutter_widget: Option<String>,
    /// Overridden Askama/HTMX template path (None = use SDK default).
    pub htmx_template: Option<String>,
}

/// Merge a base [`Component`] with an optional [`ComponentOverrideRecord`],
/// replicating the `LEFT JOIN` + `COALESCE` semantics of
/// `flint_a2ui.resolve_components_with_overrides()`.
#[must_use]
pub fn resolve_component(
    base: &Component,
    ov: Option<&ComponentOverrideRecord>,
) -> ResolvedComponent {
    let empty_obj = || serde_json::Value::Object(serde_json::Map::new());
    ResolvedComponent {
        slug: base.slug.clone(),
        primitive_type: base.primitive_type.clone(),
        category: base.category.clone(),
        schema: base.schema.clone(),
        description: base.description.clone(),
        renderers: base.renderers.clone(),
        prop_defaults: ov.map_or_else(empty_obj, |o| o.prop_defaults.clone()),
        css_vars: ov.map_or_else(empty_obj, |o| o.css_vars.clone()),
        react_component: ov.and_then(|o| o.react_component.clone()),
        flutter_widget: ov.and_then(|o| o.flutter_widget.clone()),
        htmx_template: ov.and_then(|o| o.htmx_template.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_component() -> Component {
        let now = Utc::now();
        Component {
            id: "c1".into(),
            slug: "button".into(),
            primitive_type: "Button".into(),
            category: "input".into(),
            schema: serde_json::json!({"properties": {"variant": {"type": "string"}}}),
            description: Some("A button".into()),
            usage_examples: None,
            renderers: Renderers::default(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn resolve_without_override_returns_empty_defaults() {
        let base = base_component();
        let resolved = resolve_component(&base, None);
        assert_eq!(resolved.prop_defaults, serde_json::json!({}));
        assert_eq!(resolved.css_vars, serde_json::json!({}));
        assert!(resolved.react_component.is_none());
    }

    #[test]
    fn resolve_with_override_merges_fields() {
        let base = base_component();
        let now = Utc::now();
        let ov = ComponentOverrideRecord {
            id: "ov1".into(),
            design_system_id: "ds1".into(),
            component_id: base.id.clone(),
            prop_defaults: serde_json::json!({"variant": "primary"}),
            css_vars: serde_json::json!({"--btn-bg": "#1d4ed8"}),
            react_component: Some("@acme/ui/Button".into()),
            flutter_widget: None,
            htmx_template: None,
            created_at: now,
            updated_at: now,
        };
        let resolved = resolve_component(&base, Some(&ov));
        assert_eq!(resolved.prop_defaults["variant"], "primary");
        assert_eq!(resolved.css_vars["--btn-bg"], "#1d4ed8");
        assert_eq!(resolved.react_component.as_deref(), Some("@acme/ui/Button"));
        assert!(resolved.flutter_widget.is_none());
    }

    #[test]
    fn source_format_round_trips_through_str() {
        for f in [
            SourceFormat::DesignMd,
            SourceFormat::W3cTokens,
            SourceFormat::FigmaTokens,
            SourceFormat::Manual,
        ] {
            assert_eq!(SourceFormat::from_str_lenient(f.as_str()), f);
        }
    }
}
