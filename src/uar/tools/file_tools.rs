//! Native file-system tools: file_read and file_write.

use crate::uar::runtime::native_skill::NativeSkill;
use crate::uar::tools::descriptor::{ToolEffect, ToolSource};
use async_trait::async_trait;
use cap_std::ambient_authority;
use cap_std::fs::{Dir, OpenOptions as CapabilityOpenOptions};
use serde_json::{Value, json};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Copy)]
pub(super) enum ConfinedOpenMode {
    Read,
    Write { append: bool },
    Patch,
}

#[derive(Debug)]
struct DelegatedFileRoot {
    configured_path: PathBuf,
    directory: Arc<Dir>,
}

/// Directory handles captured from trusted host configuration before any
/// delegated request exists. Child arguments can only select a relative path.
#[derive(Clone, Debug)]
pub(crate) struct DelegatedFileRoots(Arc<[DelegatedFileRoot]>);

#[cfg(unix)]
fn directory_matches_metadata(directory: &Dir, expected: &std::fs::Metadata) -> io::Result<bool> {
    use cap_std::fs::MetadataExt as CapabilityMetadataExt;
    use std::os::unix::fs::MetadataExt as StandardMetadataExt;

    let actual = directory.dir_metadata()?;
    Ok(
        StandardMetadataExt::dev(expected) == CapabilityMetadataExt::dev(&actual)
            && StandardMetadataExt::ino(expected) == CapabilityMetadataExt::ino(&actual),
    )
}

#[cfg(unix)]
fn directories_are_same(left: &Dir, right: &Dir) -> io::Result<bool> {
    use cap_std::fs::MetadataExt as CapabilityMetadataExt;

    let left = left.dir_metadata()?;
    let right = right.dir_metadata()?;
    Ok(
        CapabilityMetadataExt::dev(&left) == CapabilityMetadataExt::dev(&right)
            && CapabilityMetadataExt::ino(&left) == CapabilityMetadataExt::ino(&right),
    )
}

#[cfg(windows)]
fn directory_matches_metadata(directory: &Dir, expected: &std::fs::Metadata) -> io::Result<bool> {
    use cap_std::fs::MetadataExt as CapabilityMetadataExt;
    use std::os::windows::fs::MetadataExt as StandardMetadataExt;

    let actual = directory.dir_metadata()?;
    Ok(StandardMetadataExt::volume_serial_number(expected)
        == CapabilityMetadataExt::volume_serial_number(&actual)
        && StandardMetadataExt::file_index(expected) == CapabilityMetadataExt::file_index(&actual))
}

#[cfg(windows)]
fn directories_are_same(left: &Dir, right: &Dir) -> io::Result<bool> {
    use cap_std::fs::MetadataExt as CapabilityMetadataExt;

    let left = left.dir_metadata()?;
    let right = right.dir_metadata()?;
    Ok(CapabilityMetadataExt::volume_serial_number(&left)
        == CapabilityMetadataExt::volume_serial_number(&right)
        && CapabilityMetadataExt::file_index(&left) == CapabilityMetadataExt::file_index(&right))
}

#[cfg(not(any(unix, windows)))]
fn directory_matches_metadata(_directory: &Dir, _expected: &std::fs::Metadata) -> io::Result<bool> {
    Ok(false)
}

#[cfg(not(any(unix, windows)))]
fn directories_are_same(_left: &Dir, _right: &Dir) -> io::Result<bool> {
    Ok(true)
}

