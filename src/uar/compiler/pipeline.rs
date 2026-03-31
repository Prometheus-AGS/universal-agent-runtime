//! 8-stage compilation pipeline orchestrator.
//!
//! Transforms a parsed [`AgentDescriptorIR`] into a signed [`CompiledDescriptor`]
//! through 8 sequential stages, recording verdicts in a [`CompileReport`].

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::info;
use uuid::Uuid;

use super::error::{CompileError, CompileResult};
use super::ir::AgentDescriptorIR;
use super::registries::{EndpointBinding, EndpointRegistry, SchemaRegistry};
use super::report::{CompileOutcome, CompileReport, Diagnostic, DiagnosticLevel, StageVerdict};
use super::signing::KeyProvider;
use super::stages;

/// Compiled output: descriptor JSON + signature + compile report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompileOutput {
    /// The compiled agent descriptor as canonical JSON.
    pub descriptor: CompiledDescriptor,
    /// Ed25519 signature of the canonical JSON (hex-encoded).
    pub signature: String,
    /// The full compile report.
    pub report: CompileReport,
}

/// The final compiled descriptor — a JSON-serializable artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledDescriptor {
    /// Schema identifier.
    pub schema: String,
    /// Agent ID.
    pub agent_id: String,
    /// Agent version.
    pub version: String,
    /// Content SHA-256 hash (hex).
    pub content_hash: String,
    /// Public key of the signer (hex).
    pub signer_public_key: String,
    /// The full IR as the descriptor payload.
    pub payload: AgentDescriptorIR,
    /// Per-section content hashes for integrity checking.
    pub fingerprints: HashMap<String, String>,
    /// Compiled Cedar policy set (serialized).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cedar_policy: Option<String>,
    /// Registered endpoint bindings.
    pub endpoints: Vec<EndpointBinding>,
    /// PEP enforcement bindings.
    pub pep_bindings: Vec<PepBinding>,
}

/// A PEP enforcement binding — maps a capability surface to its policy check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PepBinding {
    /// Surface being guarded (e.g., "tool_execution", "llm_invocation").
    pub surface: String,
    /// Action ID in the Cedar policy.
    pub action: String,
    /// Whether this binding is mandatory.
    pub required: bool,
}

/// Mutable context passed through all 8 stages.
#[derive(Debug)]
pub struct CompileContext {
    /// The parsed IR (immutable reference).
    pub ir: AgentDescriptorIR,
    /// Compile report being built.
    pub report: CompileReport,
    /// Compiled Cedar policy (set by Stage 05).
    pub compiled_cedar: Option<String>,
    /// Registered actor endpoint routes (set by Stage 06).
    pub actor_routes: Vec<EndpointBinding>,
    /// PEP enforcement bindings (set by Stage 07).
    pub pep_bindings: Vec<PepBinding>,
    /// Per-section content hashes (set by various stages).
    pub fingerprints: HashMap<String, String>,
    /// The final compiled descriptor (set by Stage 08).
    pub descriptor: Option<CompiledDescriptor>,
    /// Schema registry reference.
    pub schema_registry: Arc<dyn SchemaRegistry>,
    /// Endpoint registry reference.
    pub endpoint_registry: Arc<dyn EndpointRegistry>,
    /// Key provider reference.
    pub key_provider: Arc<dyn KeyProvider>,
}

impl CompileContext {
    fn new(
        ir: AgentDescriptorIR,
        schema_registry: Arc<dyn SchemaRegistry>,
        endpoint_registry: Arc<dyn EndpointRegistry>,
        key_provider: Arc<dyn KeyProvider>,
    ) -> Self {
        let report = CompileReport {
            id: Uuid::new_v4().to_string(),
            agent_id: ir.agent_name.clone(),
            version: ir.metadata.version.clone(),
            timestamp: Utc::now(),
            stages: Vec::with_capacity(8),
            overall: CompileOutcome::Pass,
            total_duration_ms: 0,
        };

        Self {
            ir,
            report,
            compiled_cedar: None,
            actor_routes: Vec::new(),
            pep_bindings: Vec::new(),
            fingerprints: HashMap::new(),
            descriptor: None,
            schema_registry,
            endpoint_registry,
            key_provider,
        }
    }

    /// Record a stage verdict and update overall outcome.
    pub fn record_verdict(&mut self, verdict: StageVerdict) {
        if verdict.outcome == CompileOutcome::Fail {
            self.report.overall = CompileOutcome::Fail;
        }
        self.report.stages.push(verdict);
    }

