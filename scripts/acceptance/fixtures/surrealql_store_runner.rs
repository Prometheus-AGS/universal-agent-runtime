//! Host fixture tool: run SurrealQL from stdin against a stopped UAR embedded
//! store with the same pinned SurrealDB SDK that UAR links, then print each
//! statement's result as one JSON line. The host CLI cannot read stores written
//! by this SDK revision, so fixtures that must edit a stopped store use this.
//!
//! Usage: `surrealql_store_runner <endpoint> <namespace> <database> < query.surql`
//! where `<endpoint>` is, for example, `surrealkv:///path/to/memory.db`.
//!
//! Offline upgrade: `surrealql_store_runner --migrate-3.3 <snapshot> <new-destination>`.
//! The operator must verify the snapshot's provenance and quiescence first.
use anyhow::{Context, Result, ensure};
use std::io::Read;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let endpoint = args.next().context("missing <endpoint>")?;
    if endpoint == "--migrate-3.3" {
        let snapshot = args.next().context("missing <snapshot>")?;
        let destination = args.next().context("missing <new-destination>")?;
        ensure!(args.next().is_none(), "unexpected migration argument");
        #[cfg(unix)]
        return migration::run(snapshot.as_ref(), destination.as_ref()).await;
        #[cfg(not(unix))]
        {
            drop((snapshot, destination));
            anyhow::bail!("offline migration requires a Unix filesystem");
        }
    }

    if endpoint == "--restore-3.3" {
        let snapshot = args.next().context("missing <snapshot>")?;
        let destination = args.next().context("missing <new-destination>")?;
        let plan = args.next().context("missing <rollback-plan.json>")?;
        ensure!(args.next().is_none(), "unexpected restore argument");
        #[cfg(unix)]
        return migration::restore(snapshot.as_ref(), destination.as_ref(), plan.as_ref());
        #[cfg(not(unix))]
        {
            drop((snapshot, destination, plan));
            anyhow::bail!("offline restore requires a Unix filesystem");
        }
    }

    let endpoint = endpoint
        .into_string()
        .ok()
        .context("<endpoint> must be UTF-8")?;
    let namespace = args
        .next()
        .context("missing <namespace>")?
        .into_string()
        .ok()
        .context("<namespace> must be UTF-8")?;
    let database = args
        .next()
        .context("missing <database>")?
        .into_string()
        .ok()
        .context("<database> must be UTF-8")?;
    let mut query = String::new();
    std::io::stdin().read_to_string(&mut query).context("failed to read SurrealQL from stdin")?;

    let db = surrealdb::engine::any::connect(endpoint).await.context("failed to open the store")?;
    let outcome = run(&db, &namespace, &database, query).await;
    // Dropping the handle cannot await engine finalization; close it explicitly.
    let closed = db.shutdown().await.context("failed to shut the store down");
    outcome.and(closed)
}

async fn run(
    db: &surrealdb::Surreal<surrealdb::engine::any::Any>,
    namespace: &str,
    database: &str,
    query: String,
) -> Result<()> {
    db.use_ns(namespace).use_db(database).await?;
    let mut response = db.query(query).await?.check()?;
    for index in 0..response.num_statements() {
        let value: surrealdb::types::Value = response.take(index)?;
        println!("{}", value.into_json_value());
    }
    Ok(())
}

#[cfg(unix)]
mod migration {
    use anyhow::{Context, Result, ensure};
    use serde::Deserialize;
    use serde_json::json;
    use std::collections::{BTreeMap, HashSet};
    use std::fs::{self, DirBuilder, File, Metadata, OpenOptions};
    use std::io::{self, Read, Write};
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
    use std::path::{Path, PathBuf};
    use surrealdb::{Surreal, engine::local::SurrealKv, opt::Config};

    pub(super) async fn run(snapshot: &Path, destination: &Path) -> Result<()> {
        let (snapshot, destination) = distinct_paths(snapshot, destination)?;
        create_directory(&destination)?;
        convert_copy(&snapshot, &destination).await.with_context(|| {
            format!("upgrade incomplete, quarantine {}", destination.display())
        })
    }

