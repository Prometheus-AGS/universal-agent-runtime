//! `mcp-server-fetch` — a minimal MCP server exposing a single `fetch` tool
//! that retrieves a URL and returns its content as text (HTML converted to
//! readable plain text unless `raw` is requested).
//!
//! Rust reimplementation of the reference Python `mcp-server-fetch`, built so
//! this project's Tauri desktop shell can bundle a sidecar binary without a
//! Python runtime dependency. Runs over stdio, matching Tauri's `externalBin`
//! sidecar invocation model.

use rmcp::{
    ErrorData as McpError, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
};

const DEFAULT_MAX_LENGTH: i64 = 5_000;
const USER_AGENT: &str = concat!("mcp-server-fetch/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct FetchParams {
    /// URL to fetch.
    url: String,
    /// Maximum number of characters to return from the (possibly converted)
    /// content. Defaults to 5000.
    #[serde(default)]
    max_length: Option<i64>,
    /// Character index to start returning content from, for paginating
    /// through content longer than `max_length`. Defaults to 0.
    #[serde(default)]
    start_index: Option<i64>,
    /// Return the raw response body instead of converting HTML to readable
    /// plain text. Defaults to false.
    #[serde(default)]
    raw: Option<bool>,
}

#[derive(Clone)]
struct FetchMcpServer {
    client: reqwest::Client,
    #[expect(
        dead_code,
        reason = "rmcp's generated tool handler retains this router for runtime dispatch"
    )]
    tool_router: ToolRouter<FetchMcpServer>,
}

impl std::fmt::Debug for FetchMcpServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchMcpServer").finish_non_exhaustive()
    }
}

impl FetchMcpServer {
    fn new() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().user_agent(USER_AGENT).build()?;
        Ok(Self {
            client,
            tool_router: Self::tool_router(),
        })
    }
}

fn html_to_text(html: &str) -> String {
    html2text::from_read(html.as_bytes(), usize::MAX).unwrap_or_else(|_| html.to_string())
}

fn slice_content(content: &str, start_index: i64, max_length: i64) -> (String, bool) {
    let chars: Vec<char> = content.chars().collect();
    let start = start_index.max(0) as usize;
    if start >= chars.len() {
        return (String::new(), false);
    }
    let take = max_length.max(0) as usize;
    let end = (start + take).min(chars.len());
    let truncated = end < chars.len();
    (chars[start..end].iter().collect(), truncated)
}

#[tool_router]
impl FetchMcpServer {
    /// Fetch a URL and return its content as text, converting HTML to
    /// readable plain text by default.
    #[tool(
        description = "Fetch a URL from the internet and return its content as text (HTML is converted to readable plain text unless raw=true)"
    )]
    async fn fetch(
        &self,
        Parameters(p): Parameters<FetchParams>,
    ) -> Result<CallToolResult, McpError> {
        let response = self.client.get(&p.url).send().await.map_err(|e| {
            McpError::internal_error(format!("failed to fetch '{}': {e}", p.url), None)
        })?;

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body = response.text().await.map_err(|e| {
            McpError::internal_error(format!("failed to read response body: {e}"), None)
        })?;

        if !status.is_success() {
            return Err(McpError::internal_error(
                format!("'{}' returned HTTP {status}", p.url),
                None,
            ));
        }

        let raw = p.raw.unwrap_or(false);
        let is_html =
            content_type.contains("text/html") || content_type.contains("application/xhtml");
        let converted = if raw || !is_html {
            body
        } else {
            html_to_text(&body)
        };

        let max_length = p.max_length.unwrap_or(DEFAULT_MAX_LENGTH);
        let start_index = p.start_index.unwrap_or(0);
        let (slice, truncated) = slice_content(&converted, start_index, max_length);

        let text = if truncated {
            format!(
                "{slice}\n\n[Content truncated at {max_length} characters starting from index {start_index}. Call fetch again with start_index={} to continue.]",
                start_index + max_length
            )
        } else {
            slice
        };

        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for FetchMcpServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::new(ServerCapabilities::builder().enable_tools().build());
        info.server_info = Implementation::new("mcp-server-fetch", env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "Fetch MCP server. Tool: fetch(url, max_length=5000, start_index=0, raw=false) \
            — retrieves a URL and returns its content as text, converting HTML to readable \
            plain text unless raw=true."
                .to_string(),
        );
        info
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let server = FetchMcpServer::new()?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
