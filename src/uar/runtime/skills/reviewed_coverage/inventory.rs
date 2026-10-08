use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::uar::domain::collaboration::SkillRef;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Inventory {
    schema_version: String,
    hash_algorithm: String,
    files: Vec<FileDigest>,
    skills: Vec<SkillClosure>,
    inventory_digest: String,
}

/// Parsed once, shared only by the current ReviewSession's blocking jobs.
pub(super) struct PreparedInventory {
    inventory: Inventory,
    observations: Mutex<Observations>,
}

#[derive(Default)]
struct Observations {
    files: BTreeMap<String, Result<(), String>>,
    roots: BTreeMap<String, Result<BTreeSet<String>, String>>,
}

impl PreparedInventory {
    pub(super) fn new(document: Value) -> Result<Self> {
        let observed = inventory_digest(&document)?;
        let inventory: Inventory = serde_json::from_value(document)?;
        ensure!(
            inventory.schema_version == "prometheus-reviewed-skill-closures-v1"
                && inventory.hash_algorithm == "sha256",
            "unsupported reviewed inventory"
        );
        ensure!(
            observed == inventory.inventory_digest,
            "inventory digest mismatch"
        );
        ensure!(
            inventory
                .files
                .windows(2)
                .all(|pair| pair[0].path < pair[1].path),
            "inventory files must be unique and sorted"
        );
        Ok(Self {
            inventory,
            observations: Mutex::new(Observations::default()),
        })
    }
}

#[derive(Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct FileDigest {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SkillClosure {
    identity: Identity,
    entrypoint: String,
    roots: Vec<String>,
    closure: Closure,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Identity {
    id: Option<String>,
    name: String,
    version: Option<String>,
    artifact_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Closure {
    paths: Vec<String>,
    digest: String,
}

pub(super) fn digest(bytes: &[u8]) -> String {
    let mut encoded = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

// Inventory values contain strings, integers and arrays, so sorted JSON has the
// same representation as the pack's JCS contract (no floating-point numbers).
pub(super) fn canonical_digest(value: &Value) -> Result<String> {
    fn ordered(value: &Value) -> Value {
        match value {
            Value::Object(object) => {
                let sorted: std::collections::BTreeMap<_, _> = object.iter().collect();
                Value::Object(
                    sorted
                        .into_iter()
                        .map(|(k, v)| (k.clone(), ordered(v)))
                        .collect(),
                )
            }
            Value::Array(values) => Value::Array(values.iter().map(ordered).collect()),
            _ => value.clone(),
        }
    }
    Ok(digest(&serde_json::to_vec(&ordered(value))?))
}

pub(super) fn inventory_digest(value: &Value) -> Result<String> {
    let mut value = value.clone();
    value
        .as_object_mut()
        .context("inventory must be an object")?
        .remove("inventoryDigest");
    canonical_digest(&value)
}

pub(super) fn location_path(location: &str) -> Result<PathBuf> {
    let raw = location
        .strip_prefix("file://")
        .context("reviewed skill requires a file location")?;
    let path = PathBuf::from(raw);
    ensure!(
        path.is_absolute(),
        "reviewed skill location must be absolute"
    );
    Ok(path)
}

fn relative_path(value: &str) -> Result<&Path> {
    let path = Path::new(value);
    ensure!(
        !value.is_empty()
            && !value.contains(['\\', ':'])
            && path
                .components()
                .all(|part| matches!(part, Component::Normal(_)))
            && !value
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == ".."),
        "invalid closure relative path"
    );
    Ok(path)
}

fn checked_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let mut path = root.to_owned();
    for component in relative_path(relative)?.components() {
        path.push(component);
        ensure!(
            !std::fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "reviewed closure cannot contain symlinks"
        );
    }
    Ok(path)
}

