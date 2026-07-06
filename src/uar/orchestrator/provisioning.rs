//! Pluggable tool provisioning: given a declared [`ToolSpec`], resolve a
//! runnable executable path via, in order: **Adopt** (already on `PATH`),
//! **native package manager** (`apt`/`dnf`/`brew`/`winget`/`choco`, whichever
//! matches the current OS and is available), **git install** (clone the
//! tool's own repo and build it from source — the same pattern
//! `prometheus-skill-system`/`prometheus-entity-management` already use as
//! submodules), and **prebuilt binary** (download + extract a release
//! archive for the detected OS/arch).
//!
//! This module is deliberately separate from [`super::process_supervisor`],
//! which solves a different problem: detecting and reusing an already-running
//! **TCP-listening service**. The dependencies this module provisions (MCP
//! stdio server commands, skill-compilation toolchains) are executables you
//! run fresh each time, not services you connect to — there's nothing to
//! "adopt" in the TCP sense, only "does this binary exist yet."
//!
//! Strategies 2–4 default to detection-only unless
//! [`ProvisionOptions::allow_install`] is set — installing system packages,
//! cloning+building third-party source, or downloading and extracting an
//! archive are all real, host-modifying actions this module will not take
//! silently.

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use tokio::process::Command;

/// Which strategy actually resolved a [`ToolSpec`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Adopt,
    NativePackageManager,
    GitInstall,
    PrebuiltBinary,
}

/// Result of a successful [`ToolProvisioner::resolve`] call.
#[derive(Debug, Clone)]
pub struct ProvisionOutcome {
    pub strategy: Strategy,
    pub path: PathBuf,
}

/// Per-OS package name for the native-package-manager strategy. `None` for an
/// OS/manager combination means "don't attempt this manager for this tool."
#[derive(Debug, Clone, Default)]
pub struct PerOsPackageName {
    pub apt: Option<&'static str>,
    pub dnf: Option<&'static str>,
    pub brew: Option<&'static str>,
    pub winget: Option<&'static str>,
    pub choco: Option<&'static str>,
}

/// Git-install strategy: clone `url` into a cache dir and run `build_cmd`
/// (argv, first element is the program) inside it. `binary_relpath` is where
/// the resulting executable ends up relative to the clone root.
#[derive(Debug, Clone)]
pub struct GitInstallSpec {
    pub url: &'static str,
    pub build_cmd: &'static [&'static str],
    pub binary_relpath: &'static str,
}

/// Prebuilt-binary strategy: a URL template with `{os}`/`{arch}` placeholders
/// pointing at a downloadable archive, plus the path of the binary inside it
/// once extracted.
#[derive(Debug, Clone)]
pub struct PrebuiltSpec {
    pub url_template: &'static str,
    pub binary_relpath_in_archive: &'static str,
}

/// Declares how to obtain one tool.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// The executable name as it would appear on `PATH` (e.g. `"rustc"`).
    pub name: &'static str,
    pub native_pkg: PerOsPackageName,
    pub git_install: Option<GitInstallSpec>,
    pub prebuilt: Option<PrebuiltSpec>,
}

/// Controls whether strategies 2–4 may actually modify the host.
#[derive(Debug, Clone)]
pub struct ProvisionOptions {
    /// When `false` (the default), strategies 2–4 only *detect* whether they
    /// *could* provision the tool and report which one would be used, rather
    /// than actually installing/cloning/downloading anything.
    pub allow_install: bool,
    /// Directory git-install clones and prebuilt-binary downloads are cached
    /// under. Defaults to `dirs::cache_dir()/uar/provisioning`.
    pub cache_dir: PathBuf,
}

impl Default for ProvisionOptions {
    fn default() -> Self {
        Self {
            allow_install: false,
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(std::env::temp_dir)
                .join("uar")
                .join("provisioning"),
        }
    }
}

#[derive(Debug)]
pub struct ToolProvisioner;

