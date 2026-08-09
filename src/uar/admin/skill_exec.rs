//! Where a skill can execute, and why it cannot.
//!
//! THE SPLIT THAT MAKES THIS WORK
//!
//! Of 140 canonical skills in the Prometheus pack, only 37 ship `scripts/`.
//! The other ~103 are pure knowledge: the SKILL.md body is handed to the model
//! as prompt context and nothing is ever spawned. That is why a phone can carry
//! the whole catalog offline — the majority need no runner at all.
//!
//! THE PLATFORM CONSTRAINT IS NOT NEGOTIABLE
//!
//! iOS cannot spawn child processes or JIT (the same constraint that keeps
//! pglite off mobile — see `gen_ui_agent/Cargo.toml`). So an executable skill
//! on iOS is not "slow" or "degraded", it is impossible locally, and the only
//! honest options are to run it on a server through `RemoteRunner` or to say
//! clearly that it cannot run.
//!
//! REPORTING BEATS FAILING
//!
//! A skill that cannot run should say so BEFORE it is invoked, with a reason a
//! user can act on. Returning a `command not found` from the middle of a bash
//! script is the same defect as an empty array with no explanation: the caller
//! cannot tell "misconfigured" from "impossible here".

use serde::Serialize;

/// Where the runtime is hosted. Decides which execution paths exist at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecHost {
    /// Kubernetes/Docker or a bare server. The pod is the isolation boundary.
    Server,
    /// macOS, Windows or Linux desktop — can spawn processes.
    Desktop,
    /// Android: can spawn processes, but toolchains are usually absent.
    Android,
    /// iOS: cannot spawn processes or JIT, ever.
    Ios,
}

impl ExecHost {
    /// Detect from the compile target.
    pub fn current() -> Self {
        #[cfg(target_os = "ios")]
        {
            Self::Ios
        }
        #[cfg(target_os = "android")]
        {
            Self::Android
        }
        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            // Desktop and server share a target triple; the caller overrides
            // with `plan_with_host` when it knows it is a server deployment.
            Self::Desktop
        }
    }

    /// Whether this host can spawn a child process at all.
    pub fn can_spawn(self) -> bool {
        !matches!(self, Self::Ios)
    }
}

/// How a given skill can run on a given host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ExecPlan {
    /// No execution needed — the body is prompt context. Works everywhere,
    /// including offline on iOS.
    KnowledgeOnly,
    /// Run the scripts in-process under the policy gate.
    Native,
    /// Hand off to a server over `RemoteRunner`.
    Remote { reason: String },
    /// Cannot run here at all, with a reason safe to show a user.
    Unavailable {
        reason: String,
        platform_limitation: bool,
    },
}

/// Decide how a skill runs, given the host and whether a remote runner exists.
///
/// `has_scripts` comes from `pack_sync::DiscoveredSkill`; `remote_available`
/// is whether a `RemoteRunner` is configured (`UAR_SANDBOX_REMOTE_URL`).
pub fn plan_with_host(host: ExecHost, has_scripts: bool, remote_available: bool) -> ExecPlan {
    // ~74% of the pack lands here and never touches a runner.
    if !has_scripts {
        return ExecPlan::KnowledgeOnly;
    }
    if host.can_spawn() && matches!(host, ExecHost::Server | ExecHost::Desktop) {
        return ExecPlan::Native;
    }
    if remote_available {
        return ExecPlan::Remote {
            reason: match host {
                ExecHost::Ios => {
                    "iOS cannot spawn processes, so this skill runs on the server".to_string()
                }
                _ => "this device runs skill scripts on the server".to_string(),
            },
        };
    }
    ExecPlan::Unavailable {
        reason: match host {
            ExecHost::Ios => {
                "this skill runs scripts, which iOS cannot do. Connect to a server to use it."
                    .to_string()
            }
            ExecHost::Android => {
                "this skill runs scripts and no server is reachable. Connect to a server to use it."
                    .to_string()
            }
            _ => "no execution runner is available on this host".to_string(),
        },
        // iOS is a permanent platform property; Android merely lacks a server
        // right now, so a UI may legitimately offer "try again" there.
        platform_limitation: matches!(host, ExecHost::Ios),
    }
}