fn enumerate(root: &Path, directory: &Path, paths: &mut BTreeSet<String>) -> Result<()> {
    for item in std::fs::read_dir(directory)? {
        let item = item?;
        let kind = item.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "reviewed closure cannot contain symlinks"
        );
        if kind.is_dir() {
            enumerate(root, &item.path(), paths)?;
        } else {
            ensure!(
                kind.is_file(),
                "reviewed closure must contain regular files"
            );
            paths.insert(
                item.path()
                    .strip_prefix(root)?
                    .to_str()
                    .context("closure path must be UTF-8")?
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

impl PreparedInventory {
    pub(super) fn verify(
        &self,
        root: &Path,
        requested: &SkillRef,
        location: &str,
    ) -> Result<(String, String, String)> {
        let inventory = &self.inventory;
        let mut observations = self
            .observations
            .lock()
            .map_err(|_| anyhow::anyhow!("review session observation lock unavailable"))?;
        let root = std::fs::canonicalize(root)?;
        let installed = std::fs::canonicalize(location_path(location)?)?;
        let mut matches = inventory.skills.iter().filter(|entry| {
            relative_path(&entry.entrypoint).is_ok() && root.join(&entry.entrypoint) == installed
        });
        let entry = matches
            .next()
            .context("selected skill is absent from trusted inventory")?;
        ensure!(matches.next().is_none(), "ambiguous inventory skill entry");
        ensure!(
            !entry.identity.name.is_empty()
                && entry
                    .identity
                    .id
                    .as_ref()
                    .is_none_or(|id| id == &requested.id)
                && entry
                    .identity
                    .version
                    .as_ref()
                    .is_none_or(|version| version == &requested.version)
                && entry.identity.artifact_digest == requested.digest,
            "reviewed skill identity mismatch"
        );
        ensure!(
            !entry.roots.is_empty(),
            "reviewed skill has no closure roots"
        );
        ensure!(
            entry.closure.paths.windows(2).all(|pair| pair[0] < pair[1]),
            "closure paths must be unique and sorted"
        );
        let mut projected = Vec::new();
        for path in &entry.closure.paths {
            let index = inventory
                .files
                .binary_search_by(|file| file.path.cmp(path))
                .ok()
                .context("closure file missing from inventory")?;
            let file = &inventory.files[index];
            let observed = observations.files.entry(path.clone()).or_insert_with(|| {
                let read = || -> Result<()> {
                    let bytes = std::fs::read(checked_path(&root, path)?)?;
                    ensure!(
                        digest(&bytes) == file.sha256,
                        "reviewed closure bytes changed"
                    );
                    Ok(())
                };
                read().map_err(|error| error.to_string())
            });
            observed
                .as_ref()
                .map_err(|error| anyhow::anyhow!(error.clone()))?;
            projected.push(file);
        }
        ensure!(
            canonical_digest(&serde_json::json!({ "files": projected }))? == entry.closure.digest,
            "reviewed closure digest mismatch"
        );
        let artifact = inventory
            .files
            .iter()
            .find(|file| file.path == entry.entrypoint)
            .context("entrypoint missing from inventory")?;
        ensure!(
            entry.closure.paths.contains(&entry.entrypoint) && artifact.sha256 == requested.digest,
            "entrypoint is outside reviewed closure"
        );
        for boundary in &entry.roots {
            let actual = observations
                .roots
                .entry(boundary.clone())
                .or_insert_with(|| {
                    let read = || -> Result<BTreeSet<String>> {
                        let path = checked_path(&root, boundary)?;
                        let mut actual = BTreeSet::new();
                        let kind = std::fs::symlink_metadata(&path)?.file_type();
                        if kind.is_file() {
                            actual.insert(boundary.clone());
                        } else {
                            ensure!(
                                kind.is_dir(),
                                "reviewed root must be a regular file or directory"
                            );
                            enumerate(&root, &path, &mut actual)?;
                        }
                        Ok(actual)
                    };
                    read().map_err(|error| error.to_string())
                })
                .as_ref()
                .map_err(|error| anyhow::anyhow!(error.clone()))?;
            let prefix = format!("{boundary}/");
            let expected: BTreeSet<_> = entry
                .closure
                .paths
                .iter()
                .filter(|path| *path == boundary || path.starts_with(&prefix))
                .cloned()
                .collect();
            ensure!(*actual == expected, "reviewed closure membership changed");
        }
        Ok((
            entry.closure.digest.clone(),
            inventory.inventory_digest.clone(),
            digest(location.as_bytes()),
        ))
    }
}