impl ToolProvisioner {
    /// Resolve `spec` to a runnable path, trying each strategy in order.
    pub async fn resolve(spec: &ToolSpec, opts: &ProvisionOptions) -> Result<ProvisionOutcome> {
        if let Some(path) = which(spec.name) {
            return Ok(ProvisionOutcome {
                strategy: Strategy::Adopt,
                path,
            });
        }

        if let Some(pkg) = native_package_for_this_os(&spec.native_pkg)
            && let Some(manager) = detect_native_package_manager().await
        {
            if !opts.allow_install {
                bail!(
                    "{} not found on PATH; would install via {manager:?} package '{pkg}', but ProvisionOptions::allow_install is false",
                    spec.name
                );
            }
            install_via_native_package_manager(manager, pkg).await?;
            if let Some(path) = which(spec.name) {
                return Ok(ProvisionOutcome {
                    strategy: Strategy::NativePackageManager,
                    path,
                });
            }
        }

        if let Some(git) = &spec.git_install {
            if !opts.allow_install {
                bail!(
                    "{} not found on PATH; would git-install from {}, but ProvisionOptions::allow_install is false",
                    spec.name,
                    git.url
                );
            }
            let path = git_install(spec.name, git, &opts.cache_dir).await?;
            return Ok(ProvisionOutcome {
                strategy: Strategy::GitInstall,
                path,
            });
        }

        if let Some(prebuilt) = &spec.prebuilt {
            if !opts.allow_install {
                bail!(
                    "{} not found on PATH; would fetch prebuilt binary, but ProvisionOptions::allow_install is false",
                    spec.name
                );
            }
            let path = fetch_prebuilt(spec.name, prebuilt, &opts.cache_dir).await?;
            return Ok(ProvisionOutcome {
                strategy: Strategy::PrebuiltBinary,
                path,
            });
        }

        bail!(
            "unable to provision '{}': not on PATH, and no package/git/prebuilt strategy is declared for this tool",
            spec.name
        )
    }
}

/// `PATH` lookup — the "Adopt" strategy. Mirrors `build.rs`'s `which()`
/// helper (that one can't be shared directly: build scripts compile before,
/// and separately from, the main crate).
fn which(name: &str) -> Option<PathBuf> {
    let finder = if cfg!(windows) { "where" } else { "which" };
    std::process::Command::new(finder)
        .arg(name)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8(o.stdout)
                .ok()
                .and_then(|s| s.lines().next().map(str::trim).map(PathBuf::from))
        })
}

fn native_package_for_this_os(pkg: &PerOsPackageName) -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        pkg.apt.or(pkg.dnf)
    } else if cfg!(target_os = "macos") {
        pkg.brew
    } else if cfg!(target_os = "windows") {
        pkg.winget.or(pkg.choco)
    } else {
        None
    }
}

/// Which native package manager is detected as available on `PATH`, per OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativePackageManager {
    Apt,
    Dnf,
    Brew,
    Winget,
    Choco,
}

async fn detect_native_package_manager() -> Option<NativePackageManager> {
    if cfg!(target_os = "linux") {
        if which("apt-get").is_some() {
            return Some(NativePackageManager::Apt);
        }
        if which("dnf").is_some() {
            return Some(NativePackageManager::Dnf);
        }
    } else if cfg!(target_os = "macos") {
        if which("brew").is_some() {
            return Some(NativePackageManager::Brew);
        }
    } else if cfg!(target_os = "windows") {
        if which("winget").is_some() {
            return Some(NativePackageManager::Winget);
        }
        if which("choco").is_some() {
            return Some(NativePackageManager::Choco);
        }
    }
    None
}

async fn install_via_native_package_manager(
    manager: NativePackageManager,
    package: &str,
) -> Result<()> {
    let (program, args): (&str, Vec<&str>) = match manager {
        NativePackageManager::Apt => ("apt-get", vec!["install", "-y", package]),
        NativePackageManager::Dnf => ("dnf", vec!["install", "-y", package]),
        NativePackageManager::Brew => ("brew", vec!["install", package]),
        NativePackageManager::Winget => (
            "winget",
            vec!["install", "-e", "--id", package, "--accept-package-agreements"],
        ),
        NativePackageManager::Choco => ("choco", vec!["install", "-y", package]),
    };
    let status = Command::new(program)
        .args(&args)
        .status()
        .await
        .with_context(|| format!("spawning {program} to install {package}"))?;
    if !status.success() {
        bail!("{program} install {package} exited with {status}");
    }
    Ok(())
}

async fn git_install(name: &str, spec: &GitInstallSpec, cache_dir: &std::path::Path) -> Result<PathBuf> {
    let clone_dir = cache_dir.join(name);
    if !clone_dir.exists() {
        tokio::fs::create_dir_all(cache_dir)
            .await
            .with_context(|| format!("creating provisioning cache dir {}", cache_dir.display()))?;
        let status = Command::new("git")
            .args(["clone", "--depth", "1", spec.url])
            .arg(&clone_dir)
            .status()
            .await
            .with_context(|| format!("cloning {}", spec.url))?;
        if !status.success() {
            bail!("git clone {} exited with {status}", spec.url);
        }
    }

    let (program, args) = spec
        .build_cmd
        .split_first()
        .context("GitInstallSpec::build_cmd must not be empty")?;
    let status = Command::new(program)
        .args(args)
        .current_dir(&clone_dir)
        .status()
        .await
        .with_context(|| format!("running build command for {name}"))?;
    if !status.success() {
        bail!("build command for {name} exited with {status}");
    }

    let binary_path = clone_dir.join(spec.binary_relpath);
    if !binary_path.exists() {
        bail!(
            "git-install for {name} completed but expected binary not found at {}",
            binary_path.display()
        );
    }
    Ok(binary_path)
}

