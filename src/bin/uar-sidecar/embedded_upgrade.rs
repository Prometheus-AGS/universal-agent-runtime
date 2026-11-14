//! Offline host operations. Neither operation initializes the HTTP runtime.

use std::ffi::OsStr;
#[cfg(feature = "surreal-backend")]
use std::fs::File;
use std::fs::{self, Metadata, OpenOptions};
#[cfg(feature = "surreal-backend")]
use std::io::Read as _;
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result, ensure};

const LOCK_FLAG: &str = "--lock-embedded-database";
const MIGRATE_FLAG: &str = "--migrate-embedded-database";
const DATABASE_NAME: &str = "runtime.db";

pub(super) fn dispatch() -> Option<ExitCode> {
    let mut args = std::env::args_os().skip(1);
    let command = args.next()?;
    if command != LOCK_FLAG && command != MIGRATE_FLAG {
        return None;
    }
    let result = (|| {
        let path = args.next().context("Expected one absolute database path")?;
        ensure!(args.next().is_none(), "Expected one absolute database path");
        if command == LOCK_FLAG {
            hold_database_lock(Path::new(&path))
        } else {
            migrate_database(Path::new(&path))
        }
    })();
    Some(match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("UAR offline database operation failed: {error:#}");
            ExitCode::FAILURE
        }
    })
}

fn database_directory(path: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_absolute()
            && !path.components().any(|entry| entry == Component::ParentDir)
            && path.to_str().is_some_and(|text| !text.contains('?')),
        "Expected an absolute local database directory"
    );
    #[cfg(windows)]
    ensure!(
        matches!(
            path.components().next(),
            Some(Component::Prefix(prefix)) if matches!(prefix.kind(), std::path::Prefix::Disk(_))
        ),
        "Expected a local Windows drive path"
    );

    for ancestor in path.ancestors().filter(|entry| entry.file_name().is_some()) {
        let metadata =
            fs::symlink_metadata(ancestor).context("Cannot inspect database directory")?;
        ensure!(
            metadata.is_dir() && !is_link(&metadata),
            "Database directory path crosses a link or reparse point"
        );
    }
    path.canonicalize()
        .context("Cannot resolve database directory")
}

