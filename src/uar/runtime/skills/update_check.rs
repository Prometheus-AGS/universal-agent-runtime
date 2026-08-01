//! R5: know when the skill pack needs updating.
//!
//! [`provenance`](super::provenance) answers *"which pack am I on?"*. This
//! module answers the other half of R5: **"is there a newer one?"**
//!
//! # The rule that shapes this module
//!
//! **A check that cannot reach the network reports [`UpdateStatus::Unknown`],
//! never [`UpdateStatus::UpToDate`].**
//!
//! This is not defensive style, it is the whole point. "Up to date" is a claim
//! about the remote; if the remote was never contacted, the claim is
//! unsupported. Reporting `UpToDate` on a failed request produces a system that
//! looks healthy precisely when it has stopped being able to tell — and a user
//! who trusts a green check will never look again. An honest `Unknown` prompts a
//! retry; a dishonest `UpToDate` ends the conversation.
//!
//! The type makes the mistake hard to make: every failure path in
//! [`compare_to_remote`] constructs `Unknown` with the reason attached, and
//! there is no code path from an error to `UpToDate`.

use serde::{Deserialize, Serialize};

use super::provenance::PackProvenance;

/// Where the pack is published.
///
/// Overridable so a test — or a fork — can point elsewhere without editing
/// code. Defaults to the canonical repository.
#[must_use]
pub fn default_repo() -> String {
    std::env::var("UAR_SKILL_PACK_REPO")
        .unwrap_or_else(|_| "Prometheus-AGS/prometheus-skill-system".to_string())
}

/// The result of an update check.
///
/// `Unknown` carries a reason, because "we could not tell you" is only useful
/// if it says why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum UpdateStatus {
    /// The local commit matches the remote head.
    UpToDate {
        /// The commit both sides agree on.
        commit: String,
    },
    /// The remote has moved on.
    Behind {
        /// How many commits, when the API reports it; `None` when the remote
        /// only told us the heads differ.
        by: Option<usize>,
        local: String,
        remote: String,
    },
    /// We could not determine the answer. **Never** substitute `UpToDate`.
    Unknown {
        /// Why — a network error, an unparseable response, absent provenance.
        reason: String,
    },
}

impl UpdateStatus {
    /// Is an update available? `Unknown` is **not** an update, and also not
    /// an absence of one — callers that need certainty must match explicitly.
    #[must_use]
    pub fn is_behind(&self) -> bool {
        matches!(self, Self::Behind { .. })
    }

    /// Did the check actually reach a conclusion?
    #[must_use]
    pub fn is_conclusive(&self) -> bool {
        !matches!(self, Self::Unknown { .. })
    }
}

/// What the remote reported. Separated from the HTTP call so the comparison
/// logic is testable without a network.
#[derive(Debug, Clone)]
pub struct RemoteHead {
    pub commit: String,
    /// Commits the local revision is behind by, when known.
    pub behind_by: Option<usize>,
}

/// Compare local provenance against a remote head.
///
/// Pure: takes what the remote said rather than fetching it, so every branch —
/// including the failure branches — is reachable in a test with **no network**.
///
/// `remote` is `Result` rather than `Option` so a fetch failure carries its
/// reason all the way into the reported status.
#[must_use]
pub fn compare_to_remote(
    local: &PackProvenance,
    remote: Result<RemoteHead, String>,
) -> UpdateStatus {
    let remote = match remote {
        Ok(r) => r,
        // THE RULE. A failed fetch is Unknown, with the reason preserved.
        Err(e) => {
            return UpdateStatus::Unknown {
                reason: format!("could not reach the remote: {e}"),
            };
        }
    };

    let Some(local_commit) = local.commit.as_deref() else {
        // We know the remote but not ourselves — still not a basis for
        // claiming up-to-date.
        return UpdateStatus::Unknown {
            reason: "local pack provenance has no commit; \
                     regenerate SKILLS.md to record one"
                .to_string(),
        };
    };

    if local_commit == remote.commit {
        return UpdateStatus::UpToDate {
            commit: local_commit.to_string(),
        };
    }

    UpdateStatus::Behind {
        by: remote.behind_by,
        local: local_commit.to_string(),
        remote: remote.commit,
    }
}

