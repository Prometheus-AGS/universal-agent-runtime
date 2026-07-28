//! Native web_fetch tool — performs HTTP GET/POST requests on behalf of the agent.

use crate::uar::runtime::native_skill::NativeSkill;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::time::Duration;

#[derive(Debug)]
pub struct WebFetchTool {
    pub timeout_secs: u64,
    pub max_size_kb: u64,
    pub allowed_domains: Vec<String>,
}

impl WebFetchTool {
    fn domain_allowed(&self, url: &str) -> bool {
        if self.allowed_domains.is_empty() {
            return true;
        }
        let host = url
            .split("://")
            .nth(1)
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");
        self.allowed_domains
            .iter()
            .any(|d| d == "*" || host == d.as_str() || host.ends_with(&format!(".{d}")))
    }
}

impl WebFetchTool {
    /// Turn a binary document (PDF, docx, xlsx...) into text.
    ///
    /// Anthropic's reference `mcp-server-fetch` leaves non-HTML handling
    /// undefined. Extracting here is what makes a linked PDF usable as
    /// conversation context rather than bytes the model cannot read.
    ///
    /// Gated on `document-intelligence`, which `server-full` enables and
    /// `embedded-mobile` does NOT. Both arms are written out so a phone reports
    /// the limitation with a reason instead of returning lossy-decoded
    /// nonsense that looks like content.
    #[allow(unused_variables)]
    async fn extract_document(
        url: &str,
        content_type: &str,
        status: u16,
        bytes: &[u8],
    ) -> Value {
        #[cfg(feature = "document-intelligence")]
        {
            // Go through UAR's OWN kreuzberg wrapper rather than calling the
            // crate directly: it already owns the config mapping, the input-size
            // limit and the OCR setup. A second integration point would drift
            // from the one uploaded attachments already use.
            // `process_bytes` is re-exported from file_processing; the module
            // itself is private, and reaching through it was the mistake here.
            use crate::uar::file_processing::process_bytes;

            let config = crate::config::KreuzbergConfig::default();
            match process_bytes(bytes, content_type, &config).await {
                Ok(result) => {
                    return json!({
                        "ok": status < 400,
                        "status": status,
                        "content_type": content_type,
                        "url": url,
                        "body": result.content,
                        "extraction": "document",
                        "size_bytes": bytes.len(),
                    });
                }
                Err(error) => {
                    return json!({
                        "ok": false,
                        "status": status,
                        "content_type": content_type,
                        "url": url,
                        "error": format!("could not extract text from this document: {error}"),
                    });
                }
            }
        }
        #[cfg(not(feature = "document-intelligence"))]
        {
            json!({
                "ok": false,
                "status": status,
                "content_type": content_type,
                "url": url,
                "error": format!(
                    "`{content_type}` needs document extraction, which this build does not \
                     include. Fetch it from a server deployment, or request a text format."
                ),
                "platformLimitation": true,
            })
        }
    }

    /// Parse, resolve, and refuse the URL if it points at address space this
    /// tool must never reach.
    ///
    /// Resolution happens BEFORE the check so the decision is made on the
    /// resolved IP rather than the hostname string — a hostname-only rule loses
    /// to DNS rebinding, where an ordinary-looking name answers with 127.0.0.1.
    async fn guard_url(&self, url: &str) -> Result<(), super::fetch_guard::FetchDenial> {
        use super::fetch_guard::{FetchDenial, check_resolved_addresses, check_scheme};

        let parsed = url::Url::parse(url)
            .map_err(|_| FetchDenial::UnsupportedScheme(url.to_string()))?;
        check_scheme(&parsed)?;

        let host = parsed
            .host_str()
            .ok_or_else(|| FetchDenial::UnresolvableHost(url.to_string()))?
            .to_string();

        // A literal IP needs no DNS round-trip, and must not get one: resolving
        // it would be a no-op that only adds a failure mode.
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            return check_resolved_addresses(&host, &[ip]);
        }

        let port = parsed.port_or_known_default().unwrap_or(443);
        let lookup_host = host.clone();
        let resolved = tokio::task::spawn_blocking(move || {
            use std::net::ToSocketAddrs;
            (lookup_host.as_str(), port)
                .to_socket_addrs()
                .map(|addrs| addrs.map(|a| a.ip()).collect::<Vec<_>>())
                .unwrap_or_default()
        })
        .await
        .unwrap_or_default();

