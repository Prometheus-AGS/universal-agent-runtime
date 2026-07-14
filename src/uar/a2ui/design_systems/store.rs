//! Storage for design systems, their base component catalog, and
//! per-design-system component overrides.
//!
//! Mirrors the `CredentialStore` pattern in
//! `src/uar/security/credentials/store.rs`: a backend-agnostic
//! `#[async_trait]` [`DesignSystemStore`] trait, an always-available
//! [`InMemoryDesignSystemStore`], a SurrealDB-backed
//! [`SurrealDesignSystemStore`] that compiles in the default (`surreal-backend`)
//! build, and a Postgres-backed [`PostgresDesignSystemStore`] gated behind the
//! `sqlx` feature for Postgres deployments.
//!
//! This is a UAR-native design, not a line-for-line port: flint-forge's
//! source (`flint-forge/crates/fdb-gateway/src/a2ui_embedder.rs` +
//! `flint-forge/migrations/0009_flint_a2ui_design_systems.sql`) is
//! Postgres/pgvector-only (LISTEN/NOTIFY, an in-database `llm.embed()`
//! function). UAR's default and `server-full` release profile run on
//! SurrealDB (see `Cargo.toml`'s `minimal = ["surreal-backend"]`), so the
//! storage layer here is expressed against UAR's existing
//! `PersistenceLayer`-adjacent trait-per-backend convention instead of
//! requiring Postgres to build or run.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::types::{Component, ComponentOverrideRecord, DesignSystem};

/// Backend-agnostic storage for design systems, components, and overrides.
#[async_trait]
pub trait DesignSystemStore: Send + Sync + std::fmt::Debug {
    /// Insert or replace a design system.
    async fn put_design_system(&self, ds: DesignSystem) -> anyhow::Result<()>;

    /// Fetch a design system by id.
    async fn get_design_system(&self, id: &str) -> anyhow::Result<Option<DesignSystem>>;

    /// List all design systems, ordered by name.
    async fn list_design_systems(&self) -> anyhow::Result<Vec<DesignSystem>>;

    /// Insert or replace a base component.
    async fn put_component(&self, component: Component) -> anyhow::Result<()>;

    /// Fetch a component by id.
    async fn get_component(&self, id: &str) -> anyhow::Result<Option<Component>>;

    /// Fetch a component by slug.
    async fn get_component_by_slug(&self, slug: &str) -> anyhow::Result<Option<Component>>;

    /// List all components, ordered by slug.
    async fn list_components(&self) -> anyhow::Result<Vec<Component>>;

    /// List components that do not yet have an embedding recorded
    /// (`embed_component` writes the embedding back via `set_component_embedding`).
    async fn list_components_missing_embedding(&self) -> anyhow::Result<Vec<Component>>;

    /// Record the embedding vector for a component (idempotent upsert).
    async fn set_component_embedding(
        &self,
        component_id: &str,
        embedding: Vec<f32>,
        model: &str,
    ) -> anyhow::Result<()>;

    /// Fetch a component's recorded embedding and the model that produced it,
    /// if one has been set.
    async fn get_component_embedding(
        &self,
        component_id: &str,
    ) -> anyhow::Result<Option<(Vec<f32>, String)>>;

    /// Insert or replace a per-design-system component override.
    async fn put_component_override(&self, ov: ComponentOverrideRecord) -> anyhow::Result<()>;

    /// List all overrides for a design system.
    async fn list_component_overrides(
        &self,
        design_system_id: &str,
    ) -> anyhow::Result<Vec<ComponentOverrideRecord>>;
}

/// Convenience alias for a shared store handle.
pub type SharedDesignSystemStore = Arc<dyn DesignSystemStore>;

// ─────────────────────────────────────────────────────────────────────────────
// In-memory store — always available, used as the wired default in tests and
// non-durable deployments.
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct EmbeddingEntry {
    embedding: Vec<f32>,
    model: String,
}

#[derive(Debug, Default)]
pub struct InMemoryDesignSystemStore {
    design_systems: RwLock<HashMap<String, DesignSystem>>,
    components: RwLock<HashMap<String, Component>>,
    embeddings: RwLock<HashMap<String, EmbeddingEntry>>,
    overrides: RwLock<HashMap<String, ComponentOverrideRecord>>,
}