fn os_arch_placeholders() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    (os, arch)
}

async fn fetch_prebuilt(name: &str, spec: &PrebuiltSpec, cache_dir: &std::path::Path) -> Result<PathBuf> {
    let (os, arch) = os_arch_placeholders();
    let url = spec
        .url_template
        .replace("{os}", os)
        .replace("{arch}", arch);

    let extract_dir = cache_dir.join(name);
    tokio::fs::create_dir_all(&extract_dir)
        .await
        .with_context(|| format!("creating extract dir {}", extract_dir.display()))?;

    let archive_path = cache_dir.join(format!("{name}-download"));
    let response = reqwest::get(&url)
        .await
        .with_context(|| format!("downloading {url}"))?
        .error_for_status()
        .with_context(|| format!("downloading {url}"))?;
    let bytes = response.bytes().await.context("reading download body")?;
    tokio::fs::write(&archive_path, &bytes)
        .await
        .with_context(|| format!("writing {}", archive_path.display()))?;

    // Extract via the OS's native archive tool rather than adding a new
    // Rust dependency — `tar` (incl. bsdtar's built-in `tar.exe` on modern
    // Windows) handles both .tar.* and, on most platforms, .zip via -a.
    let status = Command::new("tar")
        .args(["-xf"])
        .arg(&archive_path)
        .arg("-C")
        .arg(&extract_dir)
        .status()
        .await
        .with_context(|| format!("extracting {}", archive_path.display()))?;
    if !status.success() {
        bail!("extracting {} exited with {status}", archive_path.display());
    }

    let binary_path = extract_dir.join(spec.binary_relpath_in_archive);
    if !binary_path.exists() {
        bail!(
            "prebuilt fetch for {name} completed but expected binary not found at {}",
            binary_path.display()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&binary_path).await?.permissions();
        perms.set_mode(perms.mode() | 0o111);
        tokio::fs::set_permissions(&binary_path, perms).await?;
    }
    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which_finds_a_binary_known_to_exist() {
        // `git` is a hard dependency of this project's own submodule
        // workflow, so it's guaranteed present in any environment that can
        // build this crate at all.
        assert!(which("git").is_some());
    }

    #[test]
    fn which_returns_none_for_a_nonexistent_binary() {
        assert!(which("uar-provisioning-test-nonexistent-binary-xyz").is_none());
    }

    #[test]
    fn native_package_for_this_os_picks_apt_over_dnf_on_linux() {
        let pkg = PerOsPackageName {
            apt: Some("foo-apt"),
            dnf: Some("foo-dnf"),
            ..Default::default()
        };
        if cfg!(target_os = "linux") {
            assert_eq!(native_package_for_this_os(&pkg), Some("foo-apt"));
        }
    }

    #[test]
    fn native_package_for_this_os_returns_none_when_undeclared() {
        let pkg = PerOsPackageName::default();
        assert_eq!(native_package_for_this_os(&pkg), None);
    }

    #[tokio::test]
    async fn resolve_adopts_an_already_installed_tool_without_needing_install_permission() {
        let spec = ToolSpec {
            name: "git",
            native_pkg: PerOsPackageName::default(),
            git_install: None,
            prebuilt: None,
        };
        let opts = ProvisionOptions {
            allow_install: false,
            ..Default::default()
        };
        let outcome = ToolProvisioner::resolve(&spec, &opts).await.unwrap();
        assert_eq!(outcome.strategy, Strategy::Adopt);
    }

    #[tokio::test]
    async fn resolve_errors_without_silently_installing_when_allow_install_is_false() {
        let spec = ToolSpec {
            name: "uar-provisioning-test-nonexistent-binary-xyz",
            native_pkg: PerOsPackageName {
                apt: Some("nonexistent-package-xyz"),
                dnf: Some("nonexistent-package-xyz"),
                brew: Some("nonexistent-package-xyz"),
                winget: Some("nonexistent-package-xyz"),
                choco: Some("nonexistent-package-xyz"),
            },
            git_install: None,
            prebuilt: None,
        };
        let opts = ProvisionOptions {
            allow_install: false,
            ..Default::default()
        };
        let err = ToolProvisioner::resolve(&spec, &opts).await.unwrap_err();
        assert!(err.to_string().contains("allow_install"));
    }

    #[tokio::test]
    async fn resolve_errors_when_no_strategy_is_declared_at_all() {
        let spec = ToolSpec {
            name: "uar-provisioning-test-nonexistent-binary-xyz",
            native_pkg: PerOsPackageName::default(),
            git_install: None,
            prebuilt: None,
        };
        let opts = ProvisionOptions::default();
        let err = ToolProvisioner::resolve(&spec, &opts).await.unwrap_err();
        assert!(err.to_string().contains("unable to provision"));
    }
}