        check_resolved_addresses(&host, &resolved)
    }
}

#[async_trait]
impl NativeSkill for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }
    fn description(&self) -> &str {
        "Fetch content from a URL. Supports GET and POST. Returns the response body as text."
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["url"],
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch." },
                "method": { "type": "string", "enum": ["GET", "POST"] },
                "body": { "type": "string" },
                "headers": {
                    "type": "object",
                    "additionalProperties": { "type": "string" }
                },
                "raw": { "type": "boolean" }
            }
        })
    }
    async fn execute(&self, args: Value) -> anyhow::Result<Value> {
        let url = match args.get("url").and_then(Value::as_str) {
            Some(u) => u.to_string(),
            None => return Ok(json!({"ok": false, "error": "Missing required parameter: url"})),
        };
        if !self.domain_allowed(&url) {
            return Ok(json!({"ok": false, "error": "Domain not in allowlist."}));
        }
        // SSRF guard. The operator-facing `allowed_domains` above is a
        // convenience filter and defaults to allow-everything, so it is NOT a
        // security control; this is. The rules are deliberately not
        // configurable, because the MODEL chooses the URL and a fetched page is
        // untrusted input that can instruct it to fetch somewhere else.
        if let Err(denial) = self.guard_url(&url).await {
            return Ok(json!({"ok": false, "error": denial.to_string()}));
        }
        let method = args.get("method").and_then(Value::as_str).unwrap_or("GET");
        let raw = args.get("raw").and_then(Value::as_bool).unwrap_or(false);
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .user_agent("UAR-WebFetch/1.0")
            // Redirects are NOT followed automatically. A public URL that
            // redirects to 169.254.169.254 would otherwise sail past the
            // pre-flight check, since that check only saw the original host.
            // The hop is surfaced to the caller instead.
            .redirect(reqwest::redirect::Policy::none())
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return Ok(json!({"ok": false, "error": format!("HTTP client error: {}", e)}));
            }
        };
        let mut req = match method {
            "POST" => {
                let body = args
                    .get("body")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                client.post(&url).body(body)
            }
            _ => client.get(&url),
        };
        if let Some(headers) = args.get("headers").and_then(Value::as_object) {
            for (k, v) in headers {
                if let Some(val) = v.as_str() {
                    req = req.header(k.as_str(), val);
                }
            }
        }
        let response = match req.send().await {
            Ok(r) => r,
            Err(e) => return Ok(json!({"ok": false, "error": format!("Request failed: {}", e)})),
        };
        let status = response.status().as_u16();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return Ok(
                    json!({"ok": false, "error": format!("Failed to read response: {}", e)}),
                );
            }
        };
        let size_kb = bytes.len() as u64 / 1024;
        if size_kb > self.max_size_kb {
            return Ok(json!({
                "ok": false,
                "error": format!("Response {}KB exceeds limit {}KB", size_kb, self.max_size_kb)
            }));
        }
        // Route by content type BEFORE decoding. `from_utf8_lossy` on a PDF
        // yields mojibake that still looks like text, so the model reasons over
        // garbage instead of failing honestly -- worse than an error.
        let extraction = super::fetch_guard::extraction_for(&content_type, raw);
        if matches!(extraction, super::fetch_guard::Extraction::Document) {
            return Ok(Self::extract_document(&url, &content_type, status, &bytes).await);
        }

        let body_text = String::from_utf8_lossy(&bytes).to_string();
        let content = if !raw && content_type.contains("text/html") {
            let mut out = String::with_capacity(body_text.len());
            let mut in_tag = false;
            for ch in body_text.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => out.push(ch),
                    _ => {}
                }
            }
            out.split_whitespace().collect::<Vec<_>>().join(" ")
        } else {
            body_text
        };
        Ok(json!({
            "ok": status < 400,
            "status": status,
            "content_type": content_type,
            "url": url,
            "body": content,
            "size_bytes": bytes.len()
        }))
    }
}

#[cfg(test)]
mod guard_wiring_tests {
    use super::*;

    fn tool() -> WebFetchTool {
        // Empty allowlist == allow-everything, the shipped default. That is
        // exactly the configuration under which the SSRF guard has to hold on
        // its own, so it is the one worth testing.
        WebFetchTool {
            timeout_secs: 5,
            max_size_kb: 64,
            allowed_domains: Vec::new(),
        }
    }

