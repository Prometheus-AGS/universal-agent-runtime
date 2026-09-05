//! Egress restriction for local-only inference (ADR-010 §1a, clauses b and c).
//!
//! The requested mode is "local-only inference with no external connection".
//! Ordering a loopback provider first does not deliver that, and neither does a
//! test over the config file: `seed_providers_from_registry` skips any provider
//! whose settings row already exists, so the YAML is not authoritative after
//! first run, and a provider can be enabled through `PUT /api/providers/{id}`
//! without appearing in any file. An adversarial review of ADR-010 found
//! `openai` enabled at `api.openai.com` in exactly that state.
//!
//! So the guarantee is enforced where every inference path converges instead:
//! `build_driver` is the single constructor for a driver — primary, failover,
//! server, and turn bindings all reach it — and it already returns a `Result`,
//! so refusing costs no signature change.
//!
//! This is the *egress* axis. `governance::runtime_control` checks the opposite
//! direction (is our own listener bound to loopback, `BoundIngressNotLoopback`)
//! and does not constrain where inference is sent.
//!
//! Off by default. A runtime that silently refused remote providers would break
//! every existing deployment; the mode is opt-in and, once on, fails closed.

use std::net::IpAddr;

/// Environment variable that turns the restriction on.
pub const ENV_LOCAL_ONLY: &str = "UAR_LOCAL_ONLY";

/// Whether local-only egress enforcement is active.
///
/// Default `false`. Accepts `1`/`true` (case-insensitive); anything else is
/// off, so a typo disables rather than half-enables — the operator sees remote
/// traffic work and knows the flag did not take.
#[must_use]
pub fn enabled() -> bool {
    std::env::var(ENV_LOCAL_ONLY)
        .map(|v| {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true"
        })
        .unwrap_or(false)
}

/// Why a base URL was refused. Carried so the caller can name the offender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressDenial {
    /// The URL could not be parsed, so its destination is unknown.
    Unparseable(String),
    /// The URL has no host component.
    NoHost(String),
    /// The host is a name or address that is not loopback.
    NotLoopback { url: String, host: String },
}

impl std::fmt::Display for EgressDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unparseable(url) => write!(
                f,
                "local-only inference is on ({ENV_LOCAL_ONLY}) and `{url}` is not a \
                 parseable URL, so it cannot be shown to be local"
            ),
            Self::NoHost(url) => write!(
                f,
                "local-only inference is on ({ENV_LOCAL_ONLY}) and `{url}` has no host"
            ),
            Self::NotLoopback { url, host } => write!(
                f,
                "local-only inference is on ({ENV_LOCAL_ONLY}) and `{url}` targets \
                 `{host}`, which is not loopback. The user's code context would \
                 leave this machine. Disable {ENV_LOCAL_ONLY} to allow remote \
                 providers, or point this provider at 127.0.0.1."
            ),
        }
    }
}

impl std::error::Error for EgressDenial {}

/// Whether a host string denotes this machine.
///
/// Literal addresses are parsed rather than string-matched, because
/// `::ffff:127.0.0.1` is loopback wearing a different hat — the same bypass
/// `tools::fetch_guard::is_blocked_address` documents on the SSRF side.
///
/// `localhost` is accepted by name. It is the one name a resolver is required
/// to map to loopback, and refusing it would reject the most common local
/// configuration for no gain.
///
/// Any other name is refused **without resolving it**. Resolution is a network
/// call whose answer can change between the check and the connection, so a name
/// that resolves to loopback today is not a guarantee — and a guard that makes
/// a DNS request to decide whether to avoid the network has already lost.
#[must_use]
pub fn host_is_local(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if let Ok(ip) = bare.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => v4.is_loopback(),
            IpAddr::V6(v6) => v6
                .to_ipv4_mapped()
                .map_or_else(|| v6.is_loopback(), |m| m.is_loopback()),
        };
    }
    host.eq_ignore_ascii_case("localhost")
}