impl DelegatedFileRoots {
    pub(crate) fn capture(allowed_paths: &[String]) -> Self {
        let current_directory = std::env::current_dir().ok();
        let roots = allowed_paths
            .iter()
            .filter(|configured| !configured.trim().is_empty() && configured.as_str() != "*")
            .filter_map(|configured| {
                let configured_path = PathBuf::from(configured);
                let configured_path = if configured_path.is_absolute() {
                    configured_path
                } else {
                    current_directory.as_ref()?.join(configured_path)
                };
                // Capture the object first. Later pathname observations only
                // confirm what this immutable handle already refers to.
                let directory =
                    Dir::open_ambient_dir(&configured_path, ambient_authority()).ok()?;
                let canonical_path = std::fs::canonicalize(configured_path).ok()?;
                if canonical_path.parent().is_none() {
                    return None;
                }
                let expected = std::fs::metadata(&canonical_path).ok()?;
                if !directory_matches_metadata(&directory, &expected).ok()? {
                    return None;
                }
                let filesystem_root_path = canonical_path.ancestors().last()?;
                let filesystem_root =
                    Dir::open_ambient_dir(filesystem_root_path, ambient_authority()).ok()?;
                if directories_are_same(&directory, &filesystem_root).ok()? {
                    return None;
                }
                Some(DelegatedFileRoot {
                    configured_path: canonical_path,
                    directory: Arc::new(directory),
                })
            })
            .collect::<Vec<_>>();
        Self(roots.into())
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

pub(super) fn check_delegated_file_policy(
    allowed_paths: &[String],
    delegated_roots: &DelegatedFileRoots,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !allowed_paths.is_empty(),
        "Delegated file tool has no configured roots"
    );
    anyhow::ensure!(
        allowed_paths.iter().all(|path| !path.trim().is_empty()),
        "Delegated file tool cannot inherit an empty filesystem root"
    );
    anyhow::ensure!(
        !allowed_paths.iter().any(|path| path == "*"),
        "Delegated file tool cannot inherit wildcard filesystem authority"
    );
    anyhow::ensure!(
        !delegated_roots.is_empty(),
        "Delegated file tool has no preopened filesystem roots"
    );
    Ok(())
}

/// Open a target relative to the longest configured directory capability.
/// `cap_std` rejects `..` and symlink traversal that would escape that root.
pub(super) async fn open_confined_file(
    path: &str,
    delegated_roots: &DelegatedFileRoots,
    mode: ConfinedOpenMode,
) -> io::Result<fs::File> {
    let target = PathBuf::from(path);
    if !target.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Delegated file paths must be absolute",
        ));
    }

    let selected = delegated_roots
        .0
        .iter()
        .filter_map(|root| {
            let relative = target
                .strip_prefix(&root.configured_path)
                .ok()?
                .to_path_buf();
            Some((
                Arc::clone(&root.directory),
                relative,
                root.configured_path.components().count(),
            ))
        })
        .max_by_key(|(_, _, depth)| *depth)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Path is not beneath a configured delegated filesystem root",
            )
        })?;

    let standard_file = tokio::task::spawn_blocking(move || -> io::Result<std::fs::File> {
        let (directory, relative, _) = selected;
        if matches!(mode, ConfinedOpenMode::Write { .. })
            && let Some(parent) = relative.parent()
            && !parent.as_os_str().is_empty()
        {
            directory.create_dir_all(parent)?;
        }
        let mut options = CapabilityOpenOptions::new();
        match mode {
            ConfinedOpenMode::Read => {
                options.read(true);
            }
            ConfinedOpenMode::Write { append } => {
                options
                    .create(true)
                    .write(true)
                    .append(append)
                    .truncate(false);
            }
            ConfinedOpenMode::Patch => {
                options.read(true).write(true);
            }
        }
        directory
            .open_with(relative, &options)
            .map(cap_std::fs::File::into_std)
    })
    .await
    .map_err(io::Error::other)??;
    Ok(fs::File::from_std(standard_file))
}

pub(super) fn file_limit_bytes(max_size_kb: u64) -> std::io::Result<u64> {
    max_size_kb.checked_mul(1024).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "File size limit overflows bytes",
        )
    })
}

// Read from the same handle whose metadata was inspected. Metadata alone is
// not a bound: a file can grow while being read. One extra byte detects overflow
// without accepting truncated content as a successful full-file read.
pub(super) async fn read_bounded_file(
    file: &mut fs::File,
    max_bytes: u64,
) -> std::io::Result<String> {
    let metadata = file.metadata().await?;
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path is not a regular file",
        ));
    }
    if metadata.len() > max_bytes {
        return Err(std::io::Error::other("File exceeds size limit"));
    }
    let read_limit = max_bytes.checked_add(1).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "File read limit overflows bytes",
        )
    })?;
    let mut bytes = Vec::new();
    file.take(read_limit).read_to_end(&mut bytes).await?;
    if bytes.len() as u64 > max_bytes {
        return Err(std::io::Error::other("File exceeds size limit"));
    }
    String::from_utf8(bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

pub(super) fn path_allowed(target: &Path, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return false;
    }
    if allowed.iter().any(|p| p == "*") {
        return true;
    }
    allowed
        .iter()
        .any(|prefix| target.starts_with(PathBuf::from(prefix)))
}