    async fn refusal(url: &str) -> String {
        let out = tool()
            .execute(json!({ "url": url }))
            .await
            .expect("execute returns a value rather than erroring");
        assert_eq!(
            out.get("ok").and_then(Value::as_bool),
            Some(false),
            "{url} must be refused"
        );
        out.get("error")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    }

    #[tokio::test]
    async fn refuses_the_cloud_metadata_endpoint_through_execute() {
        // The guard existing is not enough — it has to be REACHED. This asserts
        // the wiring, which the pure-function tests cannot.
        let error = refusal("http://169.254.169.254/latest/meta-data/iam/").await;
        assert!(
            error.contains("private or link-local"),
            "expected the address reason, got: {error}"
        );
    }

    #[tokio::test]
    async fn refuses_loopback_and_private_literals_through_execute() {
        for url in [
            "http://127.0.0.1:8080/admin",
            "http://10.0.0.5/internal",
            "http://192.168.1.1/",
            "http://[::1]/",
        ] {
            let error = refusal(url).await;
            assert!(!error.is_empty(), "{url} must give a reason, not a bare failure");
        }
    }

    #[tokio::test]
    async fn refuses_non_http_schemes_through_execute() {
        // file:// would turn a network tool into a local file reader.
        let error = refusal("file:///etc/passwd").await;
        assert!(
            error.contains("only http and https"),
            "expected a scheme reason, got: {error}"
        );
    }

    /// The operator asked for enable/disable at system, agent AND conversation
    /// level. System level is `native_cfg.web_fetch_enabled`, which gates
    /// registration. The other two are the run policy's `tools` selection,
    /// resolved Global -> Agent -> Conversation and applied by
    /// `NativeSkillRegistry::filtered` on the run path (see runtime/manager.rs,
    /// which filters BOTH the MCP registry and native skills with the same
    /// tool_filter).
    ///
    /// This asserts that mechanism actually excludes web_fetch, rather than
    /// trusting that it does.
    #[tokio::test]
    async fn a_scoped_policy_can_exclude_web_fetch() {
        use crate::uar::runtime::native_skill::NativeSkillRegistry;
        use std::collections::HashSet;

        let registry = NativeSkillRegistry::new();
        registry.register(tool()).await;
        assert!(
            registry.contains("web_fetch").await,
            "precondition: the tool is registered"
        );

        // A policy that allows some other tool must drop web_fetch entirely.
        let allowed: HashSet<String> = ["some_other_tool".to_string()].into_iter().collect();
        let scoped = registry.filtered(Some(&allowed)).await;
        assert!(
            !scoped.contains("web_fetch").await,
            "an agent- or conversation-scoped policy must be able to disable internet access"
        );

        // And a policy that names it keeps it, so the control is a real switch
        // rather than something that always denies.
        let allowed: HashSet<String> = ["web_fetch".to_string()].into_iter().collect();
        let scoped = registry.filtered(Some(&allowed)).await;
        assert!(scoped.contains("web_fetch").await);
    }

    /// A build without `document-intelligence` (embedded-mobile, i.e. a phone)
    /// must SAY it cannot extract, not return lossy-decoded bytes. Emitting
    /// mojibake that looks like prose is worse than an error, because the model
    /// reasons over it instead of reporting failure.
    #[cfg(not(feature = "document-intelligence"))]
    #[tokio::test]
    async fn a_build_without_extraction_reports_the_limitation() {
        let out = WebFetchTool::extract_document(
            "https://example.com/report.pdf",
            "application/pdf",
            200,
            b"%PDF-1.7 fake",
        )
        .await;
        assert_eq!(out.get("ok").and_then(Value::as_bool), Some(false));
        assert_eq!(out.get("platformLimitation").and_then(Value::as_bool), Some(true));
        let error = out.get("error").and_then(Value::as_str).unwrap_or_default();
        assert!(
            error.contains("document extraction"),
            "the reason must name the missing capability, got: {error}"
        );
        // The lossy bytes must NOT be passed off as content.
        assert!(out.get("body").is_none());
    }

    #[tokio::test]
    async fn a_missing_url_is_still_a_clean_refusal() {
        let out = tool().execute(json!({})).await.expect("no panic");
        assert_eq!(out.get("ok").and_then(Value::as_bool), Some(false));
    }
}