    fn distinct_paths(snapshot: &Path, destination: &Path) -> Result<(PathBuf, PathBuf)> {
        let snapshot_name = snapshot
            .file_name()
            .context("missing snapshot directory name")?;
        let snapshot = snapshot.with_file_name(snapshot_name);
        let source_metadata = fs::symlink_metadata(&snapshot)
            .with_context(|| format!("cannot inspect snapshot {}", snapshot.display()))?;
        ensure!(
            source_metadata.is_dir(),
            "snapshot must be a directory without a symlink"
        );
        let snapshot = snapshot.canonicalize().context("cannot resolve snapshot path")?;
        ensure!(snapshot.to_str().is_some(), "snapshot path must be UTF-8");

        let name = destination
            .file_name()
            .context("missing new destination directory name")?;
        let parent = destination
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .canonicalize()
            .context("cannot resolve destination parent")?;
        let parent_lineage = parent
            .ancestors()
            .map(|path| fs::metadata(path).map(|metadata| identity(&metadata)))
            .collect::<io::Result<HashSet<_>>>()?;
        let destination = parent.join(name);
        // The SDK splits a connection path at '?' and uses a lossy path conversion.
        ensure!(
            destination.to_str().is_some_and(|path| !path.contains('?')),
            "destination path must be UTF-8 and contain no '?'"
        );

        check_snapshot_tree(&snapshot, &parent_lineage, &mut HashSet::new())?;
        Ok((snapshot, destination))
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RollbackPlan {
        kind: String,
        snapshot: PathBuf,
        recovery_point: String,
        writes_frozen: bool,
        candidate_write_receipt_sha256: String,
        accepted_writes: Vec<String>,
        dispositions: BTreeMap<String, Disposition>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Disposition {
        action: RollbackAction,
        record: String,
    }

    #[derive(Clone, Copy, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    enum RollbackAction {
        AcceptLoss,
        PreservedSeparately,
    }

    pub(super) fn restore(snapshot: &Path, destination: &Path, plan_path: &Path) -> Result<()> {
        let (snapshot, destination) = distinct_paths(snapshot, destination)?;
        let plan: RollbackPlan = serde_json::from_reader(File::open(plan_path)?)
            .context("invalid rollback plan")?;
        ensure!(plan.kind == "uar.embedded-surrealdb-rollback/v1", "unsupported rollback plan");
        ensure!(plan.snapshot == snapshot, "rollback plan must bind the canonical snapshot");
        ensure!(plan.writes_frozen, "freeze writes before restoring");
        ensure!(!plan.recovery_point.trim().is_empty(), "missing recovery point");
        ensure!(
            plan.candidate_write_receipt_sha256.len() == 64
                && plan.candidate_write_receipt_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "missing candidate write receipt digest"
        );
        let accepted: HashSet<_> = plan.accepted_writes.iter().collect();
        ensure!(
            accepted.len() == plan.accepted_writes.len()
                && accepted.iter().all(|id| !id.trim().is_empty()),
            "invalid accepted-write inventory"
        );
        ensure!(
            accepted.len() == plan.dispositions.len()
                && accepted.iter().all(|id| plan.dispositions.contains_key(*id))
                && plan.dispositions.values().all(|entry| !entry.record.trim().is_empty()),
            "each accepted post-baseline write requires a recorded disposition"
        );
        // Check the source before creating a destination. The tool never opens
        // the restored store. That belongs to the pinned 3.3 artifact.
        check_legacy_format(&snapshot)?;
        create_directory(&destination)?;
        let files = copy_files(&snapshot, &destination).with_context(|| {
            format!("restore incomplete, quarantine {}", destination.display())
        })?;
        check_legacy_format(&destination)?;
        let accepted_loss: Vec<_> = plan.dispositions.iter()
            .filter_map(|(id, entry)| matches!(entry.action, RollbackAction::AcceptLoss).then_some(id))
            .collect();
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{}", json!({
            "event": "restored-copy",
            "snapshot": snapshot,
            "destination": destination,
            "files": files,
            "recovery_point": plan.recovery_point,
            "candidate_write_receipt_sha256": plan.candidate_write_receipt_sha256,
            "excluded_from_restored_baseline": plan.accepted_writes,
            "accepted_loss": accepted_loss,
        }))?;
        stdout.flush()?;
        Ok(())
    }

    async fn convert_copy(snapshot: &Path, destination: &Path) -> Result<()> {
        report("copy-start", snapshot, destination, 0)?;
        let files = copy_files(snapshot, destination)?;
        check_legacy_format(destination)?;
        report("copied", snapshot, destination, files)?;

        let config = Config::default().datastore_option("experimental_upgrade_from_3x", true);
        let db = Surreal::new::<SurrealKv>((destination, config))
            .await
            .context("failed to upgrade the copied SurrealKV store")?;
        db.shutdown().await.context("failed to close the upgraded store")?;
        report("closed", snapshot, destination, files)
    }

    fn identity(metadata: &Metadata) -> (u64, u64) {
        (metadata.dev(), metadata.ino())
    }

    fn check_snapshot_tree(
        directory: &Path,
        parent_lineage: &HashSet<(u64, u64)>,
        visited: &mut HashSet<(u64, u64)>,
    ) -> Result<()> {
        let metadata = fs::symlink_metadata(directory)?;
        ensure!(metadata.is_dir(), "not a snapshot directory: {}", directory.display());
        let directory_id = identity(&metadata);
        // Device and inode comparisons also detect a bind-mounted alias of a subtree.
        ensure!(!parent_lineage.contains(&directory_id), "destination overlaps the snapshot");
        ensure!(visited.insert(directory_id), "snapshot contains an aliased directory");
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                check_snapshot_tree(&entry.path(), parent_lineage, visited)?;
            } else {
                ensure!(kind.is_file(), "unsupported snapshot entry {}", entry.path().display());
            }
        }
        Ok(())
    }

    fn copy_files(source: &Path, destination: &Path) -> Result<usize> {
        let mut files = 0;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let source_path = entry.path();
            let target_path = destination.join(entry.file_name());
            let kind = entry.file_type()?;
            if kind.is_dir() {
                create_directory(&target_path)?;
                files += copy_files(&source_path, &target_path)?;
            } else {
                ensure!(
                    kind.is_file(),
                    "unsupported snapshot entry {}",
                    source_path.display()
                );
                let mut input = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                    .open(&source_path)
                    .with_context(|| format!("cannot open snapshot file {}", source_path.display()))?;
                let metadata = input.metadata()?;
                ensure!(
                    metadata.is_file() && metadata.nlink() == 1,
                    "snapshot entry must be a regular file without additional hard links: {}",
                    source_path.display()
                );
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&target_path)
                    .with_context(|| format!("cannot create copy file {}", target_path.display()))?;
                io::copy(&mut input, &mut output)
                    .with_context(|| format!("cannot copy snapshot file {}", source_path.display()))?;
                output.sync_all().with_context(|| format!("cannot sync {}", target_path.display()))?;
                files += 1;
            }
        }
        Ok(files)
    }

    fn create_directory(path: &Path) -> Result<()> {
        DirBuilder::new()
            .mode(0o700)
            .create(path)
            .with_context(|| format!("cannot create new directory {}", path.display()))
    }

    fn check_legacy_format(destination: &Path) -> Result<()> {
        for entry in fs::read_dir(destination)? {
            let name = entry?.file_name();
            ensure!(
                !matches!(name.to_str(), Some("CURRENT" | "MIGRATING" | "MIGRATING_ROCKSDB"))
                    && !name.as_encoded_bytes().starts_with(b"MANIFEST-"),
                "RocksDB or interrupted conversion is ineligible: {}", destination.display()
            );
        }
        // Match the pinned SurrealKV engine's offline legacy-format check. The version
        // of the UAR artifact that wrote this snapshot is verified separately by the operator.
        let mut manifest = File::open(destination.join("manifest/00000000000000000000.manifest"))
            .context("missing legacy SurrealKV manifest in the copy")?;
        let mut format = [0; 2];
        manifest.read_exact(&mut format).context("unreadable SurrealKV manifest format")?;
        ensure!(u16::from_be_bytes(format) == 1, "copy is not a legacy SurrealKV format 1 store");
        Ok(())
    }

    fn report(stage: &str, snapshot: &Path, destination: &Path, files: usize) -> Result<()> {
        let mut output = io::stdout().lock();
        writeln!(
            output,
            "{}",
            json!({
                "stage": stage,
                "snapshot": snapshot.display().to_string(),
                "destination": destination.display().to_string(),
                "files": files,
            })
        )?;
        output.flush().context("cannot report upgrade lineage")
    }
}