// =============================================================================
// FileReadTool
// =============================================================================

#[derive(Debug)]
pub struct FileReadTool {
    pub allowed_paths: Vec<String>,
    pub max_size_kb: u64,
    delegated_roots: DelegatedFileRoots,
}

impl FileReadTool {
    pub(crate) fn new(
        allowed_paths: Vec<String>,
        max_size_kb: u64,
        delegated_roots: DelegatedFileRoots,
    ) -> Self {
        Self {
            allowed_paths,
            max_size_kb,
            delegated_roots,
        }
    }

    async fn execute_inner(&self, args: Value, confined: bool) -> anyhow::Result<Value> {
        let path_str = match args.get("path").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: path"})),
        };
        let mut file = if confined {
            match open_confined_file(&path_str, &self.delegated_roots, ConfinedOpenMode::Read).await
            {
                Ok(file) => file,
                Err(error) => {
                    return Ok(
                        json!({"ok": false, "error": format!("Cannot open confined file: {error}")}),
                    );
                }
            }
        } else {
            let canonical = match std::fs::canonicalize(&path_str) {
                Ok(path) => path,
                Err(error) => {
                    return Ok(json!({
                        "ok": false,
                        "error": format!("Cannot resolve '{path_str}': {error}")
                    }));
                }
            };
            if !path_allowed(&canonical, &self.allowed_paths) {
                return Ok(json!({
                    "ok": false,
                    "error": format!("Path '{path_str}' is not in the allowed paths list.")
                }));
            }
            match fs::File::open(&canonical).await {
                Ok(file) => file,
                Err(error) => {
                    return Ok(json!({"ok": false, "error": format!("Cannot open file: {error}")}));
                }
            }
        };
        let max_bytes = file_limit_bytes(self.max_size_kb)?;
        match read_bounded_file(&mut file, max_bytes).await {
            Ok(content) => {
                let offset = args
                    .get("offset_lines")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize;
                let limit = args.get("limit_lines").and_then(Value::as_u64);
                let lines: Vec<&str> = content.lines().collect();
                let sliced: Vec<&str> = match limit {
                    Some(n) => lines
                        .iter()
                        .skip(offset)
                        .take(n as usize)
                        .copied()
                        .collect(),
                    None => lines.iter().skip(offset).copied().collect(),
                };
                Ok(json!({
                    "ok": true,
                    "path": path_str,
                    "content": sliced.join("\n"),
                    "total_lines": lines.len(),
                    "returned_lines": sliced.len()
                }))
            }
            Err(error) => Ok(json!({"ok": false, "error": format!("Read failed: {error}")})),
        }
    }
}

#[async_trait]
impl NativeSkill for FileReadTool {
    fn check_thread_policy(
        &self,
        _policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_delegated_file_policy(&self.allowed_paths, &self.delegated_roots)
    }

    fn name(&self) -> &str {
        "file_read"
    }
    fn description(&self) -> &str {
        "Read the contents of a file from the local filesystem."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string", "description": "Absolute path to the file." },
                "offset_lines": { "type": "integer", "minimum": 0 },
                "limit_lines": { "type": "integer", "minimum": 1 }
            }
        })
    }
    fn effect(&self) -> ToolEffect {
        ToolEffect::ReadOnly
    }
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
    async fn execute_with_context(
        &self,
        args: Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        let result = self
            .execute_inner(args, context.thread_policy.is_some())
            .await?;
        if result.get("ok").and_then(Value::as_bool) == Some(true)
            && let Some(instructions) = &context.project_instructions
            && let Some(path) = result.get("path").and_then(Value::as_str)
        {
            instructions.lock().await.on_file_read(Path::new(path))?;
        }
        Ok(result)
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        self.execute_inner(args, false).await
    }
}

// =============================================================================
// FileWriteTool
// =============================================================================

#[derive(Debug)]
pub struct FileWriteTool {
    pub allowed_paths: Vec<String>,
    pub max_size_kb: u64,
    delegated_roots: DelegatedFileRoots,
}

impl FileWriteTool {
    pub(crate) fn new(
        allowed_paths: Vec<String>,
        max_size_kb: u64,
        delegated_roots: DelegatedFileRoots,
    ) -> Self {
        Self {
            allowed_paths,
            max_size_kb,
            delegated_roots,
        }
    }