impl InMemoryDesignSystemStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl DesignSystemStore for InMemoryDesignSystemStore {
    async fn put_design_system(&self, ds: DesignSystem) -> anyhow::Result<()> {
        self.design_systems.write().await.insert(ds.id.clone(), ds);
        Ok(())
    }

    async fn get_design_system(&self, id: &str) -> anyhow::Result<Option<DesignSystem>> {
        Ok(self.design_systems.read().await.get(id).cloned())
    }

    async fn list_design_systems(&self) -> anyhow::Result<Vec<DesignSystem>> {
        let mut v: Vec<_> = self.design_systems.read().await.values().cloned().collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(v)
    }

    async fn put_component(&self, component: Component) -> anyhow::Result<()> {
        self.components
            .write()
            .await
            .insert(component.id.clone(), component);
        Ok(())
    }

    async fn get_component(&self, id: &str) -> anyhow::Result<Option<Component>> {
        Ok(self.components.read().await.get(id).cloned())
    }

    async fn get_component_by_slug(&self, slug: &str) -> anyhow::Result<Option<Component>> {
        Ok(self
            .components
            .read()
            .await
            .values()
            .find(|c| c.slug == slug)
            .cloned())
    }

    async fn list_components(&self) -> anyhow::Result<Vec<Component>> {
        let mut v: Vec<_> = self.components.read().await.values().cloned().collect();
        v.sort_by(|a, b| a.slug.cmp(&b.slug));
        Ok(v)
    }

    async fn list_components_missing_embedding(&self) -> anyhow::Result<Vec<Component>> {
        let embeddings = self.embeddings.read().await;
        let mut v: Vec<_> = self
            .components
            .read()
            .await
            .values()
            .filter(|c| !embeddings.contains_key(&c.id))
            .cloned()
            .collect();
        v.sort_by(|a, b| a.slug.cmp(&b.slug));
        Ok(v)
    }

    async fn set_component_embedding(
        &self,
        component_id: &str,
        embedding: Vec<f32>,
        model: &str,
    ) -> anyhow::Result<()> {
        self.embeddings.write().await.insert(
            component_id.to_string(),
            EmbeddingEntry {
                embedding,
                model: model.to_string(),
            },
        );
        Ok(())
    }

    async fn put_component_override(&self, ov: ComponentOverrideRecord) -> anyhow::Result<()> {
        self.overrides.write().await.insert(ov.id.clone(), ov);
        Ok(())
    }

    async fn list_component_overrides(
        &self,
        design_system_id: &str,
    ) -> anyhow::Result<Vec<ComponentOverrideRecord>> {
        Ok(self
            .overrides
            .read()
            .await
            .values()
            .filter(|o| o.design_system_id == design_system_id)
            .cloned()
            .collect())
    }

