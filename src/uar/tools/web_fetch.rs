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
        if self.allowed_domains.is_empty() { return true; }
        let host = url
            .split("://").nth(1).unwrap_or(url)
            .split('/').next().unwrap_or("")
            .split(':').next().unwrap_or("");
        self.allowed_domains.iter().any(|d| {
            d == "*" || host == d.as_str() || host.ends_with(&format!(".{d}"))
        })
    }
}

#[async_trait]
impl NativeSkill for WebFetchTool {
    fn name(&self) -> &str { "web_fetch" }
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
        let method = args.get("method").and_then(Value::as_str).unwrap_or("GET");
        let raw = args.get("raw").and_then(Value::as_bool).unwrap_or(false);
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .user_agent("UAR-WebFetch/1.0")
            .build()
        {
            Ok(c) => c,
            Err(e) => return Ok(json!({"ok": false, "error": format!("HTTP client error: {}", e)})),
        };
        let mut req = match method {
            "POST" => {
                let body = args.get("body").and_then(Value::as_str).unwrap_or("").to_string();
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
            Err(e) => return Ok(json!({"ok": false, "error": format!("Failed to read response: {}", e)})),
        };
        let size_kb = bytes.len() as u64 / 1024;
        if size_kb > self.max_size_kb {
            return Ok(json!({
                "ok": false,
                "error": format!("Response {}KB exceeds limit {}KB", size_kb, self.max_size_kb)
            }));
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