    /// Compute SHA-256 hash of data and return hex string.
    pub fn sha256_hex(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        digest.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Macro to run a stage with timing, verdict recording, and error handling.
/// This avoids the async-closure-with-mutable-reference lifetime issue in Rust.
macro_rules! run_stage {
    ($ctx:expr, $stage_num:expr, $stage_name:expr, $stage_fn:path) => {{
        let start = Instant::now();
        match $stage_fn(&mut $ctx).await {
            Ok(diagnostics) => {
                let has_errors = diagnostics
                    .iter()
                    .any(|d| d.level == DiagnosticLevel::Error);
                let outcome = if has_errors {
                    CompileOutcome::Fail
                } else {
                    CompileOutcome::Pass
                };

                $ctx.record_verdict(StageVerdict {
                    stage: $stage_num,
                    name: $stage_name.into(),
                    outcome,
                    duration_ms: start.elapsed().as_millis() as u64,
                    diagnostics,
                });

                if outcome == CompileOutcome::Fail {
                    return Err(CompileError::StageFailure {
                        stage: $stage_num,
                        name: $stage_name.into(),
                        message: "stage produced error diagnostics".into(),
                    });
                }
            }
            Err(e) => {
                $ctx.record_verdict(StageVerdict {
                    stage: $stage_num,
                    name: $stage_name.into(),
                    outcome: CompileOutcome::Fail,
                    duration_ms: start.elapsed().as_millis() as u64,
                    diagnostics: vec![Diagnostic {
                        level: DiagnosticLevel::Error,
                        message: e.to_string(),
                        section: None,
                    }],
                });
                return Err(e);
            }
        }
    }};
}

/// Run the full 8-stage compilation pipeline.
pub async fn compile(
    ir: AgentDescriptorIR,
    schema_registry: Arc<dyn SchemaRegistry>,
    endpoint_registry: Arc<dyn EndpointRegistry>,
    key_provider: Arc<dyn KeyProvider>,
) -> CompileResult<CompileOutput> {
    let start = Instant::now();
    let mut ctx = CompileContext::new(ir, schema_registry, endpoint_registry, key_provider);

    info!(agent = %ctx.ir.agent_name, "starting compilation pipeline");

    // Stage 01: Validate Frontmatter
    run_stage!(ctx, 1, "Validate Frontmatter", stages::s01_frontmatter::run);

    // Stage 02: Validate A2UI Schemas
    run_stage!(ctx, 2, "Validate A2UI Schemas", stages::s02_a2ui::run);

    // Stage 03: Validate MCP Server Config
    run_stage!(ctx, 3, "Validate MCP Config", stages::s03_mcp::run);

    // Stage 04: Validate A2A JSON Schemas
    run_stage!(ctx, 4, "Validate A2A Schemas", stages::s04_a2a_schemas::run);

    // Stage 05: Compile Cedar Policy
    run_stage!(ctx, 5, "Compile Cedar Policy", stages::s05_cedar::run);

    // Stage 06: Register Actor Endpoints
    run_stage!(
        ctx,
        6,
        "Register Endpoints",
        stages::s06_actor_endpoints::run
    );

    // Stage 07: Install PEP Enforcement
    run_stage!(ctx, 7, "Install PEP", stages::s07_pep::run);

    // Stage 08: Emit Signed Descriptor
    run_stage!(ctx, 8, "Sign & Emit", stages::s08_emit::run);

    ctx.report.total_duration_ms = start.elapsed().as_millis() as u64;

    info!(
        agent = %ctx.report.agent_id,
        outcome = ?ctx.report.overall,
        duration_ms = ctx.report.total_duration_ms,
        "compilation pipeline complete"
    );

    let descriptor = ctx.descriptor.ok_or_else(|| CompileError::StageFailure {
        stage: 8,
        name: "Sign & Emit".into(),
        message: "descriptor was not emitted".into(),
    })?;

    // Sign the canonical JSON
    let canonical_json =
        serde_json::to_string(&descriptor).map_err(|e| CompileError::Internal(e.into()))?;
    let sig_bytes = ctx.key_provider.sign(canonical_json.as_bytes()).await?;
    let signature = hex_encode(&sig_bytes);

    Ok(CompileOutput {
        descriptor,
        signature,
        report: ctx.report,
    })
}

/// Hex encoding utility.
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
