use std::path::PathBuf;

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;

use crate::uar::domain::reviewed_skill_coverage::ReviewedSkillCoverage;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Sources {
    schema_version: u32,
    sources: Vec<TrustedSource>,
}

/// Only the launching host supplies this configuration; binding JSON cannot.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct TrustedSource {
    pub source: String,
    pub root: PathBuf,
    inventory_path: Option<PathBuf>,
    inventory_digest: Option<String>,
    verifier: Option<FullVerifier>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FullVerifier {
    executable: PathBuf,
    script: PathBuf,
    plugin_root: PathBuf,
    home: PathBuf,
    trust_store: PathBuf,
    trust_root_digest: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FullVerification {
    schema_version: String,
    source_class: String,
    generation: String,
    generation_digest: String,
    manifest_digest: String,
    signer_key_id: String,
    trust_root_digest: String,
    target_receipts: Vec<Value>,
    inventory: Value,
}

fn configured() -> Result<Vec<TrustedSource>> {
    let config = match std::env::var("UAR_REVIEWED_SKILL_SOURCES") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let config: Sources = serde_json::from_str(&config)?;
    ensure!(
        config.schema_version == 1,
        "unsupported trusted skill sources"
    );
    Ok(config.sources)
}

pub(super) async fn verified_full_roots() -> Result<Vec<PathBuf>> {
    let mut roots = Vec::new();
    for mut source in configured()?
        .into_iter()
        .filter(|source| source.source == "signed-full-generation")
    {
        source.root = tokio::fs::canonicalize(&source.root).await?;
        source.inventory().await?;
        roots.push(source.root.join("skills"));
    }
    Ok(roots)
}

pub(super) fn select(location: &str) -> Result<Option<TrustedSource>> {
    let configured = configured()?;
    if configured.is_empty() {
        return Ok(None);
    }
    let location = std::fs::canonicalize(super::inventory::location_path(location)?)?;
    let mut selected = None;
    for mut source in configured {
        let root = std::fs::canonicalize(&source.root)?;
        if location.starts_with(&root) {
            ensure!(selected.is_none(), "ambiguous trusted skill source");
            source.root = root;
            selected = Some(source);
        }
    }
    Ok(selected)
}

impl TrustedSource {
    pub async fn inventory(&self) -> Result<(Value, ReviewedSkillCoverage)> {
        if self.source == "signed-full-generation" {
            return self.full_inventory().await;
        }
        ensure!(
            self.source == "boss-packaged-mini",
            "unsupported trusted skill source"
        );
        let path = self
            .inventory_path
            .as_ref()
            .context("missing packaged inventory path")?;
        let expected = self
            .inventory_digest
            .as_ref()
            .context("missing packaged inventory pin")?;
        let bytes = tokio::fs::read(path)
            .await
            .context("trusted skill inventory unavailable")?;
        let value: Value = serde_json::from_slice(&bytes)?;
        ensure!(
            super::inventory::inventory_digest(&value)? == *expected,
            "trusted skill inventory baseline mismatch"
        );
        Ok((
            value,
            ReviewedSkillCoverage {
                source: self.source.clone(),
                closure_digest: String::new(),
                inventory_digest: expected.clone(),
                location_digest: String::new(),
                generation_digest: None,
                signer_key_id: None,
                trust_root_digest: None,
            },
        ))
    }

    async fn full_inventory(&self) -> Result<(Value, ReviewedSkillCoverage)> {
        let verifier = self
            .verifier
            .as_ref()
            .context("full generation verifier unavailable")?;
        ensure!(
            verifier.executable.is_absolute() && verifier.script.is_absolute(),
            "full verifier requires host-provisioned absolute paths"
        );
        let trust = tokio::fs::read(&verifier.trust_store).await?;
        ensure!(
            super::inventory::digest(&trust) == verifier.trust_root_digest,
            "full generation trust store changed"
        );
        let output = tokio::process::Command::new(&verifier.executable)
            .arg(&verifier.script)
            .args(["--plugin-root"])
            .arg(&verifier.plugin_root)
            .args(["--home"])
            .arg(&verifier.home)
            .args(["--trust-store"])
            .arg(&verifier.trust_store)
            .env("ELECTRON_RUN_AS_NODE", "1")
            .kill_on_drop(true)
            .output()
            .await
            .context("full generation verifier unavailable")?;
        ensure!(
            output.status.success(),
            "signed full generation verification failed"
        );
        let verified: FullVerification = serde_json::from_slice(&output.stdout)?;
        ensure!(
            verified.schema_version == "prometheus-reviewed-skill-coverage-verification-v1"
                && verified.source_class == self.source
                && verified.trust_root_digest == verifier.trust_root_digest
                && !verified.signer_key_id.is_empty()
                && verified.manifest_digest.starts_with("sha256:")
                && !verified.target_receipts.is_empty(),
            "invalid full generation verification"
        );
        let root = tokio::fs::canonicalize(&self.root).await?;
        ensure!(
            root.file_name().and_then(|value| value.to_str()) == Some(verified.generation.as_str())
                && verified.generation_digest == format!("sha256:{}", verified.generation),
            "verified generation does not match installed root"
        );
        Ok((
            verified.inventory,
            ReviewedSkillCoverage {
                source: self.source.clone(),
                closure_digest: String::new(),
                inventory_digest: String::new(),
                location_digest: String::new(),
                generation_digest: Some(verified.generation_digest),
                signer_key_id: Some(verified.signer_key_id),
                trust_root_digest: Some(verified.trust_root_digest),
            },
        ))
    }
}