/// Convenience wrapper using the detected host.
pub fn plan(has_scripts: bool, remote_available: bool) -> ExecPlan {
    plan_with_host(ExecHost::current(), has_scripts, remote_available)
}

/// Binaries the pack's scripts actually rely on, in frequency order.
///
/// Measured by grepping the 37 script-bearing skills: `jq` appears 437 times,
/// `python3` 317, `node` 184, `cargo` 74. NONE of these are declared in any
/// skill's frontmatter — only 6 of 140 skills declare `compatibility` at all —
/// so a host that lacks them fails deep inside a bash script with
/// `command not found`. Probing up front turns that into a legible message.
pub const COMMON_TOOLCHAINS: &[&str] = &["jq", "python3", "node", "cargo"];

/// Which of `COMMON_TOOLCHAINS` are missing from `PATH`.
///
/// Returns empty on a host that cannot spawn processes: probing would be
/// meaningless there, and the caller has already been told execution is
/// impossible for a more fundamental reason.
pub fn missing_toolchains(host: ExecHost) -> Vec<&'static str> {
    if !host.can_spawn() {
        return Vec::new();
    }
    COMMON_TOOLCHAINS
        .iter()
        .copied()
        .filter(|bin| which(bin).is_none())
        .collect()
}

/// Minimal `which`: first match on `PATH`. Avoids a dependency for one lookup.
fn which(bin: &str) -> Option<std::path::PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let candidate = dir.join(bin);
        candidate.is_file().then_some(candidate)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knowledge_skills_run_everywhere_including_ios() {
        // The whole reason a phone can carry the catalog offline.
        for host in [
            ExecHost::Server,
            ExecHost::Desktop,
            ExecHost::Android,
            ExecHost::Ios,
        ] {
            assert_eq!(
                plan_with_host(host, false, false),
                ExecPlan::KnowledgeOnly,
                "a knowledge skill must need no runner on {host:?}"
            );
        }
    }

    #[test]
    fn a_server_runs_scripts_natively() {
        assert_eq!(
            plan_with_host(ExecHost::Server, true, false),
            ExecPlan::Native
        );
        assert_eq!(
            plan_with_host(ExecHost::Desktop, true, false),
            ExecPlan::Native
        );
    }

    #[test]
    fn ios_never_executes_locally_even_with_scripts() {
        // iOS cannot spawn processes. This must not degrade to Native under any
        // combination of inputs.
        assert!(!ExecHost::Ios.can_spawn());
        assert!(matches!(
            plan_with_host(ExecHost::Ios, true, true),
            ExecPlan::Remote { .. }
        ));
        assert!(matches!(
            plan_with_host(ExecHost::Ios, true, false),
            ExecPlan::Unavailable { .. }
        ));
    }

    #[test]
    fn unavailability_distinguishes_permanent_from_transient() {
        // A UI must not offer "retry" for something that can never work, and
        // must not imply permanence for a server that is merely unreachable.
        let ios = plan_with_host(ExecHost::Ios, true, false);
        let android = plan_with_host(ExecHost::Android, true, false);
        match (ios, android) {
            (
                ExecPlan::Unavailable {
                    platform_limitation: ios_perm,
                    reason: ios_reason,
                },
                ExecPlan::Unavailable {
                    platform_limitation: android_perm,
                    ..
                },
            ) => {
                assert!(ios_perm, "iOS cannot spawn processes — that is permanent");
                assert!(
                    !android_perm,
                    "Android just needs a server — that is transient"
                );
                assert!(
                    !ios_reason.is_empty(),
                    "an unavailable plan needs an actionable reason"
                );
            }
            other => panic!("expected both to be unavailable, got {other:?}"),
        }
    }

    #[test]
    fn a_host_that_cannot_spawn_is_not_probed_for_toolchains() {
        // Probing PATH on iOS would be meaningless noise.
        assert!(missing_toolchains(ExecHost::Ios).is_empty());
    }
}