    async fn get_component_embedding(
        &self,
        component_id: &str,
    ) -> anyhow::Result<Option<(Vec<f32>, String)>> {
        Ok(self
            .embeddings
            .read()
            .await
            .get(component_id)
            .map(|e| (e.embedding.clone(), e.model.clone())))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SurrealDB-backed store. Compiles in the default (`surreal-backend`) build —
// the profile UAR's `server-full` release ships.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "surreal-backend")]
#[derive(Debug, Clone)]
pub struct SurrealDesignSystemStore {
    db: surrealdb::Surreal<surrealdb::engine::any::Any>,
}

#[cfg(feature = "surreal-backend")]
impl SurrealDesignSystemStore {
    #[must_use]
    pub fn new(db: surrealdb::Surreal<surrealdb::engine::any::Any>) -> Self {
        Self { db }
    }
}

#[cfg(feature = "surreal-backend")]
fn from_values<T: serde::de::DeserializeOwned>(
    rows: Vec<surrealdb::types::Value>,
) -> anyhow::Result<Vec<T>> {
    use crate::uar::persistence::providers::surreal::surreal_to_json;
    rows.into_iter()
        .map(|v| {
            let json = surreal_to_json(v)?;
            serde_json::from_value(json).map_err(|e| anyhow::anyhow!("deserialize row: {e}"))
        })
        .collect()
}

/// Serialize `value` and strip its `id` field before sending as `CONTENT` to
/// a `type::record(...)` target — the record id is already set by the query
/// (`type::record('table', $rid)`); including a duplicate plain-string `id`
/// field in the content risks colliding with SurrealDB's own record-id
/// field. Matches the existing pattern in
/// `src/uar/persistence/providers/surreal.rs` (`upsert_settings_type`,
/// `upsert_setting`), which build their `CONTENT` payloads without an `id`
/// key for the same reason. On read, `SELECT *`'s `id` field round-trips
/// back into `T`'s `id` field via `surreal_to_json`'s `RecordId` unwrapping.
#[cfg(feature = "surreal-backend")]
fn to_db_value_without_id<T: serde::Serialize>(value: &T) -> anyhow::Result<serde_json::Value> {
    let mut json = crate::uar::persistence::providers::surreal::to_db_value(value)?;
    if let serde_json::Value::Object(map) = &mut json {
        map.remove("id");
    }
    Ok(json)
}

#[cfg(feature = "surreal-backend")]
#[async_trait]
impl DesignSystemStore for SurrealDesignSystemStore {
    async fn put_design_system(&self, ds: DesignSystem) -> anyhow::Result<()> {
        let payload = to_db_value_without_id(&ds)?;
        self.db
            .query("UPSERT type::record('design_systems', $rid) CONTENT $data")
            .bind(("rid", ds.id.clone()))
            .bind(("data", payload))
            .await?;
        Ok(())
    }

    async fn get_design_system(&self, id: &str) -> anyhow::Result<Option<DesignSystem>> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::record('design_systems', $id)")
            .bind(("id", id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        Ok(from_values::<DesignSystem>(rows)?.into_iter().next())
    }

    async fn list_design_systems(&self) -> anyhow::Result<Vec<DesignSystem>> {
        let mut resp = self
            .db
            .query("SELECT * FROM design_systems ORDER BY name")
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        from_values(rows)
    }

    async fn put_component(&self, component: Component) -> anyhow::Result<()> {
        let payload = to_db_value_without_id(&component)?;
        self.db
            .query("UPSERT type::record('components', $rid) CONTENT $data")
            .bind(("rid", component.id.clone()))
            .bind(("data", payload))
            .await?;
        Ok(())
    }

    async fn get_component(&self, id: &str) -> anyhow::Result<Option<Component>> {
        let mut resp = self
            .db
            .query("SELECT * FROM type::record('components', $id)")
            .bind(("id", id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        Ok(from_values::<Component>(rows)?.into_iter().next())
    }

    async fn get_component_by_slug(&self, slug: &str) -> anyhow::Result<Option<Component>> {
        let mut resp = self
            .db
            .query("SELECT * FROM components WHERE slug = $slug LIMIT 1")
            .bind(("slug", slug.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        Ok(from_values::<Component>(rows)?.into_iter().next())
    }

    async fn list_components(&self) -> anyhow::Result<Vec<Component>> {
        let mut resp = self
            .db
            .query("SELECT * FROM components ORDER BY slug")
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        from_values(rows)
    }

    async fn list_components_missing_embedding(&self) -> anyhow::Result<Vec<Component>> {
        let all = self.list_components().await?;
        if all.is_empty() {
            return Ok(vec![]);
        }
        let mut resp = self
            .db
            .query("SELECT VALUE component_id FROM component_embeddings")
            .await?;
        let embedded: Vec<String> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        let embedded: std::collections::HashSet<String> = embedded.into_iter().collect();
        Ok(all
            .into_iter()
            .filter(|c| !embedded.contains(&c.id))
            .collect())
    }

    async fn set_component_embedding(
        &self,
        component_id: &str,
        embedding: Vec<f32>,
        model: &str,
    ) -> anyhow::Result<()> {
        self.db
            .query(
                "UPSERT type::record('component_embeddings', $rid) CONTENT { \
                 component_id: $cid, embedding: $emb, model: $model }",
            )
            .bind(("rid", component_id.to_string()))
            .bind(("cid", component_id.to_string()))
            .bind(("emb", embedding))
            .bind(("model", model.to_string()))
            .await?;
        Ok(())
    }

    async fn put_component_override(&self, ov: ComponentOverrideRecord) -> anyhow::Result<()> {
        let payload = to_db_value_without_id(&ov)?;
        self.db
            .query("UPSERT type::record('component_overrides', $rid) CONTENT $data")
            .bind(("rid", ov.id.clone()))
            .bind(("data", payload))
            .await?;
        Ok(())
    }

    async fn list_component_overrides(
        &self,
        design_system_id: &str,
    ) -> anyhow::Result<Vec<ComponentOverrideRecord>> {
        let mut resp = self
            .db
            .query("SELECT * FROM component_overrides WHERE design_system_id = $dsid")
            .bind(("dsid", design_system_id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        from_values(rows)
    }

    async fn get_component_embedding(
        &self,
        component_id: &str,
    ) -> anyhow::Result<Option<(Vec<f32>, String)>> {
        #[derive(serde::Deserialize)]
        struct EmbeddingRow {
            embedding: Vec<f32>,
            model: String,
        }
        let mut resp = self
            .db
            .query("SELECT * FROM type::record('component_embeddings', $id)")
            .bind(("id", component_id.to_string()))
            .await?;
        let rows: Vec<surrealdb::types::Value> = resp.take(0).or_else(|e| {
            if e.to_string().contains("does not exist") {
                Ok(vec![])
            } else {
                Err(anyhow::anyhow!(e))
            }
        })?;
        Ok(from_values::<EmbeddingRow>(rows)?
            .into_iter()
            .next()
            .map(|r| (r.embedding, r.model)))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Postgres-backed store (feature-gated: `sqlx`/`postgres-backend`). Schema:
// `migrations/<timestamp>_a2ui_design_systems.sql`.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "sqlx")]
#[derive(Debug, Clone)]
pub struct PostgresDesignSystemStore {
    pool: sqlx::PgPool,
}

#[cfg(feature = "sqlx")]
impl PostgresDesignSystemStore {
    #[must_use]
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(feature = "sqlx")]
#[async_trait]
impl DesignSystemStore for PostgresDesignSystemStore {
    async fn put_design_system(&self, ds: DesignSystem) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO a2ui_design_systems \
             (id, name, tokens, source_format, source_content, imported_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
             ON CONFLICT (id) DO UPDATE SET \
             name = EXCLUDED.name, tokens = EXCLUDED.tokens, \
             source_format = EXCLUDED.source_format, source_content = EXCLUDED.source_content, \
             imported_at = EXCLUDED.imported_at, updated_at = EXCLUDED.updated_at",
        )
        .bind(&ds.id)
        .bind(&ds.name)
        .bind(&ds.tokens)
        .bind(ds.source_format.as_str())
        .bind(&ds.source_content)
        .bind(ds.imported_at)
        .bind(ds.created_at)
        .bind(ds.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_design_system(&self, id: &str) -> anyhow::Result<Option<DesignSystem>> {
        let row = sqlx::query_as::<_, PgDesignSystemRow>(
            "SELECT id, name, tokens, source_format, source_content, imported_at, \
             created_at, updated_at FROM a2ui_design_systems WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(Into::into))
    }

    async fn list_design_systems(&self) -> anyhow::Result<Vec<DesignSystem>> {
        let rows = sqlx::query_as::<_, PgDesignSystemRow>(
            "SELECT id, name, tokens, source_format, source_content, imported_at, \
             created_at, updated_at FROM a2ui_design_systems ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn put_component(&self, component: Component) -> anyhow::Result<()> {
        let renderers = serde_json::to_value(&component.renderers)?;
        sqlx::query(
            "INSERT INTO a2ui_components \
             (id, slug, primitive_type, category, schema, description, usage_examples, \
              renderers, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET \
             slug = EXCLUDED.slug, primitive_type = EXCLUDED.primitive_type, \
             category = EXCLUDED.category, schema = EXCLUDED.schema, \
             description = EXCLUDED.description, usage_examples = EXCLUDED.usage_examples, \
             renderers = EXCLUDED.renderers, updated_at = EXCLUDED.updated_at",
        )
        .bind(&component.id)
        .bind(&component.slug)
        .bind(&component.primitive_type)
        .bind(&component.category)
        .bind(&component.schema)
        .bind(&component.description)
        .bind(&component.usage_examples)
        .bind(&renderers)
        .bind(component.created_at)
        .bind(component.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_component(&self, id: &str) -> anyhow::Result<Option<Component>> {
        let row = sqlx::query_as::<_, PgComponentRow>(
            "SELECT id, slug, primitive_type, category, schema, description, \
             usage_examples, renderers, created_at, updated_at \
             FROM a2ui_components WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn get_component_by_slug(&self, slug: &str) -> anyhow::Result<Option<Component>> {
        let row = sqlx::query_as::<_, PgComponentRow>(
            "SELECT id, slug, primitive_type, category, schema, description, \
             usage_examples, renderers, created_at, updated_at \
             FROM a2ui_components WHERE slug = $1",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await?;
        row.map(TryInto::try_into).transpose()
    }

    async fn list_components(&self) -> anyhow::Result<Vec<Component>> {
        let rows = sqlx::query_as::<_, PgComponentRow>(
            "SELECT id, slug, primitive_type, category, schema, description, \
             usage_examples, renderers, created_at, updated_at \
             FROM a2ui_components ORDER BY slug",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn list_components_missing_embedding(&self) -> anyhow::Result<Vec<Component>> {
        let rows = sqlx::query_as::<_, PgComponentRow>(
            "SELECT c.id, c.slug, c.primitive_type, c.category, c.schema, c.description, \
             c.usage_examples, c.renderers, c.created_at, c.updated_at \
             FROM a2ui_components c \
             WHERE NOT EXISTS ( \
                 SELECT 1 FROM a2ui_component_embeddings e WHERE e.component_id = c.id \
             ) ORDER BY c.slug",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn set_component_embedding(
        &self,
        component_id: &str,
        embedding: Vec<f32>,
        model: &str,
    ) -> anyhow::Result<()> {
        let vector = pgvector::Vector::from(embedding);
        sqlx::query(
            "INSERT INTO a2ui_component_embeddings (component_id, embedding, model, created_at) \
             VALUES ($1, $2, $3, now()) \
             ON CONFLICT (component_id) DO UPDATE SET \
             embedding = EXCLUDED.embedding, model = EXCLUDED.model, created_at = now()",
        )
        .bind(component_id)
        .bind(vector)
        .bind(model)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn put_component_override(&self, ov: ComponentOverrideRecord) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO a2ui_component_overrides \
             (id, design_system_id, component_id, prop_defaults, css_vars, \
              react_component, flutter_widget, htmx_template, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             ON CONFLICT (id) DO UPDATE SET \
             prop_defaults = EXCLUDED.prop_defaults, css_vars = EXCLUDED.css_vars, \
             react_component = EXCLUDED.react_component, \
             flutter_widget = EXCLUDED.flutter_widget, \
             htmx_template = EXCLUDED.htmx_template, updated_at = EXCLUDED.updated_at",
        )
        .bind(&ov.id)
        .bind(&ov.design_system_id)
        .bind(&ov.component_id)
        .bind(&ov.prop_defaults)
        .bind(&ov.css_vars)
        .bind(&ov.react_component)
        .bind(&ov.flutter_widget)
        .bind(&ov.htmx_template)
        .bind(ov.created_at)
        .bind(ov.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list_component_overrides(
        &self,
        design_system_id: &str,
    ) -> anyhow::Result<Vec<ComponentOverrideRecord>> {
        let rows = sqlx::query_as::<_, PgOverrideRow>(
            "SELECT id, design_system_id, component_id, prop_defaults, css_vars, \
             react_component, flutter_widget, htmx_template, created_at, updated_at \
             FROM a2ui_component_overrides WHERE design_system_id = $1",
        )
        .bind(design_system_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_component_embedding(
        &self,
        component_id: &str,
    ) -> anyhow::Result<Option<(Vec<f32>, String)>> {
        use sqlx::Row;
        let row = sqlx::query(
            "SELECT embedding, model FROM a2ui_component_embeddings WHERE component_id = $1",
        )
        .bind(component_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| {
            let vector: pgvector::Vector = r.get("embedding");
            let model: String = r.get("model");
            (vector.to_vec(), model)
        }))
    }
}

#[cfg(feature = "sqlx")]
#[derive(sqlx::FromRow)]
struct PgDesignSystemRow {
    id: String,
    name: String,
    tokens: serde_json::Value,
    source_format: String,
    source_content: Option<String>,
    imported_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "sqlx")]
impl From<PgDesignSystemRow> for DesignSystem {
    fn from(r: PgDesignSystemRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            tokens: r.tokens,
            source_format: super::types::SourceFormat::from_str_lenient(&r.source_format),
            source_content: r.source_content,
            imported_at: r.imported_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[cfg(feature = "sqlx")]
#[derive(sqlx::FromRow)]
struct PgComponentRow {
    id: String,
    slug: String,
    primitive_type: String,
    category: String,
    schema: serde_json::Value,
    description: Option<String>,
    usage_examples: Option<serde_json::Value>,
    renderers: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "sqlx")]
impl TryFrom<PgComponentRow> for Component {
    type Error = anyhow::Error;

    fn try_from(r: PgComponentRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: r.id,
            slug: r.slug,
            primitive_type: r.primitive_type,
            category: r.category,
            schema: r.schema,
            description: r.description,
            usage_examples: r.usage_examples,
            renderers: serde_json::from_value(r.renderers)?,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[cfg(feature = "sqlx")]
#[derive(sqlx::FromRow)]
struct PgOverrideRow {
    id: String,
    design_system_id: String,
    component_id: String,
    prop_defaults: serde_json::Value,
    css_vars: serde_json::Value,
    react_component: Option<String>,
    flutter_widget: Option<String>,
    htmx_template: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(feature = "sqlx")]
impl From<PgOverrideRow> for ComponentOverrideRecord {
    fn from(r: PgOverrideRow) -> Self {
        Self {
            id: r.id,
            design_system_id: r.design_system_id,
            component_id: r.component_id,
            prop_defaults: r.prop_defaults,
            css_vars: r.css_vars,
            react_component: r.react_component,
            flutter_widget: r.flutter_widget,
            htmx_template: r.htmx_template,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn ds(id: &str) -> DesignSystem {
        let now = Utc::now();
        DesignSystem {
            id: id.to_string(),
            name: format!("Design System {id}"),
            tokens: serde_json::json!({}),
            source_format: super::super::types::SourceFormat::Manual,
            source_content: None,
            imported_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn component(id: &str, slug: &str) -> Component {
        let now = Utc::now();
        Component {
            id: id.to_string(),
            slug: slug.to_string(),
            primitive_type: "Button".into(),
            category: "input".into(),
            schema: serde_json::json!({}),
            description: None,
            usage_examples: None,
            renderers: crate::uar::a2ui::design_systems::types::Renderers::default(),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn stores_and_retrieves_design_system() {
        let store = InMemoryDesignSystemStore::new();
        store.put_design_system(ds("ds1")).await.unwrap();
        let got = store.get_design_system("ds1").await.unwrap();
        assert_eq!(got.unwrap().id, "ds1");
    }

    #[tokio::test]
    async fn list_design_systems_sorted_by_name() {
        let store = InMemoryDesignSystemStore::new();
        store.put_design_system(ds("b")).await.unwrap();
        store.put_design_system(ds("a")).await.unwrap();
        let all = store.list_design_systems().await.unwrap();
        assert_eq!(all[0].id, "a");
        assert_eq!(all[1].id, "b");
    }

    #[tokio::test]
    async fn component_lookup_by_slug() {
        let store = InMemoryDesignSystemStore::new();
        store
            .put_component(component("c1", "button"))
            .await
            .unwrap();
        let got = store.get_component_by_slug("button").await.unwrap();
        assert_eq!(got.unwrap().id, "c1");
        assert!(
            store
                .get_component_by_slug("missing")
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn missing_embedding_list_shrinks_after_embed() {
        let store = InMemoryDesignSystemStore::new();
        store
            .put_component(component("c1", "button"))
            .await
            .unwrap();
        store.put_component(component("c2", "card")).await.unwrap();
        let missing = store.list_components_missing_embedding().await.unwrap();
        assert_eq!(missing.len(), 2);

        store
            .set_component_embedding("c1", vec![0.1, 0.2], "test-model")
            .await
            .unwrap();
        let missing = store.list_components_missing_embedding().await.unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].id, "c2");
    }

    #[tokio::test]
    async fn overrides_scoped_to_design_system() {
        let store = InMemoryDesignSystemStore::new();
        let now = Utc::now();
        store
            .put_component_override(ComponentOverrideRecord {
                id: "ov1".into(),
                design_system_id: "ds1".into(),
                component_id: "c1".into(),
                prop_defaults: serde_json::json!({}),
                css_vars: serde_json::json!({}),
                react_component: None,
                flutter_widget: None,
                htmx_template: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .unwrap();
        store
            .put_component_override(ComponentOverrideRecord {
                id: "ov2".into(),
                design_system_id: "ds2".into(),
                component_id: "c1".into(),
                prop_defaults: serde_json::json!({}),
                css_vars: serde_json::json!({}),
                react_component: None,
                flutter_widget: None,
                htmx_template: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .unwrap();

        let ds1_overrides = store.list_component_overrides("ds1").await.unwrap();
        assert_eq!(ds1_overrides.len(), 1);
        assert_eq!(ds1_overrides[0].id, "ov1");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SurrealDB store integration tests — embedded SurrealKV, no external server.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "surreal-backend"))]
mod surreal_tests {
    use super::*;
    use chrono::Utc;

    async fn store() -> SurrealDesignSystemStore {
        let dir = tempfile_dir();
        let db = surrealdb::engine::any::connect(format!("surrealkv://{dir}"))
            .await
            .expect("connect embedded surrealkv");
        db.use_ns("uar_test")
            .use_db("uar_test")
            .await
            .expect("use ns/db");
        SurrealDesignSystemStore::new(db)
    }

    fn tempfile_dir() -> String {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "uar-a2ui-design-systems-test-{}",
            uuid::Uuid::new_v4()
        ));
        dir.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn stores_and_retrieves_design_system() {
        let store = store().await;
        let now = Utc::now();
        let ds = DesignSystem {
            id: "ds1".into(),
            name: "Acme".into(),
            tokens: serde_json::json!({"color": {"primary": "#2563eb"}}),
            source_format: crate::uar::a2ui::design_systems::types::SourceFormat::DesignMd,
            source_content: None,
            imported_at: Some(now),
            created_at: now,
            updated_at: now,
        };
        store.put_design_system(ds).await.unwrap();
        let got = store.get_design_system("ds1").await.unwrap();
        assert_eq!(got.unwrap().name, "Acme");
    }

    #[tokio::test]
    async fn component_round_trip_and_embedding() {
        let store = store().await;
        let now = Utc::now();
        let c = Component {
            id: "c1".into(),
            slug: "button".into(),
            primitive_type: "Button".into(),
            category: "input".into(),
            schema: serde_json::json!({}),
            description: Some("A button".into()),
            usage_examples: None,
            renderers: crate::uar::a2ui::design_systems::types::Renderers::default(),
            created_at: now,
            updated_at: now,
        };
        store.put_component(c).await.unwrap();
        let missing = store.list_components_missing_embedding().await.unwrap();
        assert_eq!(missing.len(), 1);

        store
            .set_component_embedding("c1", vec![0.1, 0.2, 0.3], "test-model")
            .await
            .unwrap();
        let missing = store.list_components_missing_embedding().await.unwrap();
        assert!(
            missing.is_empty(),
            "component should no longer be missing an embedding"
        );
    }
}