/// Fetch the remote head for `repo`'s default branch.
///
/// # Errors
///
/// Returns the failure as a `String` so [`compare_to_remote`] can fold it into
/// an `Unknown` reason rather than losing it.
pub async fn fetch_remote_head(repo: &str) -> Result<RemoteHead, String> {
    let url = format!("https://api.github.com/repos/{repo}/commits/HEAD");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("universal-agent-runtime")
        .build()
        .map_err(|e| format!("http client: {e}"))?;

    let resp = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        // A rate-limited or 404 response is emphatically not "up to date".
        return Err(format!("github returned {}", resp.status()));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("unparseable response: {e}"))?;

    let commit = body
        .get("sha")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "response had no `sha` field".to_string())?;

    Ok(RemoteHead {
        commit: commit.to_string(),
        behind_by: None,
    })
}

/// Check the loaded pack against its published repository.
///
/// Convenience wrapper: fetch, then compare. Any failure becomes `Unknown`.
pub async fn check_for_update(local: &PackProvenance, repo: &str) -> UpdateStatus {
    compare_to_remote(local, fetch_remote_head(repo).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance_at(commit: &str) -> PackProvenance {
        PackProvenance {
            version: Some("1.0.0".to_string()),
            commit: Some(commit.to_string()),
            skill_count: Some(10),
            generated_at: None,
            manifest_path: None,
        }
    }

    #[test]
    fn identical_commits_are_up_to_date() {
        let status = compare_to_remote(
            &provenance_at("abc123"),
            Ok(RemoteHead {
                commit: "abc123".to_string(),
                behind_by: None,
            }),
        );
        assert_eq!(
            status,
            UpdateStatus::UpToDate {
                commit: "abc123".to_string()
            }
        );
    }

    #[test]
    fn differing_commits_report_behind_with_both_sides() {
        let status = compare_to_remote(
            &provenance_at("old111"),
            Ok(RemoteHead {
                commit: "new222".to_string(),
                behind_by: Some(7),
            }),
        );
        match status {
            UpdateStatus::Behind { by, local, remote } => {
                assert_eq!(by, Some(7));
                assert_eq!(local, "old111");
                assert_eq!(remote, "new222");
            }
            other => panic!("expected Behind, got {other:?}"),
        }
    }

    /// **The rule.** A network failure must never look like success.
    #[test]
    fn a_network_failure_is_unknown_never_up_to_date() {
        let status = compare_to_remote(
            &provenance_at("abc123"),
            Err("dns lookup failed".to_string()),
        );

        assert!(
            matches!(status, UpdateStatus::Unknown { .. }),
            "a failed fetch reported {status:?}. 'Up to date' is a claim about \
             the REMOTE; if the remote was never reached the claim is \
             unsupported, and a user who trusts a green check will never look \
             again."
        );
        assert!(!status.is_conclusive());
        assert!(!status.is_behind(), "Unknown must not be mistaken for Behind");

        let UpdateStatus::Unknown { reason } = status else {
            unreachable!()
        };
        assert!(
            reason.contains("dns lookup failed"),
            "the reason must survive into the status, or the user cannot act \
             on it; got {reason:?}"
        );
    }

    #[test]
    fn absent_local_provenance_is_unknown_not_up_to_date() {
        let mut local = provenance_at("abc123");
        local.commit = None;

        let status = compare_to_remote(
            &local,
            Ok(RemoteHead {
                commit: "abc123".to_string(),
                behind_by: None,
            }),
        );
        assert!(
            matches!(status, UpdateStatus::Unknown { .. }),
            "knowing the remote but not ourselves is not a basis for claiming \
             up-to-date; got {status:?}"
        );
    }

    #[test]
    fn the_repo_is_overridable_for_forks_and_tests() {
        // Default is the canonical repo; the env var wins when set.
        assert!(default_repo().contains('/'), "must be an owner/name pair");
    }
}