fn is_link(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt as _;
        use windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn hold_database_lock(path: &Path) -> Result<()> {
    let database = database_directory(path)?;
    let name = database.file_name();
    ensure!(
        name == Some(OsStr::new(DATABASE_NAME))
            || (name == Some(OsStr::new("original"))
                && database.parent().and_then(Path::file_name)
                    == Some(OsStr::new(".runtime-db-upgrade"))),
        "Expected runtime.db or the original database in a migration transaction"
    );
    let lock_path = database.join("LOCK");
    let metadata =
        fs::symlink_metadata(&lock_path).context("Cannot inspect existing database lock")?;
    ensure!(
        metadata.is_file() && !is_link(&metadata),
        "Database lock must be an existing regular file"
    );

    let mut options = OpenOptions::new();
    options.read(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        };
        // Retain the lock on this inode across the host's source-to-backup rename.
        options
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options
        .open(lock_path)
        .context("Cannot open existing database lock")?;
    let metadata = file
        .metadata()
        .context("Cannot inspect opened database lock")?;
    ensure!(
        metadata.is_file() && !is_link(&metadata),
        "Database lock must be an existing regular file"
    );

    // The pinned Rust implementation uses the same flock/LockFileEx range as
    // both SurrealKV v0.21 and 1.2. Opening without create/truncate preserves LOCK.
    file.try_lock()
        .context("Cannot acquire database lock; stop any other runtime using it")?;
    let mut input =
        super::inherited_input::Input::open().context("Cannot open host control pipe")?;
    write_marker("MIGRATION_LOCKED")?;
    ensure!(
        input.read_token_line(1)?.is_empty(),
        "Database lock control pipe expects EOF"
    );
    drop(file);
    Ok(())
}

fn write_marker(marker: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{marker}")?;
    stdout
        .flush()
        .context("Cannot acknowledge offline database operation")
}

#[cfg(not(feature = "surreal-backend"))]
fn migrate_database(_path: &Path) -> Result<()> {
    anyhow::bail!("This UAR build has no embedded SurrealDB engine")
}

#[cfg(feature = "surreal-backend")]
fn migrate_database(path: &Path) -> Result<()> {
    use surrealdb::Surreal;
    use surrealdb::engine::local::SurrealKv;
    use surrealdb::opt::Config;

    const CURRENT_MANIFEST_FORMAT: u16 = 3;
    const LEGACY_MANIFEST_FORMAT: u16 = 1;

    let staged = database_directory(path)?;
    let valid_stage = staged
        .parent()
        .and_then(Path::file_name)
        .and_then(OsStr::to_str)
        .and_then(|name| name.strip_prefix("stage-"))
        .is_some_and(|name| {
            uuid::Uuid::parse_str(name).is_ok_and(|id| {
                id.get_version() == Some(uuid::Version::Random) && id.to_string() == name
            })
        });
    ensure!(
        staged.file_name() == Some(OsStr::new(DATABASE_NAME))
            && valid_stage
            && staged
                .parent()
                .and_then(Path::parent)
                .and_then(Path::file_name)
                == Some(OsStr::new(".runtime-db-upgrade")),
        "Database conversion requires a private migration stage"
    );
    validate_stage(&staged)?;

    let manifest = staged.join("manifest/00000000000000000000.manifest");
    ensure!(
        manifest_version(&manifest)? == LEGACY_MANIFEST_FORMAT,
        "Migration stage is not a legacy SurrealKV database"
    );
    ensure!(
        !staged.join("MIGRATING").try_exists()?
            && !staged.join("MIGRATING_ROCKSDB").try_exists()?
            && !staged.join("_surrealkv_v0_backup").try_exists()?,
        "Migration stage contains evidence of an earlier incomplete attempt"
    );

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Cannot start offline database engine")?;
    // Use the validated DOS path on Windows. The SDK interprets the '?' in a
    // verbatim Windows path as a query delimiter.
    runtime.block_on(async {
        let config = Config::new().datastore_option("experimental_upgrade_from_3x", true);
        let database = Surreal::new::<SurrealKv>((path, config))
            .await
            .context("Cannot convert the staged database")?;
        database
            .shutdown()
            .await
            .context("Cannot close the converted database")?;
        drop(database);

        let reopened = Surreal::new::<SurrealKv>(path)
            .await
            .context("Cannot reopen the converted database with normal settings")?;
        reopened
            .shutdown()
            .await
            .context("Cannot close the verified database")
    })?;
    ensure!(
        manifest_version(&manifest)? == CURRENT_MANIFEST_FORMAT,
        "Converted database has an unexpected SurrealKV format"
    );
    write_marker("MIGRATION_COMPLETE")
}

#[cfg(feature = "surreal-backend")]
fn manifest_version(path: &Path) -> Result<u16> {
    let mut header = [0u8; 2];
    File::open(path)
        .and_then(|mut file| file.read_exact(&mut header))
        .context("Cannot read embedded database manifest")?;
    Ok(u16::from_be_bytes(header))
}

#[cfg(feature = "surreal-backend")]
fn validate_stage(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root).context("Cannot inspect migration stage")? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        ensure!(
            !is_link(&metadata),
            "Migration stage contains a link or reparse point"
        );
        if metadata.is_dir() {
            validate_stage(&entry.path())?;
        } else {
            ensure!(
                metadata.is_file() && single_link(&entry.path(), &metadata)?,
                "Migration stage must contain only directories and independent regular files"
            );
        }
    }
    Ok(())
}

#[cfg(all(unix, feature = "surreal-backend"))]
fn single_link(_path: &Path, metadata: &Metadata) -> io::Result<bool> {
    use std::os::unix::fs::MetadataExt as _;
    Ok(metadata.nlink() == 1)
}

#[cfg(all(windows, feature = "surreal-backend"))]
fn single_link(path: &Path, _metadata: &Metadata) -> io::Result<bool> {
    use std::os::windows::fs::OpenOptionsExt as _;
    use std::os::windows::io::AsRawHandle as _;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
        GetFileInformationByHandle,
    };
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let mut information = BY_HANDLE_FILE_INFORMATION::default();
    // SAFETY: the owned file handle and writable output remain valid for this synchronous call.
    let ok = unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) };
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(
        information.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT == 0
            && information.nNumberOfLinks == 1,
    )
}
