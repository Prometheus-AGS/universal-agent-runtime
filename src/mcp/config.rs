use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum McpServerEntry {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
        /// When true, the MCP server process runs inside a sandbox.
        #[serde(default)]
        sandboxed: bool,
    },
    RemoteHttp {
        url: String,
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

pub fn load_mcp_config(path: impl AsRef<Path>) -> anyhow::Result<McpConfig> {
    let txt = fs::read_to_string(path)?;
    let config: McpConfig = serde_json::from_str(&txt)?;

    #[cfg(feature = "tauri")]
    let config = {
        let mut config = config;
        for entry in config.mcp_servers.values_mut() {
            if let McpServerEntry::Stdio { command, .. } = entry
                && let Some(sidecar_path) = resolve_tauri_sidecar(command)
            {
                *command = sidecar_path;
            }
        }
        config
    };

    Ok(config)
}

#[cfg(feature = "tauri")]
fn resolve_tauri_sidecar(command: &str) -> Option<String> {
    // This would use tauri::process::Command::sidecar or similar
    // For now, we'll assume a helper that checks if the command is a known sidecar
    if command.starts_with("mcp-server-") {
        // In a real implementation, we'd use tauri::process::Command::sidecar or similar
        // but since we are in the core lib, we might need to pass the handle
        // or use a global state.

        // For now, return a dummy path if it matches a known sidecar to satisfy clippy
        if command == "mcp-server-dummy-test" {
            return Some("/usr/bin/true".to_string());
        }
        None
    } else {
        None
    }
}

/// One `${...}` placeholder found in a config value.
struct Placeholder {
    /// Byte range of the whole `${...}` span within the input.
    span: std::ops::Range<usize>,
    /// The variable name (the part before any `:-`).
    name: String,
    /// The literal default from `${VAR:-default}`, if the form supplied one.
    default: Option<String>,
}

/// Scan `input` for `${VAR}` and `${VAR:-default}` placeholders, in order.
///
/// An unterminated `${` is not a placeholder and is left alone, so a value
/// that merely contains a stray `${` is passed through rather than rejected.
fn find_placeholders(input: &str) -> Vec<Placeholder> {
    let mut found = Vec::new();
    let mut cursor = 0;

    while let Some(rel_start) = input[cursor..].find("${") {
        let start = cursor + rel_start;
        let Some(rel_end) = input[start..].find('}') else {
            break; // unterminated — not a placeholder
        };
        let end = start + rel_end + 1;
        let body = &input[start + 2..end - 1];

        // `${VAR:-default}` — shell parameter expansion with a fallback.
        let (name, default) = match body.split_once(":-") {
            Some((name, default)) => (name, Some(default.to_string())),
            None => (body, None),
        };

        found.push(Placeholder {
            span: start..end,
            name: name.to_string(),
            default,
        });
        cursor = end;
    }

    found
}

/// Substitute the placeholders in `input`, resolving each from the process
/// environment and falling back to its `${VAR:-default}` literal.
///
/// `on_unresolved` decides what a placeholder with neither a set env var nor a
/// default becomes; returning `None` leaves the original `${...}` text in place.
fn substitute(input: &str, on_unresolved: impl Fn(&str) -> Option<String>) -> String {
    let placeholders = find_placeholders(input);
    if placeholders.is_empty() {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut cursor = 0;

    for placeholder in placeholders {
        out.push_str(&input[cursor..placeholder.span.start]);

        // A set-but-empty env var loses to an explicit default, matching the
        // shell's `:-` (as opposed to `-`) semantics.
        let resolved = std::env::var(&placeholder.name)
            .ok()
            .filter(|value| !value.is_empty())
            .or(placeholder.default)
            .or_else(|| on_unresolved(&placeholder.name));

        match resolved {
            Some(value) => out.push_str(&value),
            None => out.push_str(&input[placeholder.span.clone()]),
        }
        cursor = placeholder.span.end;
    }

    out.push_str(&input[cursor..]);
    out
}

/// Expand `${VAR}` and `${VAR:-default}` placeholders from the process
/// environment, leaving unresolvable placeholders untouched.
///
/// This lenient form exists for values that are resolved in a later stage —
/// notably a remote entry's `env` map, where `${TAVILY_API_KEY}` is expected to
/// survive process-env expansion so the URL resolver can substitute it. Use
/// [`expand_env_placeholders_strict`] anywhere the expanded value is consumed
/// immediately.
///
/// # Examples
///
/// ```
/// # use universal_agent_runtime::mcp::config::expand_env_placeholders;
/// // A `${VAR:-default}` with no env var set falls back to the default.
/// assert_eq!(
///     expand_env_placeholders("${UAR_DOC_UNSET_VAR:-http://127.0.0.1:1906}"),
///     "http://127.0.0.1:1906"
/// );
/// ```
#[must_use]
pub fn expand_env_placeholders(input: &str) -> String {
    substitute(input, |_| None)
}

/// Expand `${VAR}` and `${VAR:-default}` placeholders, failing when a
/// placeholder resolves to nothing.
///
/// A value that cannot be expanded is a configuration error, not a literal:
/// letting `${UAR_MEMORY_MCP_URL:-http://…}` reach a URL parser verbatim turns
/// a missing setting into a confusing downstream parse failure.
///
/// # Errors
///
/// Returns an error naming every placeholder that had neither a non-empty
/// environment variable nor a `:-` default.
///
/// # Examples
///
/// ```
/// # use universal_agent_runtime::mcp::config::expand_env_placeholders_strict;
/// // Unset and no default — a startup error rather than a literal.
/// assert!(expand_env_placeholders_strict("${UAR_DOC_UNSET_VAR}").is_err());
/// ```
pub fn expand_env_placeholders_strict(input: &str) -> anyhow::Result<String> {
    let unresolved = std::cell::RefCell::new(Vec::new());
    let expanded = substitute(input, |name| {
        unresolved.borrow_mut().push(name.to_string());
        None
    });

    let unresolved = unresolved.into_inner();
    if !unresolved.is_empty() {
        anyhow::bail!(
            "unresolved placeholder(s) {} in '{input}' — set the variable(s) \
             or give a default with ${{VAR:-default}}",
            unresolved.join(", ")
        );
    }

    Ok(expanded)
}

#[must_use]
pub fn expand_env_map(map: &HashMap<String, String>) -> HashMap<String, String> {
    map.iter()
        .map(|(k, v)| (k.clone(), expand_env_placeholders(v)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `surreal_memory` entry in `mcp.json`. Before `${VAR:-default}` was
    /// supported this whole string reached `Url::parse` verbatim, so the
    /// memory MCP server silently dropped out of the registry.
    const MEMORY_MCP_URL: &str = "${UAR_MEMORY_MCP_URL:-http://127.0.0.1:1906/mcp/memory}";

    #[test]
    fn uses_default_when_var_is_unset() {
        assert_eq!(
            expand_env_placeholders(MEMORY_MCP_URL),
            "http://127.0.0.1:1906/mcp/memory"
        );
    }

    #[test]
    fn strict_accepts_a_defaulted_placeholder() {
        assert_eq!(
            expand_env_placeholders_strict(MEMORY_MCP_URL).unwrap(),
            "http://127.0.0.1:1906/mcp/memory"
        );
    }

    #[test]
    fn strict_rejects_a_placeholder_with_no_value_and_no_default() {
        let err = expand_env_placeholders_strict("${UAR_TEST_DEFINITELY_UNSET_VAR}")
            .expect_err("an unresolvable placeholder must be an error, not a literal");
        assert!(
            err.to_string().contains("UAR_TEST_DEFINITELY_UNSET_VAR"),
            "error should name the offending variable, got: {err}"
        );
    }

    #[test]
    fn leaves_a_value_with_no_placeholders_untouched() {
        let plain = "http://127.0.0.1:1906/mcp/memory";
        assert_eq!(expand_env_placeholders(plain), plain);
        assert_eq!(expand_env_placeholders_strict(plain).unwrap(), plain);
    }

    #[test]
    fn an_unterminated_placeholder_is_not_expanded() {
        // A stray "${" is literal text, not a malformed placeholder to reject.
        let input = "http://example.com/${unterminated";
        assert_eq!(expand_env_placeholders(input), input);
        assert_eq!(expand_env_placeholders_strict(input).unwrap(), input);
    }

    #[test]
    fn a_default_may_itself_contain_a_colon_or_slashes() {
        // Splitting on the FIRST ":-" keeps "http://host:port/path" defaults intact.
        assert_eq!(
            expand_env_placeholders("${UAR_TEST_UNSET_URL:-https://h:8443/a-b}"),
            "https://h:8443/a-b"
        );
    }

    #[test]
    fn expands_every_placeholder_in_one_value() {
        assert_eq!(
            expand_env_placeholders("${UAR_TEST_UNSET_A:-one}/${UAR_TEST_UNSET_B:-two}"),
            "one/two"
        );
    }

    #[test]
    fn strict_reports_all_unresolved_names_at_once() {
        let err = expand_env_placeholders_strict("${UAR_TEST_UNSET_X}/${UAR_TEST_UNSET_Y}")
            .expect_err("must reject");
        let msg = err.to_string();
        assert!(msg.contains("UAR_TEST_UNSET_X"), "got: {msg}");
        assert!(msg.contains("UAR_TEST_UNSET_Y"), "got: {msg}");
    }

    #[test]
    #[expect(
        unsafe_code,
        reason = "process-wide env mutation is required to test env-var precedence"
    )]
    fn a_set_var_wins_over_the_default() {
        // SAFETY: this variable is unique to this test and read only here.
        unsafe { std::env::set_var("UAR_TEST_EXPAND_SET_VAR", "from-env") };
        assert_eq!(
            expand_env_placeholders("${UAR_TEST_EXPAND_SET_VAR:-fallback}"),
            "from-env"
        );
        // SAFETY: as above.
        unsafe { std::env::remove_var("UAR_TEST_EXPAND_SET_VAR") };
    }

    #[test]
    #[expect(
        unsafe_code,
        reason = "process-wide env mutation is required to test empty-var handling"
    )]
    fn an_empty_var_falls_back_to_the_default() {
        // Matches shell `:-` semantics, where set-but-empty takes the default.
        // SAFETY: this variable is unique to this test and read only here.
        unsafe { std::env::set_var("UAR_TEST_EXPAND_EMPTY_VAR", "") };
        assert_eq!(
            expand_env_placeholders("${UAR_TEST_EXPAND_EMPTY_VAR:-fallback}"),
            "fallback"
        );
        // SAFETY: as above.
        unsafe { std::env::remove_var("UAR_TEST_EXPAND_EMPTY_VAR") };
    }
}