/// Refuse a base URL that would send inference off this machine.
///
/// A no-op when [`enabled`] is false. An empty or absent base URL is allowed
/// through: it means "the driver's own default", which is decided elsewhere and
/// is not something this function can classify — treating it as a denial would
/// break local drivers that legitimately carry no explicit URL.
///
/// # Errors
///
/// Returns [`EgressDenial`] when the mode is on and the URL is not loopback.
pub fn check_base_url(base_url: Option<&str>) -> Result<(), EgressDenial> {
    if !enabled() {
        return Ok(());
    }
    let Some(raw) = base_url.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(());
    };
    let parsed = url::Url::parse(raw).map_err(|_| EgressDenial::Unparseable(raw.to_string()))?;
    let Some(host) = parsed.host_str() else {
        return Err(EgressDenial::NoHost(raw.to_string()));
    };
    if host_is_local(host) {
        Ok(())
    } else {
        Err(EgressDenial::NotLoopback {
            url: raw.to_string(),
            host: host.to_string(),
        })
    }
}

/// One enabled provider whose base URL would leave this machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offender {
    pub id: String,
    pub base_url: String,
}

/// Audit the **resolved** provider registry for non-loopback egress.
///
/// This is clause (c). It runs after the registry is hydrated from the settings
/// database, so it sees what a run will actually resolve against — not the YAML,
/// which `seed_providers_from_registry` stops honouring once a settings row
/// exists, and not a file at all for a provider created through the API.
///
/// Returns the offenders rather than logging them, so the caller decides
/// between refusing to boot and reporting. Disabled providers are ignored: a
/// disabled `openai` row is inert, and refusing to start because one exists
/// would make the mode unusable for anyone who has ever configured a vendor.
#[must_use]
pub fn audit_providers<'a, I>(providers: I) -> Vec<Offender>
where
    I: IntoIterator<Item = (&'a str, &'a str, bool)>,
{
    if !enabled() {
        return Vec::new();
    }
    providers
        .into_iter()
        .filter(|&(_, _, is_enabled)| is_enabled)
        .filter_map(|(id, base_url, _)| {
            check_base_url(Some(base_url)).err().map(|_| Offender {
                id: id.to_string(),
                base_url: base_url.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `enabled()` reads a process-global, so these cases are driven through
    /// the pure predicates rather than by mutating the environment -- a set/
    /// unset race across parallel tests would make failures non-reproducible.
    #[test]
    fn loopback_literals_are_local() {
        for host in [
            "127.0.0.1",
            "127.1.2.3",
            "::1",
            "[::1]",
            "localhost",
            "LOCALHOST",
            // The bypass fetch_guard documents: loopback wearing a v6 hat.
            "::ffff:127.0.0.1",
        ] {
            assert!(host_is_local(host), "{host} should be local");
        }
    }

    #[test]
    fn remote_and_lan_hosts_are_not_local() {
        for host in [
            "api.openai.com",
            "api.kimi.com",
            "0.0.0.0",
            // A LAN address is someone else's machine. "Private" is not "local"
            // for this product: the code context still leaves the box.
            "192.168.1.50",
            "10.0.0.7",
            // A name that merely contains the word must not pass.
            "localhost.attacker.example",
            "notlocalhost",
        ] {
            assert!(!host_is_local(host), "{host} should NOT be local");
        }
    }

    #[test]
    fn denial_message_names_the_offending_host() {
        let d = EgressDenial::NotLoopback {
            url: "https://api.openai.com/v1".to_string(),
            host: "api.openai.com".to_string(),
        };
        let msg = d.to_string();
        assert!(msg.contains("api.openai.com"), "must name the host: {msg}");
        assert!(msg.contains(ENV_LOCAL_ONLY), "must name the flag: {msg}");
    }

    #[test]
    fn audit_is_inert_when_the_mode_is_off() {
        // Default state. A remote provider must NOT be reported, or every
        // existing deployment would fail to boot on upgrade.
        assert!(!enabled(), "test env must not set {ENV_LOCAL_ONLY}");
        let rows = vec![("openai", "https://api.openai.com/v1", true)];
        assert!(audit_providers(rows).is_empty());
    }

    #[test]
    fn check_allows_absent_or_empty_base_url() {
        // "Use the driver default" is not classifiable here, and denying it
        // would break local drivers that carry no explicit URL.
        assert!(check_base_url(None).is_ok());
        assert!(check_base_url(Some("")).is_ok());
        assert!(check_base_url(Some("   ")).is_ok());
    }
}
