//! Owned loopback MCP counterpart using the pinned official Rust SDK.
//! A tool call appends only synthetic input to its owned journal; denied UAR
//! calls must never appear there. No network/filesystem capability tool exists.
use std::{io::Write, path::{Path, PathBuf}, sync::Arc};

use rmcp::{
    ErrorData as McpError, RoleServer,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct EchoInput {
    /// Synthetic qualification marker only.
    marker: String,
}

#[derive(Clone)]
struct Peer {
    journal: PathBuf,
    protocol_journal: PathBuf,
    #[expect(
        dead_code,
        reason = "rmcp generated handler owns the tool dispatch router"
    )]
    tool_router: ToolRouter<Self>,
}

fn protocol_record(path: &Path, value: serde_json::Value) -> Result<(), McpError> {
    let mut journal = std::fs::OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|_| McpError::internal_error("owned protocol journal unavailable", None))?;
    let bytes = format!("{value}\n");
    journal.write_all(bytes.as_bytes())
        .and_then(|()| journal.sync_all())
        .map_err(|_| McpError::internal_error("owned protocol journal write failed", None))
}

struct PendingCall {
    path: PathBuf,
    request_id: serde_json::Value,
    marker: String,
    cancellation: tokio_util::sync::CancellationToken,
    terminal: bool,
}

impl PendingCall {
    fn finish(&mut self, phase: &str) -> Result<(), McpError> {
        protocol_record(&self.path, serde_json::json!({
            "request_id": self.request_id, "marker": self.marker,
            "tool": "qualification_wait", "phase": phase,
            "sdk_cancelled": self.cancellation.is_cancelled(), "effect_performed": false,
        }))?;
        self.terminal = true;
        Ok(())
    }
}

impl Drop for PendingCall {
    fn drop(&mut self) {
        if !self.terminal {
            // SDK cancellation may drop the handler before its select branch
            // runs. Preserve that distinction and the actual SDK token state.
            if self.finish("handler_dropped").is_err() {
                eprintln!("qualification protocol journal could not retain handler drop");
            }
        }
    }
}

#[tool_router]
impl Peer {
    #[tool(
        description = "Return the synthetic marker and record that this allowed call reached the peer"
    )]
    async fn qualification_echo(
        &self,
        Parameters(input): Parameters<EchoInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if !input.marker.starts_with("PR476-") || input.marker.len() > 128 {
            return Err(McpError::invalid_params(
                "expected a bounded synthetic qualification marker",
                None,
            ));
        }
        let mut journal = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.journal)
            .map_err(|_| McpError::internal_error("owned journal unavailable", None))?;
        writeln!(journal, "{}", input.marker)
            .and_then(|()| journal.sync_all())
            .map_err(|_| McpError::internal_error("owned journal write failed", None))?;
        protocol_record(&self.protocol_journal, serde_json::json!({
            "request_id": context.id, "marker": input.marker,
            "tool": "qualification_echo", "phase": "completed", "effect_performed": true,
        }))?;
        Ok(CallToolResult::success(vec![Content::text(input.marker)]))
    }

    #[tool(description = "Wait without performing an effect so cancellation of the actual MCP request can be observed")]
    async fn qualification_wait(
        &self,
        Parameters(input): Parameters<EchoInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if !input.marker.starts_with("PR476-") || input.marker.len() > 128 {
            return Err(McpError::invalid_params("expected a bounded synthetic qualification marker", None));
        }
        let request_id = serde_json::to_value(&context.id)
            .map_err(|_| McpError::internal_error("request identity unavailable", None))?;
        protocol_record(&self.protocol_journal, serde_json::json!({
            "request_id": request_id, "marker": input.marker,
            "tool": "qualification_wait", "phase": "started", "effect_performed": false,
        }))?;
        let mut pending = PendingCall {
            path: self.protocol_journal.clone(), request_id, marker: input.marker,
            cancellation: context.ct.clone(), terminal: false,
        };
        tokio::select! {
            () = context.ct.cancelled() => {
                pending.finish("cancelled")?;
                Err(McpError::internal_error("qualification call cancelled; no effect performed", None))
            }
            () = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                pending.finish("expired")?;
                Err(McpError::internal_error("qualification cancellation was not observed before fixture expiry", None))
            }
        }
    }

    #[tool(description = "Return an intentional MCP tool error without performing an effect")]
    async fn qualification_fail(
        &self,
        Parameters(input): Parameters<EchoInput>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        if !input.marker.starts_with("PR476-") || input.marker.len() > 128 {
            return Err(McpError::invalid_params("expected a bounded synthetic qualification marker", None));
        }
        protocol_record(&self.protocol_journal, serde_json::json!({
            "request_id": context.id, "marker": input.marker,
            "tool": "qualification_fail", "phase": "intentional_error", "effect_performed": false,
        }))?;
        Ok(CallToolResult::error(vec![Content::text(
            "qualification intentional tool error; no effect performed",
        )]))
    }
}

#[tool_handler]
impl rmcp::ServerHandler for Peer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_server_info(Implementation::new(
                "uar-qualification-official-sdk-peer",
                "0.1.0",
            ))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let journal = PathBuf::from(std::env::var("UAR_QUALIFICATION_MCP_JOURNAL")?);
    let protocol_journal = journal.with_extension("protocol.jsonl");
    // Create-only journal prevents accidental reuse of another invocation.
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&journal)?;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&protocol_journal)?;
    let service = StreamableHttpService::new(
        move || {
            Ok(Peer {
                journal: journal.clone(),
                protocol_journal: protocol_journal.clone(),
                tool_router: Peer::tool_router(),
            })
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    println!("PEER_URL http://{}/mcp", listener.local_addr()?);
    axum::serve(listener, axum::Router::new().nest_service("/mcp", service))
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
