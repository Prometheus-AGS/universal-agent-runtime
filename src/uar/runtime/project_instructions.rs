//! Host-owned, trust-gated project instruction discovery. This module never
//! follows instruction files outside the selected project root.

use std::collections::BTreeMap;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Operator-controlled discovery rules. No workspace is trusted by default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default)]
pub struct ProjectInstructionsConfig {
    /// Ordered names loaded in each directory. Defaults to `AGENTS.md` only.
    pub file_names: Vec<String>,
    /// Stop the ancestor walk at the nearest directory with one of these markers.
    pub root_markers: Vec<String>,
    /// Explicit trusted roots; request/model data cannot extend this list.
    pub trusted_workspaces: Vec<PathBuf>,
}

impl Default for ProjectInstructionsConfig {
    fn default() -> Self {
        Self {
            file_names: vec!["AGENTS.md".into()],
            root_markers: vec![".git".into()],
            trusted_workspaces: Vec::new(),
        }
    }
}

impl ProjectInstructionsConfig {
    /// Reject path-bearing names before they can escape an ancestor directory.
    ///
    /// # Errors
    /// Returns `InvalidInput` for empty names, traversal, or absolute paths.
    pub fn validate(&self) -> io::Result<()> {
        if let Some(root) = self
            .trusted_workspaces
            .iter()
            .find(|root| !root.is_absolute())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "trusted workspace must be an absolute path: {}",
                    root.display()
                ),
            ));
        }
        for name in self.file_names.iter().chain(&self.root_markers) {
            let mut components = Path::new(name).components();
            if !matches!(components.next(), Some(Component::Normal(_)))
                || components.next().is_some()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("project instruction name must be one filename: {name}"),
                ));
            }
        }
        Ok(())
    }
}

/// One host-admitted file, with provenance retained separately from its body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstructionFile {
    pub path: PathBuf,
    pub content: String,
}

/// Instructions discovered at turn start plus subtrees actually accessed later.
#[derive(Debug, Clone)]
pub struct ProjectInstructions {
    config: ProjectInstructionsConfig,
    root: Option<PathBuf>,
    loaded: BTreeMap<PathBuf, Vec<InstructionFile>>,
}

impl ProjectInstructions {
    /// Discover only the root-to-cwd chain, never siblings or descendants.
    /// Without a marker, the explicit trusted workspace is the upper boundary.
    ///
    /// # Errors
    /// Returns invalid configuration, cwd resolution, or admitted file I/O errors.
    pub fn discover(config: ProjectInstructionsConfig, cwd: &Path) -> io::Result<Self> {
        config.validate()?;
        let mut instructions = Self {
            config,
            root: None,
            loaded: BTreeMap::new(),
        };
        // The empty-trust case does not even inspect the requested directory.
        if instructions.config.trusted_workspaces.is_empty() {
            return Ok(instructions);
        }
        let cwd = cwd.canonicalize()?;
        let trust_root = instructions
            .config
            .trusted_workspaces
            .iter()
            .filter_map(|root| root.canonicalize().ok())
            .filter(|root| cwd.starts_with(root))
            .max_by_key(|root| root.components().count());
        let Some(trust_root) = trust_root else {
            return Ok(instructions);
        };
        let root = cwd
            .ancestors()
            .take_while(|path| path.starts_with(&trust_root))
            .find(|path| {
                instructions
                    .config
                    .root_markers
                    .iter()
                    .any(|marker| path.join(marker).exists())
            })
            .unwrap_or(&trust_root)
            .to_path_buf();
        instructions.root = Some(root);
        instructions.load_chain(&cwd)?;
        Ok(instructions)
    }

    /// Canonical project root, or `None` for an untrusted workspace.
    pub fn root(&self) -> Option<&Path> {
        self.root.as_deref()
    }

    /// Retain subtrees previously accessed in this session only when both the
    /// project root and the operator's trust/discovery configuration still match.
    pub fn retain_accessed_subtrees(&mut self, previous: &Self) {
        if self.root.is_none() || self.root != previous.root || self.config != previous.config {
            return;
        }
        for (directory, files) in &previous.loaded {
            self.loaded
                .entry(directory.clone())
                .or_insert_with(|| files.clone());
        }
    }

    /// Load new subtree instructions after a governed file read succeeds.
    /// Calls outside the selected project cannot extend the instruction scope.
    ///
    /// # Errors
    /// Returns path resolution or admitted instruction file I/O errors.
    pub fn on_file_read(&mut self, path: &Path) -> io::Result<Vec<InstructionFile>> {
        let Some(root) = &self.root else {
            return Ok(Vec::new());
        };
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(root) {
            return Ok(Vec::new());
        }
        let Some(parent) = canonical.parent() else {
            return Ok(Vec::new());
        };
        self.load_chain(parent)
    }

    /// Deterministic root-before-descendant view of all admitted files.
    pub fn files(&self) -> Vec<InstructionFile> {
        self.loaded.values().flatten().cloned().collect()
    }

    /// Concatenate bodies with explicit file separators and source attribution.
    pub fn render(&self) -> String {
        self.files()
            .iter()
            .map(|file| {
                format!(
                    "Project instructions: {}\n{}",
                    file.path.display(),
                    file.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    fn load_chain(&mut self, directory: &Path) -> io::Result<Vec<InstructionFile>> {
        let Some(root) = self.root.clone() else {
            return Ok(Vec::new());
        };
        let mut chain = directory
            .ancestors()
            .take_while(|path| path.starts_with(&root))
            .map(Path::to_path_buf)
            .collect::<Vec<_>>();
        chain.reverse();
        let mut added = Vec::new();
        for directory in chain {
            if self.loaded.contains_key(&directory) {
                continue;
            }
            let mut files = Vec::new();
            for name in &self.config.file_names {
                let base = directory.join(name);
                let override_path = name
                    .strip_suffix(".md")
                    .map(|stem| directory.join(format!("{stem}.override.md")));
                let path = match override_path {
                    Some(path) if path.try_exists()? => path,
                    _ => base,
                };
                let canonical = match path.canonicalize() {
                    Ok(path) => path,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                    Err(error) => return Err(error),
                };
                if !canonical.starts_with(&root) {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!(
                            "project instruction file escapes project root: {}",
                            path.display()
                        ),
                    ));
                }
                files.push(InstructionFile {
                    path,
                    content: std::fs::read_to_string(canonical)?,
                });
            }
            added.extend(files.iter().cloned());
            self.loaded.insert(directory, files);
        }
        Ok(added)
    }
}
