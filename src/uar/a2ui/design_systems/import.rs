//! Import pipeline — turns a parsed [`DesignMd`] document (or raw W3C tokens)
//! into a persisted [`DesignSystem`] plus per-component
//! [`ComponentOverrideRecord`]s.
//!
//! This is UAR-native glue: flint-forge's application layer
//! (`flint-forge/crates/fdb-app/src/a2ui/`) only defines the parser and
//! domain types, and does not implement an end-to-end "apply a parsed
//! DESIGN.md to the catalog" use case in a single function — that logic
//! lives in flint-forge's interface layer, which is out of scope for this
//! change. `import_design_md` below is written for UAR to make the ported
//! parser + types actually usable end-to-end.

use chrono::Utc;
use uuid::Uuid;

use super::design_md_parser::{DesignMd, ParseError, parse as parse_design_md};
use super::store::DesignSystemStore;
use super::types::{ComponentOverrideRecord, DesignSystem, SourceFormat};

/// Errors from the design-system import pipeline.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("failed to parse DESIGN.md: {0}")]
    Parse(#[from] ParseError),

    #[error("storage error: {0}")]
    Store(#[from] anyhow::Error),
}

/// Result of importing a `DesignMd` document: the persisted design system
/// plus a report of which component overrides were applied vs. skipped
/// (skipped because no base component with a matching slug exists in the
/// catalog yet).
#[derive(Debug, Clone)]
pub struct ImportReport {
    pub design_system: DesignSystem,
    pub applied_overrides: usize,
    pub skipped_slugs: Vec<String>,
}

/// Parse `raw_markdown` as a DESIGN.md document and persist it as a new
/// [`DesignSystem`], applying any §5 component overrides to base components
/// already present in `store` (matched by slug). Overrides referencing an
/// unknown slug are skipped and reported in
/// [`ImportReport::skipped_slugs`] rather than failing the whole import —
/// this mirrors the graceful-degradation behavior flint-forge documents for
/// its embedder ("semantic search degrades gracefully to text search").
///
/// # Errors
///
/// Returns [`ImportError::Parse`] if `raw_markdown` is not a valid
/// DESIGN.md document, or [`ImportError::Store`] if persistence fails.
pub async fn import_design_md(
    store: &dyn DesignSystemStore,
    raw_markdown: &str,
) -> Result<ImportReport, ImportError> {
    let doc: DesignMd = parse_design_md(raw_markdown)?;
    let now = Utc::now();

    let design_system = DesignSystem {
        id: Uuid::new_v4().to_string(),
        name: doc.name.clone(),
        tokens: doc.tokens.clone(),
        source_format: SourceFormat::DesignMd,
        source_content: Some(raw_markdown.to_string()),
        imported_at: Some(now),
        created_at: now,
        updated_at: now,
    };
    store.put_design_system(design_system.clone()).await?;

    let mut applied_overrides = 0usize;
    let mut skipped_slugs = Vec::new();

    for ov in doc.component_overrides {
        match store.get_component_by_slug(&ov.slug).await? {
            Some(component) => {
                let record = ComponentOverrideRecord {
                    id: Uuid::new_v4().to_string(),
                    design_system_id: design_system.id.clone(),
                    component_id: component.id,
                    prop_defaults: ov.prop_defaults,
                    css_vars: ov.css_vars,
                    react_component: ov.react_component,
                    flutter_widget: ov.flutter_widget,
                    htmx_template: ov.htmx_template,
                    created_at: now,
                    updated_at: now,
                };
                store.put_component_override(record).await?;
                applied_overrides += 1;
            }
            None => skipped_slugs.push(ov.slug),
        }
    }

    Ok(ImportReport {
        design_system,
        applied_overrides,
        skipped_slugs,
    })
}

/// Import a raw W3C Design Tokens Community Group 2024 JSON document as a
/// new design system (no component overrides — W3C token exports carry no
/// component-level information).
///
/// # Errors
///
/// Returns [`ImportError::Parse`] if `raw_json` is not valid JSON, wrapped
/// via [`ParseError::InvalidJson`] with section `0` (no DESIGN.md section
/// applies to a pure token import).
pub async fn import_w3c_tokens(
    store: &dyn DesignSystemStore,
    name: &str,
    raw_json: &str,
) -> Result<DesignSystem, ImportError> {
    let tokens = super::design_md_parser::map_w3c_tokens(raw_json)
        .map_err(|source| ImportError::Parse(ParseError::InvalidJson { section: 0, source }))?;
    let now = Utc::now();
    let design_system = DesignSystem {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        tokens,
        source_format: SourceFormat::W3cTokens,
        source_content: Some(raw_json.to_string()),
        imported_at: Some(now),
        created_at: now,
        updated_at: now,
    };
    store.put_design_system(design_system.clone()).await?;
    Ok(design_system)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uar::a2ui::design_systems::store::InMemoryDesignSystemStore;
    use crate::uar::a2ui::design_systems::types::{Component, Renderers};

    const SAMPLE: &str = r##"
# Design System — Acme Corp

## 1. Color

```json
{ "primary": "#2563eb" }
```

## 5. Components

### button

```json
{ "variant": "primary" }
```

```json
{ "--btn-bg": "#1d4ed8" }
```

### unknown-widget

```json
{ "foo": "bar" }
```
"##;

    fn component(id: &str, slug: &str) -> Component {
        let now = chrono::Utc::now();
        Component {
            id: id.to_string(),
            slug: slug.to_string(),
            primitive_type: "Button".into(),
            category: "input".into(),
            schema: serde_json::json!({}),
            description: None,
            usage_examples: None,
            renderers: Renderers::default(),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn imports_design_system_and_applies_known_overrides() {
        let store = InMemoryDesignSystemStore::new();
        store
            .put_component(component("c1", "button"))
            .await
            .unwrap();

        let report = import_design_md(&store, SAMPLE).await.unwrap();

        assert_eq!(report.design_system.name, "Acme Corp");
        assert_eq!(report.applied_overrides, 1);
        assert_eq!(report.skipped_slugs, vec!["unknown-widget".to_string()]);

        let overrides = store
            .list_component_overrides(&report.design_system.id)
            .await
            .unwrap();
        assert_eq!(overrides.len(), 1);
        assert_eq!(overrides[0].component_id, "c1");
        assert_eq!(overrides[0].css_vars["--btn-bg"], "#1d4ed8");

        let persisted = store
            .get_design_system(&report.design_system.id)
            .await
            .unwrap()
            .expect("design system persisted");
        assert_eq!(persisted.tokens["color"]["primary"], "#2563eb");
    }

    #[tokio::test]
    async fn imports_w3c_tokens_without_overrides() {
        let store = InMemoryDesignSystemStore::new();
        let raw = r##"{"color": {"primary": {"$value": "#2563eb"}}}"##;
        let ds = import_w3c_tokens(&store, "Acme W3C", raw).await.unwrap();
        assert_eq!(ds.tokens["color"]["primary"], "#2563eb");
        assert_eq!(ds.source_format, SourceFormat::W3cTokens);
    }
}