    async fn execute_inner(&self, args: Value, confined: bool) -> anyhow::Result<Value> {
        let path_str = match args.get("path").and_then(Value::as_str) {
            Some(p) => p.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: path"})),
        };
        let content = match args.get("content").and_then(Value::as_str) {
            Some(c) => c.to_string(),
            None => {
                return Ok(json!({"ok": false, "error": "Missing required parameter: content"}));
            }
        };
        let append = args.get("append").and_then(Value::as_bool).unwrap_or(false);
        let max_bytes = file_limit_bytes(self.max_size_kb)?;
        if content.len() as u64 > max_bytes {
            return Ok(
                json!({"ok": false, "error": format!("Content exceeds limit {}KB", self.max_size_kb)}),
            );
        }
        let target = PathBuf::from(&path_str);
        let mut file = if confined {
            match open_confined_file(
                &path_str,
                &self.delegated_roots,
                ConfinedOpenMode::Write { append },
            )
            .await
            {
                Ok(file) => file,
                Err(error) => {
                    return Ok(
                        json!({"ok": false, "error": format!("Cannot open confined file: {error}")}),
                    );
                }
            }
        } else {
            let check_path = if target.exists() {
                match std::fs::canonicalize(&target) {
                    Ok(path) => path,
                    Err(error) => {
                        return Ok(
                            json!({"ok": false, "error": format!("Cannot resolve: {error}")}),
                        );
                    }
                }
            } else {
                let parent = target.parent().unwrap_or(Path::new("."));
                match std::fs::canonicalize(parent) {
                    Ok(path) => path.join(target.file_name().unwrap_or_default()),
                    Err(error) => {
                        return Ok(json!({
                            "ok": false,
                            "error": format!("Cannot resolve parent: {error}")
                        }));
                    }
                }
            };
            if !path_allowed(&check_path, &self.allowed_paths) {
                return Ok(json!({
                    "ok": false,
                    "error": format!("Path '{path_str}' is not in the allowed paths list.")
                }));
            }
            if let Some(parent) = target.parent()
                && let Err(error) = fs::create_dir_all(parent).await
            {
                return Ok(json!({
                    "ok": false,
                    "error": format!("Cannot create directories: {error}")
                }));
            }
            match fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(append)
                .truncate(false)
                .open(&target)
                .await
            {
                Ok(file) => file,
                Err(error) => {
                    return Ok(json!({"ok": false, "error": format!("Cannot open file: {error}")}));
                }
            }
        };
        let result: std::io::Result<()> = async {
            let metadata = file.metadata().await?;
            if !metadata.is_file() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Path is not a regular file",
                ));
            }
            if append {
                if metadata
                    .len()
                    .checked_add(content.len() as u64)
                    .is_none_or(|size| size > max_bytes)
                {
                    return Err(std::io::Error::other(
                        "Appended file would exceed size limit",
                    ));
                }
            } else {
                file.set_len(0).await?;
            }
            file.write_all(content.as_bytes()).await?;
            // Wait for Tokio's queued write before reporting success. This is
            // not fsync, rollback, or a lock against external writers.
            file.flush().await
        }
        .await;
        match result {
            Ok(()) => Ok(json!({
                "ok": true,
                "path": path_str,
                "bytes_written": content.len(),
                "mode": if append { "append" } else { "overwrite" }
            })),
            Err(error) => Ok(json!({"ok": false, "error": format!("Write failed: {error}")})),
        }
    }
}

#[async_trait]
impl NativeSkill for FileWriteTool {
    fn check_thread_policy(
        &self,
        _policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        check_delegated_file_policy(&self.allowed_paths, &self.delegated_roots)
    }

    fn name(&self) -> &str {
        "file_write"
    }
    fn description(&self) -> &str {
        "Write or overwrite a file on the local filesystem."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" },
                "append": { "type": "boolean" }
            }
        })
    }
    fn effect(&self) -> ToolEffect {
        ToolEffect::ExternalMutation
    }
    fn source(&self) -> ToolSource {
        ToolSource::BuiltIn
    }
    async fn execute_with_context(
        &self,
        args: Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        self.execute_inner(args, context.thread_policy.is_some())
            .await
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        self.execute_inner(args, false).await
    }
}
