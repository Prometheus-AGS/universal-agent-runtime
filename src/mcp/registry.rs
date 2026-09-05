use crate::mcp::config::{
    McpServerEntry, expand_env_map, expand_env_placeholders, expand_env_placeholders_strict,
    expand_from_environment, load_mcp_config,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rmcp::{
    model::{CallToolRequestParams, Tool, ToolAnnotations},
    service::ServiceExt,
    transport::{
        StreamableHttpClientTransport, TokioChildProcess,
        streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::process::Command;
use url::Url;
use uuid::Uuid;

use super::binding_cache::{
    ConnectedMcpServer, McpBindingError, McpBindingRequest, lifecycle_failure,
};
use super::catalog::ServerAuthentication;
use super::lifecycle::McpLifecycle;
use super::projection::ServerToolCatalog;
use super::stdio_process::StdioProcessSupervisor;
use crate::uar::domain::events::{McpServerState, McpStateReason};
use crate::uar::runtime::context::truncate::TruncationPolicy;
use crate::uar::tools::descriptor::{
    ApprovalClass, Exposure, ToolAssemblyError, ToolCollision, ToolDescriptor, ToolEffect,
    ToolSource,
};
use crate::uar::tools::validate::ValidatorCompiler;

/// How long one MCP server gets to complete its handshake before it is
/// skipped.
///
/// **No MCP server may prevent the runtime from serving prompts.** This is a
/// hard invariant, not a tuning knob: an agent runtime whose startup can be
/// blocked by a third-party tool process is one bad `npx` package away from
/// total unavailability, and the failure is silent — ports bind, nothing
/// answers, and no log names the cause.
///
/// 20s is generous for a local process handshake (a cold `npx` fetch is the
/// slow case) while bounding worst-case startup at
/// `servers × MCP_CONNECT_TIMEOUT` rather than infinity.
const MCP_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// How long a connected server gets to answer `tools/list`.
///
/// Separate from the connect budget because the failures differ: a handshake
/// hangs on process startup, `tools/list` hangs on a server that connected and
/// then stopped responding. Shorter, because the connection is already proven.
const MCP_LIST_TOOLS_TIMEOUT: Duration = Duration::from_secs(10);

/// How long any single tool call gets — MCP or native — before it is failed.
///
/// A tool that hangs mid-turn must surface as a failed tool result the model
/// can react to, never as a run that never ends. This was already applied to
/// MCP tool calls as a bare `30` and is now named and shared, so the native
/// path cannot silently diverge from it again.
const MCP_TOOL_CALL_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait NativeTool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> serde_json::Value;
    fn effect(&self) -> ToolEffect {
        ToolEffect::Unknown
    }
    fn approval_class(&self) -> ApprovalClass {
        match self.effect() {
            ToolEffect::ReadOnly => ApprovalClass::NotRequired,
            ToolEffect::ExternalMutation | ToolEffect::CodeExecution | ToolEffect::Unknown => {
                ApprovalClass::Required
            }
        }
    }
    fn sandbox_required(&self) -> bool {
        false
    }
    fn concurrency_key(&self) -> Option<&str> {
        None
    }
    fn exposure(&self) -> Exposure {
        Exposure::Eager
    }
    fn output_limit(&self) -> Option<TruncationPolicy> {
        None
    }
    /// Admit this captured in-process implementation for a delegated turn.
    /// Unknown implementations must not inherit unrestricted ambient access.
    fn check_thread_policy(
        &self,
        _policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        anyhow::bail!("In-process tool has no delegated permission contract")
    }
    /// Context is supplied by the host after schema validation and governance,
    /// never deserialized from model arguments. Legacy host calls are unchanged.
    async fn call_with_context(
        &self,
        args: Value,
        _context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        self.call(args).await
    }
    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

fn insert_descriptor(
    descriptors: &mut BTreeMap<String, Arc<ToolDescriptor>>,
    descriptor: Arc<ToolDescriptor>,
) -> Result<(), ToolAssemblyError> {
    if let Some(existing) = descriptors.get(&descriptor.provider_name) {
        if existing.equivalent_to(&descriptor) {
            return Ok(());
        }
        return Err(ToolCollision {
            provider_name: descriptor.provider_name.clone(),
        }
        .into());
    }
    descriptors.insert(descriptor.provider_name.clone(), descriptor);
    Ok(())
}

fn mcp_descriptor(
    compiler: &ValidatorCompiler,
    server: &str,
    provider_name: &str,
    tool: &Tool,
) -> Result<Arc<ToolDescriptor>, ToolAssemblyError> {
    let input_schema = Value::Object((*tool.input_schema).clone());
    let validator = compiler.compile(provider_name, &input_schema)?;
    let effect = if tool
        .annotations
        .as_ref()
        .is_some_and(|annotations| annotations.read_only_hint == Some(true))
    {
        ToolEffect::ReadOnly
    } else {
        ToolEffect::Unknown
    };
    Ok(Arc::new(ToolDescriptor {
        id: format!("{server}::{}", tool.name),
        provider_name: provider_name.to_string(),
        description: tool.description.as_deref().unwrap_or_default().to_string(),
        source: ToolSource::Mcp,
        server: Some(server.to_string()),
        input_schema,
        validator,
        effect,
        // MCP annotations are untrusted. readOnlyHint affects scheduling only.
        approval_class: ApprovalClass::Required,
        sandbox_required: false,
        concurrency_key: None,
        exposure: Exposure::Eager,
        output_limit: None,
    }))
}

fn native_tool_descriptor(
    compiler: &ValidatorCompiler,
    provider_name: &str,
    tool: &dyn NativeTool,
) -> Result<Arc<ToolDescriptor>, ToolAssemblyError> {
    let input_schema = tool.schema();
    let validator = compiler.compile(provider_name, &input_schema)?;
    Ok(Arc::new(ToolDescriptor {
        id: tool.name().to_string(),
        provider_name: provider_name.to_string(),
        description: tool.description().to_string(),
        source: ToolSource::BuiltIn,
        server: None,
        input_schema,
        validator,
        effect: tool.effect(),
        approval_class: tool.approval_class(),
        sandbox_required: tool.sandbox_required(),
        concurrency_key: tool.concurrency_key().map(str::to_owned),
        exposure: tool.exposure(),
        output_limit: tool.output_limit(),
    }))
}

type DynClientService = rmcp::service::RunningService<rmcp::service::RoleClient, ()>;
struct ClientServiceState {
    service: Arc<DynClientService>,
    // Replacement is synchronous, but transport close is asynchronous. Retain
    // old services here so cancellation of the replacing caller cannot detach
    // cleanup or let application shutdown overlook them.
    retired: Vec<Arc<DynClientService>>,
    reconnect_entry: McpServerEntry,
    snapshot: Option<Arc<SnapshotBinding>>,
    lifecycle: Option<(McpLifecycle, Uuid)>,
    generation: u64,
    reconnects_in_flight: usize,
    updates_in_flight: usize,
    shutting_down: bool,
}

struct SnapshotBinding {
    request: Arc<McpBindingRequest>,
    catalog: ServerToolCatalog,
    transport: SnapshotTransport,
}

#[derive(Clone)]
enum SnapshotTransport {
    Stdio(StdioProcessSupervisor),
    RemoteHttp,
}

type SharedClientService = Arc<RwLock<ClientServiceState>>;

struct BoundServices {
    connections: HashMap<String, Arc<DynClientService>>,
    closed: tokio_util::sync::CancellationToken,
}

#[derive(Default)]
struct RegistryAdmission {
    closed: AtomicBool,
    active: AtomicUsize,
    idle: tokio::sync::Notify,
    retired: RwLock<Vec<Arc<DynClientService>>>,
    retired_slots: RwLock<Vec<SharedClientService>>,
}

impl RegistryAdmission {
    fn enter(self: &Arc<Self>) -> anyhow::Result<RegistryAdmissionGuard> {
        if self.closed.load(Ordering::Acquire) {
            anyhow::bail!("MCP registry is shutting down");
        }
        self.active.fetch_add(1, Ordering::AcqRel);
        if self.closed.load(Ordering::Acquire) {
            self.leave();
            anyhow::bail!("MCP registry is shutting down");
        }
        Ok(RegistryAdmissionGuard {
            admission: Arc::clone(self),
        })
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        let retired = self
            .retired
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for service in &*retired {
            service.cancellation_token().cancel();
        }
        let slots = self
            .retired_slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in &*slots {
            begin_service_shutdown(slot).cancellation_token().cancel();
        }
    }

    fn retire(&self, service: DynClientService) {
        let service = Arc::new(service);
        service.cancellation_token().cancel();
        self.retired
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(service);
    }

    fn retire_slot(&self, slot: SharedClientService) {
        self.retired_slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(Arc::clone(&slot));
        begin_service_shutdown(&slot).cancellation_token().cancel();
    }

    fn leave(&self) {
        if self.active.fetch_sub(1, Ordering::AcqRel) == 1 {
            self.idle.notify_waiters();
        }
    }

    async fn join(&self) {
        loop {
            let idle = self.idle.notified();
            if self.active.load(Ordering::Acquire) == 0 {
                break;
            }
            idle.await;
        }
        let retired = self
            .retired
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for service in &retired {
            service.cancellation_token().cancel();
            while !service.is_transport_closed() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        self.retired
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|service| !retired.iter().any(|closed| Arc::ptr_eq(service, closed)));

        let slots = self
            .retired_slots
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for slot in &slots {
            let service = begin_service_shutdown(slot);
            service.cancellation_token().cancel();
            while !service_shutdown_complete(slot, &service) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
        self.retired_slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|slot| !slots.iter().any(|closed| Arc::ptr_eq(slot, closed)));
    }
}

struct RegistryAdmissionGuard {
    admission: Arc<RegistryAdmission>,
}

impl Drop for RegistryAdmissionGuard {
    fn drop(&mut self) {
        self.admission.leave();
    }
}

struct PendingClientService {
    service: Option<DynClientService>,
    admission: Arc<RegistryAdmission>,
}

impl PendingClientService {
    fn new(service: DynClientService, admission: Arc<RegistryAdmission>) -> Self {
        Self {
            service: Some(service),
            admission,
        }
    }

    fn service(&self) -> &DynClientService {
        self.service.as_ref().expect("pending MCP service is owned")
    }

    fn take(&mut self) -> DynClientService {
        self.service.take().expect("pending MCP service is owned")
    }
}

impl Drop for PendingClientService {
    fn drop(&mut self) {
        if let Some(service) = self.service.take() {
            self.admission.retire(service);
        }
    }
}

fn validate_bound_service(
    slot: &SharedClientService,
    bound: &Arc<DynClientService>,
) -> anyhow::Result<()> {
    let state = slot
        .read()
        .map_err(|_| anyhow!("MCP binding state unavailable"))?;
    if state.shutting_down || !Arc::ptr_eq(&state.service, bound) || bound.is_transport_closed() {
        anyhow::bail!("Frozen MCP connection was replaced, revoked, or closed");
    }
    Ok(())
}

fn new_service_slot(
    service: DynClientService,
    reconnect_entry: McpServerEntry,
) -> SharedClientService {
    Arc::new(RwLock::new(ClientServiceState {
        service: Arc::new(service),
        retired: Vec::new(),
        reconnect_entry,
        snapshot: None,
        lifecycle: None,
        generation: 0,
        reconnects_in_flight: 0,
        updates_in_flight: 0,
        shutting_down: false,
    }))
}

struct SlotUpdateGuard {
    slot: SharedClientService,
}

impl Drop for SlotUpdateGuard {
    fn drop(&mut self) {
        let mut state = self
            .slot
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.updates_in_flight -= 1;
    }
}

fn begin_slot_update(slot: &SharedClientService) -> anyhow::Result<SlotUpdateGuard> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    anyhow::ensure!(!state.shutting_down, "MCP server is shutting down");
    state.updates_in_flight += 1;
    Ok(SlotUpdateGuard {
        slot: Arc::clone(slot),
    })
}

fn current_service(slot: &SharedClientService) -> Arc<DynClientService> {
    Arc::clone(
        &slot
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .service,
    )
}

fn current_reconnect_entry(slot: &SharedClientService) -> (McpServerEntry, u64) {
    let state = slot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    (state.reconnect_entry.clone(), state.generation)
}

fn replace_configured_service(
    slot: &SharedClientService,
    service: DynClientService,
    reconnect_entry: McpServerEntry,
) -> bool {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let service = Arc::new(service);
    if state.shutting_down {
        service.cancellation_token().cancel();
        state.retired.push(service);
        return false;
    }
    let replaced = std::mem::replace(&mut state.service, service);
    replaced.cancellation_token().cancel();
    state.retired.push(replaced);
    state.reconnect_entry = reconnect_entry;
    state.snapshot = None;
    if let Some((lifecycle, generation)) = state.lifecycle.take() {
        lifecycle.transition(
            generation,
            McpServerState::ShuttingDown,
            Some(McpStateReason::Invalidated),
        );
    }
    state.generation = state.generation.wrapping_add(1);
    true
}

struct ReconnectAttempt {
    slot: SharedClientService,
    entry: McpServerEntry,
    snapshot: Option<Arc<SnapshotBinding>>,
    generation: u64,
}

impl Drop for ReconnectAttempt {
    fn drop(&mut self) {
        finish_reconnect(&self.slot, self.generation);
    }
}

fn begin_reconnect(slot: &SharedClientService) -> Option<ReconnectAttempt> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.shutting_down || state.reconnects_in_flight > 0 {
        return None;
    }
    state.reconnects_in_flight += 1;
    if let Some((lifecycle, generation)) = &state.lifecycle {
        lifecycle.transition(*generation, McpServerState::Connecting, None);
    }
    Some(ReconnectAttempt {
        slot: Arc::clone(slot),
        entry: state.reconnect_entry.clone(),
        snapshot: state.snapshot.clone(),
        generation: state.generation,
    })
}

fn install_reconnected_service(
    slot: &SharedClientService,
    expected_generation: u64,
    service: DynClientService,
) -> bool {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let service = Arc::new(service);
    if state.shutting_down || state.generation != expected_generation {
        service.cancellation_token().cancel();
        state.retired.push(service);
        return false;
    }
    let replaced = std::mem::replace(&mut state.service, service);
    replaced.cancellation_token().cancel();
    state.retired.push(replaced);
    if let Some((lifecycle, generation)) = &state.lifecycle {
        lifecycle.transition(*generation, McpServerState::Ready, None);
    }
    true
}

async fn reap_retired_services(slot: &SharedClientService) {
    let retired = slot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .retired
        .clone();
    for service in &retired {
        service.cancellation_token().cancel();
        while !service.is_transport_closed() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
    slot.write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .retired
        .retain(|service| !retired.iter().any(|closed| Arc::ptr_eq(service, closed)));
}

fn finish_reconnect(slot: &SharedClientService, expected_generation: u64) {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if state.generation == expected_generation
        && let Some((lifecycle, generation)) = &state.lifecycle
    {
        lifecycle.cancel_connecting(*generation);
    }
    state.reconnects_in_flight -= 1;
}

fn record_reconnect_failure(
    slot: &SharedClientService,
    expected_generation: u64,
    error: &anyhow::Error,
) {
    let state = slot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !state.shutting_down
        && state.generation == expected_generation
        && let Some((lifecycle, generation)) = &state.lifecycle
    {
        let (next, reason) = error
            .downcast_ref::<McpBindingError>()
            .map(lifecycle_failure)
            .unwrap_or((
                McpServerState::Failed,
                Some(McpStateReason::ConnectionFailed),
            ));
        lifecycle.transition(*generation, next, reason);
    }
}

fn begin_service_shutdown(slot: &SharedClientService) -> Arc<DynClientService> {
    let mut state = slot
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.shutting_down = true;
    if let Some((lifecycle, generation)) = &state.lifecycle {
        lifecycle.transition(*generation, McpServerState::ShuttingDown, None);
    }
    for service in &state.retired {
        service.cancellation_token().cancel();
    }
    Arc::clone(&state.service)
}

fn service_shutdown_complete(slot: &SharedClientService, service: &DynClientService) -> bool {
    let state = slot
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.reconnects_in_flight == 0
        && state.updates_in_flight == 0
        && service.is_transport_closed()
        && state
            .retired
            .iter()
            .all(|retired| retired.is_transport_closed())
}

async fn connect_server(name: &str, entry: &McpServerEntry) -> anyhow::Result<DynClientService> {
    entry.validate_sandbox_policy(name)?;
    match entry {
        McpServerEntry::Stdio {
            command, args, env, ..
        } => {
            let mut cmd = Command::new(resolve_mcp_command(command));
            cmd.args(args);
            for (key, value) in expand_env_map(env) {
                cmd.env(key, value);
            }
            let transport = TokioChildProcess::new(cmd)
                .with_context(|| format!("failed to spawn stdio MCP server '{name}'"))?;
            ().serve(transport)
                .await
                .with_context(|| format!("failed to connect stdio MCP server '{name}'"))
        }
        McpServerEntry::RemoteHttp { url, env } => {
            let env = expand_env_map(env);
            let endpoint = resolve_remote_http_url(name, url, &env)?;
            ().serve(StreamableHttpClientTransport::from_uri(
                endpoint.to_string(),
            ))
            .await
            .with_context(|| format!("failed to connect remote MCP server '{name}'"))
        }
    }
}

async fn connect_stdio_snapshot(
    request: &McpBindingRequest,
    processes: &StdioProcessSupervisor,
) -> Result<DynClientService, McpBindingError> {
    let definition = request.definition();
    let name = definition.name();
    definition
        .configuration()
        .validate_sandbox_policy(name)
        .map_err(|_| McpBindingError::InvalidBinding {
            server: name.to_owned(),
        })?;
    let McpServerEntry::Stdio {
        command, args, env, ..
    } = definition.configuration()
    else {
        return Err(McpBindingError::InvalidBinding {
            server: name.to_owned(),
        });
    };
    if matches!(
        definition.authentication(),
        ServerAuthentication::Unknown | ServerAuthentication::Required
    ) {
        return Err(McpBindingError::AuthenticationRequired {
            server: name.to_owned(),
        });
    }
    let executable =
        snapshot_command(command, request).ok_or_else(|| McpBindingError::ConnectionFailed {
            server: name.to_owned(),
        })?;
    let child_environment = snapshot_child_environment(request, env);
    let mut command = Command::new(executable);
    command
        .args(args)
        .current_dir(request.environment().directory())
        .env_clear()
        .envs(child_environment)
        .kill_on_drop(true);
    let transport = processes
        .spawn(command)
        .map_err(|_| McpBindingError::ConnectionFailed {
            server: name.to_owned(),
        })?;
    tokio::time::timeout(MCP_CONNECT_TIMEOUT, ().serve(transport))
        .await
        .map_err(|_| McpBindingError::ConnectionFailed {
            server: name.to_owned(),
        })?
        .map_err(|error| {
            if error.is_authorization_required() {
                McpBindingError::AuthenticationRequired {
                    server: name.to_owned(),
                }
            } else {
                McpBindingError::ConnectionFailed {
                    server: name.to_owned(),
                }
            }
        })
}

async fn connect_http_snapshot(
    request: &McpBindingRequest,
) -> Result<DynClientService, McpBindingError> {
    let definition = request.definition();
    let name = definition.name();
    let McpServerEntry::RemoteHttp { url, .. } = definition.configuration() else {
        return Err(McpBindingError::InvalidBinding {
            server: name.to_owned(),
        });
    };
    if matches!(
        definition.authentication(),
        ServerAuthentication::Unknown | ServerAuthentication::Required
    ) {
        return Err(McpBindingError::AuthenticationRequired {
            server: name.to_owned(),
        });
    }
    let endpoint = expand_from_environment(url, request.environment().variables())
        .ok()
        .and_then(|expanded| Url::parse(&expanded).ok())
        .ok_or_else(|| McpBindingError::ConnectionFailed {
            server: name.to_owned(),
        })?;
    let client = reqwest_mcp::Client::builder()
        .no_proxy()
        .redirect(reqwest_mcp::redirect::Policy::none())
        .build()
        .map_err(|_| McpBindingError::ConnectionFailed {
            server: name.to_owned(),
        })?;
    let transport = StreamableHttpClientTransport::with_client(
        client,
        StreamableHttpClientTransportConfig::with_uri(endpoint.to_string()),
    );
    tokio::time::timeout(MCP_CONNECT_TIMEOUT, ().serve(transport))
        .await
        .map_err(|_| McpBindingError::ConnectionFailed {
            server: name.to_owned(),
        })?
        .map_err(|error| {
            if error.is_authorization_required() {
                McpBindingError::AuthenticationRequired {
                    server: name.to_owned(),
                }
            } else {
                McpBindingError::ConnectionFailed {
                    server: name.to_owned(),
                }
            }
        })
}

fn snapshot_command(command: &str, request: &McpBindingRequest) -> Option<PathBuf> {
    let path = Path::new(command);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }
    let environment = request.environment();
    if let Some(directory) = environment
        .variables()
        .get(std::ffi::OsStr::new("MCP_SERVER_DIR"))
    {
        let candidate = environment.directory().join(directory).join(path);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    if path.components().count() > 1 {
        return Some(environment.directory().join(path));
    }
    let search = environment.variables().get(std::ffi::OsStr::new("PATH"))?;
    for directory in std::env::split_paths(search) {
        let candidate = environment.directory().join(directory).join(path);
        if executable_file(&candidate) {
            return Some(candidate);
        }
        #[cfg(windows)]
        if path.extension().is_none() {
            let extensions = environment
                .variables()
                .get(std::ffi::OsStr::new("PATHEXT"))
                .and_then(|value| value.to_str())
                .unwrap_or(".EXE");
            for extension in extensions.split(';').filter(|value| !value.is_empty()) {
                let candidate = candidate.with_extension(extension.trim_start_matches('.'));
                if executable_file(&candidate) {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn snapshot_child_environment(
    request: &McpBindingRequest,
    declared: &HashMap<String, String>,
) -> BTreeMap<std::ffi::OsString, std::ffi::OsString> {
    const LAUNCH_KEYS: &[&str] = &[
        "PATH",
        "PATHEXT",
        "SYSTEMROOT",
        "WINDIR",
        "COMSPEC",
        "TMPDIR",
        "TEMP",
        "TMP",
    ];
    let captured = request.environment().variables();
    let mut environment = LAUNCH_KEYS
        .iter()
        .filter_map(|key| {
            captured
                .get(std::ffi::OsStr::new(key))
                .map(|value| (std::ffi::OsString::from(key), value.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    for key in declared.keys() {
        if let Some(value) = captured.get(std::ffi::OsStr::new(key)) {
            environment.insert(std::ffi::OsString::from(key), value.clone());
        }
    }
    environment
}

fn executable_file(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

struct DiscoveredTools {
    tools: Vec<(String, Tool)>,
    index: HashMap<String, (String, String)>,
    descriptors: BTreeMap<String, Arc<ToolDescriptor>>,
}

fn compile_discovered_tools(
    compiler: &ValidatorCompiler,
    server: &str,
    tools: Vec<Tool>,
) -> anyhow::Result<DiscoveredTools> {
    let mut discovered = DiscoveredTools {
        tools: Vec::new(),
        index: HashMap::new(),
        descriptors: BTreeMap::new(),
    };
    for tool in tools {
        let raw_name = tool.name.to_string();
        anyhow::ensure!(
            !raw_name.trim().is_empty(),
            "MCP server {server:?} returned an empty tool name"
        );
        let name = McpRegistry::sanitize_tool_name(&format!("{server}__{raw_name}"));
        let descriptor = mcp_descriptor(compiler, server, &name, &tool)?;
        insert_descriptor(&mut discovered.descriptors, descriptor)?;
        if !discovered.index.contains_key(&name) {
            discovered
                .index
                .insert(name.clone(), (server.to_owned(), raw_name));
            discovered.tools.push((name, tool));
        }
    }
    Ok(discovered)
}

async fn reconnect_snapshot(
    snapshot: &SnapshotBinding,
    compiler: &ValidatorCompiler,
) -> anyhow::Result<DynClientService> {
    let service = match &snapshot.transport {
        SnapshotTransport::Stdio(processes) => {
            connect_stdio_snapshot(&snapshot.request, processes).await?
        }
        SnapshotTransport::RemoteHttp => connect_http_snapshot(&snapshot.request).await?,
    };
    let server = snapshot.request.definition().name();
    let catalog_check = async {
        let tools = service.list_all_tools().await?;
        let discovered = compile_discovered_tools(compiler, server, tools)?;
        anyhow::ensure!(
            discovered.descriptors.len() == snapshot.catalog.tools().len()
                && discovered.descriptors.iter().all(|(name, actual)| {
                    snapshot
                        .catalog
                        .tools()
                        .get(name)
                        .is_some_and(|expected| expected.equivalent_to(actual))
                }),
            "MCP server {server:?} changed its catalog while reconnecting"
        );
        Ok::<_, anyhow::Error>(())
    };
    match tokio::time::timeout(MCP_LIST_TOOLS_TIMEOUT, catalog_check).await {
        Ok(Ok(())) => Ok(service),
        _ => {
            let _ = service.cancel().await;
            Err(McpBindingError::IncompleteCatalog {
                server: server.to_owned(),
            }
            .into())
        }
    }
}

/// Expands `${VAR}` and `${VAR:-default}` placeholders in a remote MCP
/// server's `url` (per the process environment, same as its `env` map).
/// Tavily's config style embeds `${TAVILY_API_KEY}` directly in the URL and
/// additionally declares it in `env`; if the placeholder is still present
/// after process-env expansion, this substitutes it from the entry's own `env`
/// map, matching that convention. Any other `RemoteHttp` entry (e.g.
/// `surreal_memory`, which has no `TAVILY_API_KEY` at all) is left alone --
/// requiring a Tavily-specific key for every remote server was the bug that
/// took down the whole registry whenever a non-Tavily entry was present.
///
/// The Tavily substitution runs first, against the lenient expansion, so its
/// `env`-map indirection still works. Only afterwards is the result required
/// to be placeholder-free: an unexpanded `${...}` is a configuration error, not
/// a hostname, and must not reach the URL parser verbatim.
fn resolve_remote_http_url(
    name: &str,
    url: &str,
    env: &HashMap<String, String>,
) -> anyhow::Result<Url> {
    let mut expanded = expand_env_placeholders(url);
    if expanded.contains("${TAVILY_API_KEY}") {
        let api_key = env
            .get("TAVILY_API_KEY")
            .cloned()
            .filter(|key| !key.is_empty())
            .ok_or_else(|| anyhow!("remote MCP '{name}' missing TAVILY_API_KEY"))?;
        expanded = expanded.replace("${TAVILY_API_KEY}", &api_key);
    }
    let expanded = expand_env_placeholders_strict(&expanded)
        .with_context(|| format!("cannot resolve url for remote MCP '{name}'"))?;
    Url::parse(&expanded).with_context(|| format!("invalid url for remote MCP '{name}'"))
}

#[derive(Clone)]
pub struct McpRegistry {
    services: Arc<RwLock<HashMap<String, SharedClientService>>>,
    /// A delegated view pins concrete transports, not reconnectable identities.
    bound_services: Option<Arc<BoundServices>>,
    shutting_down: Arc<AtomicBool>,
    admission: Arc<RegistryAdmission>,
    server_config: Arc<RwLock<HashMap<String, McpServerEntry>>>,
    // namespaced_tool_name -> (server_name, tool_name)
    tool_index: Arc<RwLock<HashMap<String, (String, String)>>>,
    tools: Arc<RwLock<Vec<(String, Tool)>>>, // (namespaced_name, Tool)
    descriptors: Arc<RwLock<BTreeMap<String, Arc<ToolDescriptor>>>>,
    validator_compiler: Arc<ValidatorCompiler>,
    // namespaced_tool_name -> NativeTool
    native_tools: Arc<HashMap<String, Arc<dyn NativeTool>>>,
}

/// Registry composition must not turn an immutable delegation into a new grant.
#[derive(Debug, thiserror::Error)]
pub enum McpMergeError {
    #[error(transparent)]
    Collision(#[from] ToolCollision),
    #[error("cannot merge executable resources into a frozen MCP binding")]
    FrozenBinding,
}

impl std::fmt::Debug for McpRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpRegistry")
            .field(
                "tool_count",
                &self
                    .tools
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .len(),
            )
            .field(
                "service_count",
                &self
                    .services
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .len(),
            )
            .field("native_tool_count", &self.native_tools.len())
            .finish()
    }
}

impl McpRegistry {
    pub(crate) fn attach_lifecycle(&self, lifecycle: McpLifecycle, generation: Uuid) {
        let services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in services.values() {
            slot.write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .lifecycle = Some((lifecycle.clone(), generation));
        }
    }

    /// Connect one stdio server from immutable host inputs and discover all pages.
    /// No process-global env lookup or implicit command provisioning occurs here.
    ///
    /// # Errors
    /// Rejects wrong transport, unsupported sandbox, missing command/authentication,
    /// bounded handshake/discovery failure, invalid schemas and tool collisions.
    pub(crate) async fn connect_stdio_binding(
        request: Arc<McpBindingRequest>,
        processes: StdioProcessSupervisor,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        let service = connect_stdio_snapshot(&request, &processes).await?;
        Self::connect_snapshot_binding(request, service, SnapshotTransport::Stdio(processes)).await
    }

    /// Connect one remote HTTP server from immutable host inputs and discover
    /// all pages. The concrete client ignores ambient proxy variables.
    pub(crate) async fn connect_http_binding(
        request: Arc<McpBindingRequest>,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        let service = connect_http_snapshot(&request).await?;
        Self::connect_snapshot_binding(request, service, SnapshotTransport::RemoteHttp).await
    }

    async fn connect_snapshot_binding(
        request: Arc<McpBindingRequest>,
        service: DynClientService,
        transport: SnapshotTransport,
    ) -> Result<ConnectedMcpServer, McpBindingError> {
        let server = request.definition().name().to_owned();
        let compiler = Arc::new(ValidatorCompiler::default());
        let discovery = async {
            let tools = service.list_all_tools().await?;
            let discovered = compile_discovered_tools(&compiler, &server, tools)?;
            let catalog = ServerToolCatalog::new(
                Arc::clone(request.definition()),
                discovered.descriptors.values().cloned(),
                true,
            )?;
            Ok::<_, anyhow::Error>((discovered, catalog))
        };
        let (discovered, catalog) =
            match tokio::time::timeout(MCP_LIST_TOOLS_TIMEOUT, discovery).await {
                Ok(Ok(discovered)) => discovered,
                _ => {
                    let _ = service.cancel().await;
                    return Err(McpBindingError::IncompleteCatalog { server });
                }
            };
        let configuration = request.definition().configuration().clone();
        let slot = new_service_slot(service, configuration.clone());
        slot.write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot = Some(Arc::new(SnapshotBinding {
            request,
            catalog: catalog.clone(),
            transport,
        }));
        let registry = Self {
            services: Arc::new(RwLock::new(HashMap::from([(server.clone(), slot)]))),
            bound_services: None,
            shutting_down: Arc::new(AtomicBool::new(false)),
            admission: Arc::new(RegistryAdmission::default()),
            server_config: Arc::new(RwLock::new(HashMap::from([(server, configuration)]))),
            tool_index: Arc::new(RwLock::new(discovered.index)),
            tools: Arc::new(RwLock::new(discovered.tools)),
            descriptors: Arc::new(RwLock::new(discovered.descriptors)),
            validator_compiler: compiler,
            native_tools: Arc::new(HashMap::new()),
        };
        Ok(ConnectedMcpServer::new(registry, catalog))
    }

    /// Create an empty registry with no MCP servers or tools.
    pub fn empty() -> Self {
        let validator_compiler = Arc::new(ValidatorCompiler::default());
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            bound_services: None,
            shutting_down: Arc::new(AtomicBool::new(false)),
            admission: Arc::new(RegistryAdmission::default()),
            server_config: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
            descriptors: Arc::new(RwLock::new(BTreeMap::new())),
            validator_compiler,
            native_tools: Arc::new(HashMap::new()),
        }
    }

    pub async fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let resolved = resolve_mcp_config_path(path);
        let cfg = load_mcp_config(resolved)?;
        Self::from_config(&cfg).await
    }

    pub async fn from_config(cfg: &crate::mcp::config::McpConfig) -> anyhow::Result<Self> {
        cfg.validate_sandbox_policy()?;
        // 1) connect all servers
        let mut services: HashMap<String, SharedClientService> = HashMap::new();

        for (name, entry) in &cfg.mcp_servers {
            // A server that never completes its handshake must not be able to
            // hang startup. Observed 2026-09-01: a stdio server printed
            // "running on stdio" and neither side ever finished, so the
            // process bound its ports and then served nothing, forever, with
            // no log line naming the cause. Error handling below was already
            // correct — a failure is skipped and the registry continues — but
            // "never answers" is not an error, so it never reached it.
            let svc = match tokio::time::timeout(
                MCP_CONNECT_TIMEOUT,
                Self::connect_configured_server(name, entry),
            )
            .await
            {
                Err(_elapsed) => {
                    crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
                    tracing::warn!(
                        server = %name,
                        timeout_secs = MCP_CONNECT_TIMEOUT.as_secs(),
                        "MCP server did not complete its handshake in time; skipping it, registry continues without it"
                    );
                    continue;
                }
                Ok(Ok(svc)) => svc,
                Ok(Err(e)) => {
                    crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
                    tracing::warn!(server = %name, error = ?e, "MCP server failed to connect; skipping it, registry continues without it");
                    continue;
                }
            };
            crate::uar::telemetry::metrics::set_mcp_server_status(name, true);
            services.insert(name.clone(), new_service_slot(svc, entry.clone()));
        }

        // 2) list tools + build index
        let mut all_tools: Vec<(String, Tool)> = Vec::new();
        let mut tool_index: HashMap<String, (String, String)> = HashMap::new();
        let mut descriptors = BTreeMap::new();
        let validator_compiler = Arc::new(ValidatorCompiler::default());

        for (server_name, service_slot) in &services {
            let svc = current_service(service_slot);
            // list_tools exists on the rmcp running service in examples
            let result = match tokio::time::timeout(
                MCP_LIST_TOOLS_TIMEOUT,
                svc.list_tools(Default::default()),
            )
            .await
            {
                Err(_elapsed) => {
                    tracing::warn!(
                        server = %server_name,
                        timeout_secs = MCP_LIST_TOOLS_TIMEOUT.as_secs(),
                        "tools/list timed out for MCP server; skipping its tools, registry continues without them"
                    );
                    continue;
                }
                Ok(Ok(result)) => result,
                Ok(Err(e)) => {
                    tracing::warn!(server = %server_name, error = ?e, "tools/list failed for MCP server; skipping its tools, registry continues without them");
                    continue;
                }
            };

            for t in result.tools {
                let tool_name = t.name.to_string();
                // Sanitize tool name for OpenAI compatibility
                // OpenAI requires: ^[a-zA-Z0-9_-]+$ (no colons, dots, or special chars)
                // Replace :: with __ for namespacing, and sanitize any other invalid chars
                let ns_name = Self::sanitize_tool_name(&format!("{server_name}__{tool_name}"));
                insert_descriptor(
                    &mut descriptors,
                    mcp_descriptor(&validator_compiler, server_name, &ns_name, &t)?,
                )?;
                tool_index.insert(ns_name.clone(), (server_name.clone(), tool_name));
                all_tools.push((ns_name, t));
            }
        }

        Ok(Self {
            services: Arc::new(RwLock::new(services)),
            bound_services: None,
            shutting_down: Arc::new(AtomicBool::new(false)),
            admission: Arc::new(RegistryAdmission::default()),
            server_config: Arc::new(RwLock::new(cfg.mcp_servers.clone())),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(all_tools)),
            descriptors: Arc::new(RwLock::new(descriptors)),
            validator_compiler,
            native_tools: Arc::new(HashMap::new()),
        })
    }

    /// Connects a single configured MCP server. Split out of `from_config` so
    /// one server's failure (bad URL, unreachable process, missing key) can
    /// be caught and logged per-server instead of aborting the whole
    /// registry load -- previously any single misconfigured entry (e.g. a
    /// non-Tavily `RemoteHttp` server tripping the old hardcoded
    /// `TAVILY_API_KEY` requirement) took every other server down with it.
    async fn connect_configured_server(
        name: &str,
        entry: &McpServerEntry,
    ) -> anyhow::Result<DynClientService> {
        entry.validate_sandbox_policy(name)?;
        match entry {
            McpServerEntry::Stdio {
                command, args, env, ..
            } => {
                let env = expand_env_map(env);

                let mut command_path = resolve_mcp_command(command);
                // Provisioning: only attempted when the configured
                // command isn't already resolvable (preserves the fast
                // path for the common case where it's already
                // installed). A curated ToolSpec exists for `kreuzberg`;
                // any other name gets an Adopt-only spec, so this never
                // silently installs something for an uncurated tool —
                // it just surfaces a clearer error than the raw spawn
                // failure below would.
                if !crate::uar::orchestrator::provisioning::is_on_path(
                    &command_path.to_string_lossy(),
                ) {
                    let spec = crate::uar::orchestrator::provisioning::known_tool_spec(command);
                    let opts = crate::uar::orchestrator::provisioning::ProvisionOptions::default();
                    match crate::uar::orchestrator::provisioning::ToolProvisioner::resolve(
                        &spec, &opts,
                    )
                    .await
                    {
                        Ok(outcome) => {
                            tracing::info!(
                                server = %name,
                                command = %command,
                                strategy = ?outcome.strategy,
                                path = %outcome.path.display(),
                                "provisioned MCP server command"
                            );
                            command_path = outcome.path;
                        }
                        Err(e) => {
                            tracing::warn!(
                                server = %name,
                                command = %command,
                                error = %e,
                                "could not provision MCP server command; falling back to spawning it as configured"
                            );
                        }
                    }
                }
                let mut cmd = Command::new(&command_path);
                cmd.args(args);

                for (k, v) in env {
                    cmd.env(k, v);
                }

                // rmcp docs show TokioChildProcess + configure pattern for adding args
                let transport = TokioChildProcess::new(cmd)?;
                ().serve(transport)
                    .await
                    .with_context(|| format!("failed to connect stdio MCP server '{name}'"))
            }

            McpServerEntry::RemoteHttp { url, env } => {
                let env = expand_env_map(env);
                let endpoint = resolve_remote_http_url(name, url, &env)?;
                ().serve(StreamableHttpClientTransport::from_uri(
                    endpoint.to_string(),
                ))
                .await
                .with_context(|| format!("failed to connect remote MCP server '{name}'"))
            }
        }
    }

    /// Creates an empty registry for testing.
    pub fn new_empty() -> Self {
        Self::empty()
    }

    /// Creates a registry with a single test tool.
    pub fn new_with_test_tool(name: &str, description: &str) -> Self {
        Self::new_with_test_tool_annotations(name, description, ToolAnnotations::new())
            .expect("the static test tool schema is valid")
    }

    /// Creates a registry with one annotated test tool.
    ///
    /// # Errors
    ///
    /// Returns an assembly error if the static schema or generated descriptor
    /// is invalid.
    pub fn new_with_test_tool_annotations(
        name: &str,
        description: &str,
        annotations: ToolAnnotations,
    ) -> Result<Self, ToolAssemblyError> {
        Self::new_with_test_tool_for_server("test", name, description, annotations)
    }

    /// Creates a registry with one annotated test tool under `server`.
    ///
    /// # Errors
    ///
    /// Returns an assembly error if the static schema or generated descriptor
    /// is invalid.
    pub fn new_with_test_tool_for_server(
        server: &str,
        name: &str,
        description: &str,
        annotations: ToolAnnotations,
    ) -> Result<Self, ToolAssemblyError> {
        let ns_name = Self::sanitize_tool_name(&format!("{server}__{name}"));
        // rmcp 1.8: Tool is #[non_exhaustive] -- struct-literal syntax (even
        // with ..Default::default()) is rejected cross-crate; use Tool::new.
        let tool = Tool::new(
            name.to_string(),
            description.to_string(),
            serde_json::json!({
                "type": "object",
                "properties": {
                    "mirror": { "type": "string" }
                },
                "required": ["mirror"]
            })
            .as_object()
            .unwrap()
            .clone(),
        )
        .with_annotations(annotations);

        let validator_compiler = Arc::new(ValidatorCompiler::default());
        let descriptor = mcp_descriptor(&validator_compiler, server, &ns_name, &tool)?;
        let tools = vec![(ns_name.clone(), tool)];
        let mut tool_index = HashMap::new();
        tool_index.insert(ns_name.clone(), (server.to_string(), name.to_string()));
        let descriptors = BTreeMap::from([(ns_name, descriptor)]);

        Ok(Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            bound_services: None,
            shutting_down: Arc::new(AtomicBool::new(false)),
            admission: Arc::new(RegistryAdmission::default()),
            server_config: Arc::new(RwLock::new(HashMap::new())),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(tools)),
            descriptors: Arc::new(RwLock::new(descriptors)),
            validator_compiler,
            native_tools: Arc::new(HashMap::new()),
        })
    }

    /// Sanitize tool names for `OpenAI` API compatibility.
    fn sanitize_tool_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    // Replace dots, colons, and any other invalid chars with underscore
                    '_'
                }
            })
            .collect()
    }

    /// Return namespaced tools as `(namespaced_name, Tool)`
    pub fn tools(&self) -> Vec<(String, Tool)> {
        self.tools
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn descriptor_map(&self) -> BTreeMap<String, Arc<ToolDescriptor>> {
        self.descriptors
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Return compiled descriptors in provider-name order.
    pub fn descriptors(&self) -> Vec<Arc<ToolDescriptor>> {
        self.descriptors
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect()
    }

    /// Look up one compiled descriptor by provider-visible name.
    pub fn descriptor(&self, provider_name: &str) -> Option<Arc<ToolDescriptor>> {
        self.descriptors
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(provider_name)
            .cloned()
    }

    /// Return configured MCP server names.
    pub fn server_names(&self) -> Vec<String> {
        self.services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect()
    }

    /// Freeze the selected descriptors and exact live transports for delegation.
    /// Re-list selected remote tools on those same transports to reject a
    /// configuration/catalog replacement racing snapshot capture. New tools
    /// are not added. This view cannot connect, reconnect, merge, or add tools.
    ///
    /// # Errors
    /// Refuses missing/closed bindings, changed descriptors, or failed discovery.
    pub async fn freeze_bindings(&self) -> anyhow::Result<Self> {
        if self.bound_services.is_some() {
            self.require_bound_servers(self.server_names().iter().map(String::as_str))?;
            return Ok(self.filtered(None, None));
        }
        if self.shutting_down.load(Ordering::Acquire) {
            anyhow::bail!("MCP registry is shutting down");
        }
        let mut frozen = self.filtered(None, None);
        let slots = frozen
            .services
            .read()
            .map_err(|_| anyhow!("MCP service index unavailable"))?
            .clone();
        let mut connections = HashMap::new();
        for (name, slot) in &slots {
            let service = current_service(slot);
            validate_bound_service(slot, &service)?;
            connections.insert(name.clone(), service);
        }
        let descriptors = frozen.descriptors();
        let tool_index = frozen
            .tool_index
            .read()
            .map_err(|_| anyhow!("MCP tool index unavailable"))?
            .clone();
        for (server, _) in tool_index.values() {
            if !connections.contains_key(server) {
                anyhow::bail!("Selected MCP tool has no live binding");
            }
        }
        for (server, service) in &connections {
            let selected = descriptors
                .iter()
                .filter(|descriptor| descriptor.server.as_deref() == Some(server.as_str()))
                .collect::<Vec<_>>();
            if selected.is_empty() {
                continue;
            }
            let discovered = tokio::time::timeout(MCP_LIST_TOOLS_TIMEOUT, service.list_all_tools())
                .await
                .context("Frozen MCP catalog discovery timed out")??;
            for descriptor in selected {
                let (_, raw_name) = tool_index
                    .get(&descriptor.provider_name)
                    .ok_or_else(|| anyhow!("Selected MCP descriptor has no tool identity"))?;
                let tool = discovered
                    .iter()
                    .find(|tool| tool.name.as_ref() == raw_name.as_str())
                    .ok_or_else(|| anyhow!("Selected MCP tool disappeared during binding"))?;
                let current = mcp_descriptor(
                    &frozen.validator_compiler,
                    server,
                    &descriptor.provider_name,
                    tool,
                )?;
                if !descriptor.equivalent_to(&current) {
                    anyhow::bail!("Selected MCP descriptor changed during binding");
                }
            }
        }
        frozen.bound_services = Some(Arc::new(BoundServices {
            connections,
            closed: tokio_util::sync::CancellationToken::new(),
        }));
        *frozen
            .server_config
            .write()
            .map_err(|_| anyhow!("MCP configuration index unavailable"))? = HashMap::new();
        frozen.require_bound_servers(frozen.server_names().iter().map(String::as_str))?;
        Ok(frozen)
    }

    /// Whether this registry is a captured connection view, not a catalog that
    /// may establish connections from configuration or environment variables.
    pub fn has_frozen_bindings(&self) -> bool {
        self.bound_services.is_some()
    }

    /// Validate inherited dependencies without starting or reconfiguring them.
    ///
    /// # Errors
    /// Refuses a non-frozen registry or any unavailable/replaced connection.
    pub fn require_bound_servers<'a>(
        &self,
        servers: impl IntoIterator<Item = &'a str>,
    ) -> anyhow::Result<()> {
        let bindings = self
            .bound_services
            .as_ref()
            .ok_or_else(|| anyhow!("MCP bindings are not frozen"))?;
        if bindings.closed.is_cancelled() || self.shutting_down.load(Ordering::Acquire) {
            anyhow::bail!("Frozen MCP view is closed");
        }
        let slots = self
            .services
            .read()
            .map_err(|_| anyhow!("MCP service index unavailable"))?;
        for server in servers {
            let service = bindings
                .connections
                .get(server)
                .ok_or_else(|| anyhow!("Required MCP binding is unavailable"))?;
            let slot = slots
                .get(server)
                .ok_or_else(|| anyhow!("Required MCP service slot is unavailable"))?;
            validate_bound_service(slot, service)?;
        }
        Ok(())
    }

    /// Return whether the namespaced tool is backed by an in-process native tool.
    pub fn is_native_tool(&self, namespaced_name: &str) -> bool {
        self.native_tools.contains_key(namespaced_name)
    }

    /// Check the actual captured implementation, not a descriptor's effect.
    pub(crate) fn check_native_thread_policy(
        &self,
        name: &str,
        policy: &crate::uar::runtime::thread::policy_intersection::ThreadPolicy,
    ) -> anyhow::Result<()> {
        self.native_tools
            .get(name)
            .ok_or_else(|| anyhow!("In-process tool is unavailable"))?
            .check_thread_policy(policy)
    }

    /// Governed contextual entry for in-process tools only. Remote MCP keeps
    /// its frozen connection/preflight path and cannot enter this adapter.
    pub(crate) async fn call_native_with_context(
        &self,
        name: &str,
        arguments: Value,
        context: &crate::uar::runtime::native_skill::NativeExecutionContext,
    ) -> anyhow::Result<Value> {
        if self.bound_services.as_ref().is_some_and(|bindings| {
            bindings.closed.is_cancelled() || self.shutting_down.load(Ordering::Acquire)
        }) {
            anyhow::bail!("Frozen MCP view is closed");
        }
        let tool = self
            .native_tools
            .get(name)
            .ok_or_else(|| anyhow!("In-process tool is unavailable"))?;
        if let Some(policy) = &context.thread_policy {
            anyhow::ensure!(
                context
                    .verified_owner
                    .as_ref()
                    .is_some_and(|owner| owner.user_id() == policy.owner_id()),
                "In-process tool has no matching verified owner"
            );
            tool.check_thread_policy(policy)?;
        }
        match tokio::time::timeout(
            MCP_TOOL_CALL_TIMEOUT,
            tool.call_with_context(arguments, context),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(anyhow!(
                "native tool {name} timed out after {}s",
                MCP_TOOL_CALL_TIMEOUT.as_secs()
            )),
        }
    }

    /// Resolve the backing MCP server and raw tool name for a namespaced tool.
    pub fn resolve_mcp_tool(&self, namespaced_name: &str) -> Option<(String, String)> {
        self.tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(namespaced_name)
            .cloned()
    }

    /// Return persisted runtime MCP definitions without exposing connected handles.
    pub fn server_entries(&self) -> HashMap<String, McpServerEntry> {
        // A delegated connection is not authority to reconnect from a recipe.
        if self.bound_services.is_some() {
            return HashMap::new();
        }
        let mut entries = self
            .server_config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (name, service_slot) in self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            entries.insert(name.clone(), current_reconnect_entry(service_slot).0);
        }
        entries
    }

    /// Add or replace one MCP server and immediately refresh its advertised tools.
    pub async fn upsert_server(&self, name: String, entry: McpServerEntry) -> anyhow::Result<()> {
        if self.bound_services.is_some() {
            anyhow::bail!("Cannot replace a frozen MCP binding");
        }
        let _admission = self.admission.enter()?;
        // A merged/filtered registry may own an independent service index but
        // share this slot. Register before replacement I/O so shutdown through
        // any view cannot declare the slot closed while a producer remains.
        let mut slot_updates = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&name)
            .cloned()
            .map(|slot| begin_slot_update(&slot))
            .transpose()?
            .into_iter()
            .collect::<Vec<_>>();
        let service = tokio::time::timeout(
            MCP_CONNECT_TIMEOUT,
            Self::connect_configured_server(&name, &entry),
        )
        .await
        .context("MCP server handshake timed out")??;
        let mut pending = PendingClientService::new(service, Arc::clone(&self.admission));
        let result = tokio::time::timeout(
            MCP_LIST_TOOLS_TIMEOUT,
            pending.service().list_tools(Default::default()),
        )
        .await
        .context("MCP tools/list timed out")?
        .with_context(|| format!("tools/list failed for MCP server '{name}'"))?;

        let mut discovered = Vec::new();
        let mut indexed = Vec::new();
        let mut discovered_descriptors = Vec::new();
        for tool in result.tools {
            let raw_name = tool.name.to_string();
            let namespaced = Self::sanitize_tool_name(&format!("{name}__{raw_name}"));
            discovered_descriptors.push(mcp_descriptor(
                &self.validator_compiler,
                &name,
                &namespaced,
                &tool,
            )?);
            indexed.push((namespaced.clone(), (name.clone(), raw_name)));
            discovered.push((namespaced, tool));
        }

        let mut next_descriptors = self
            .descriptors
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        next_descriptors
            .retain(|_, descriptor| descriptor.server.as_deref() != Some(name.as_str()));
        for descriptor in discovered_descriptors {
            insert_descriptor(&mut next_descriptors, descriptor)?;
        }

        let replaced_slot = {
            let mut services = self
                .services
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if self.shutting_down.load(Ordering::Acquire)
                || self.admission.closed.load(Ordering::Acquire)
            {
                return Err(anyhow!("MCP registry is shutting down"));
            }
            if let Some(service_slot) = services.get(&name) {
                // Another admitted upsert may have installed this name after
                // our initial lookup. Register with the actual shared slot
                // before publishing a replacement into it.
                if !slot_updates
                    .iter()
                    .any(|guard| Arc::ptr_eq(&guard.slot, service_slot))
                {
                    slot_updates.push(begin_slot_update(service_slot)?);
                }
                let service = pending.take();
                let installed = replace_configured_service(service_slot, service, entry.clone());
                Some((Arc::clone(service_slot), installed))
            } else {
                let service = pending.take();
                services.insert(name.clone(), new_service_slot(service, entry.clone()));
                None
            }
        };
        if let Some((slot, installed)) = replaced_slot {
            reap_retired_services(&slot).await;
            if !installed {
                return Err(anyhow!("MCP server is shutting down"));
            }
        }
        self.server_config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(name.clone(), entry);
        {
            let mut index = self
                .tool_index
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            index.retain(|_, (server, _)| server != &name);
            index.extend(indexed);
        }
        {
            let mut tools = self
                .tools
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            tools.retain(|(tool_name, _)| {
                self.native_tools.contains_key(tool_name)
                    || self
                        .tool_index
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .get(tool_name)
                        .is_some_and(|(server, _)| server != &name)
            });
            tools.extend(discovered);
        }
        *self
            .descriptors
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = next_descriptors;
        crate::uar::telemetry::metrics::set_mcp_server_status(&name, true);
        Ok(())
    }

    /// Remove one MCP server and every tool it contributed.
    pub fn remove_server(&self, name: &str) -> bool {
        if self.bound_services.is_some() {
            return false;
        }
        let Ok(_admission) = self.admission.enter() else {
            return false;
        };
        let removed = {
            let mut services = self
                .services
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match services.remove(name) {
                Some(slot) => {
                    // Transfer ownership before releasing the index lock, so
                    // shutdown must see either the live map entry or this queue.
                    self.admission.retire_slot(slot);
                    true
                }
                None => false,
            }
        };
        self.server_config
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name);
        let removed_names = {
            let mut index = self
                .tool_index
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let names = index
                .iter()
                .filter(|(_, (server, _))| server == name)
                .map(|(tool, _)| tool.clone())
                .collect::<std::collections::HashSet<_>>();
            index.retain(|_, (server, _)| server != name);
            names
        };
        self.tools
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(tool, _)| !removed_names.contains(tool));
        self.descriptors
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|_, descriptor| descriptor.server.as_deref() != Some(name));
        crate::uar::telemetry::metrics::set_mcp_server_status(name, false);
        removed
    }

    /// Cancel every connected MCP transport and wait for its cleanup to finish.
    pub(crate) async fn shutdown(&self) {
        self.begin_shutdown();
        // A borrower closes its own view, never its parent's shared transport.
        if let Some(bindings) = &self.bound_services {
            bindings.closed.cancel();
            return;
        }
        self.shutting_down.store(true, Ordering::Release);
        let services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(name, slot)| {
                let service = begin_service_shutdown(slot);
                (name.clone(), Arc::clone(slot), service)
            })
            .collect::<Vec<_>>();

        // Initiate every close before awaiting any one transport. A slow MCP
        // server must not delay the start of cleanup for its peers.
        for (_, _, service) in &services {
            service.cancellation_token().cancel();
        }

        for (name, slot, service) in services {
            while !service_shutdown_complete(&slot, &service) {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            crate::uar::telemetry::metrics::set_mcp_server_status(&name, false);
            tracing::info!(server = %name, "MCP transport shut down");
        }
        self.admission.join().await;
    }

    /// Begin revocation without awaiting, including from an owning drop guard.
    /// `shutdown` must still be awaited to prove transport closure.
    pub(crate) fn begin_shutdown(&self) {
        if let Some(bindings) = &self.bound_services {
            bindings.closed.cancel();
            return;
        }
        let services = self
            .services
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.shutting_down.store(true, Ordering::Release);
        self.admission.close();
        for slot in services.values() {
            begin_service_shutdown(slot).cancellation_token().cancel();
        }
    }

    /// Merge another registry into this one, returning a deduplicated registry.
    /// This is used to combine global tools with skill-specific tools.
    ///
    /// # Errors
    ///
    /// Returns a collision for incompatible descriptors, or rejects composition
    /// involving a frozen connection view. Delegation only supports filtering.
    pub fn merge(&self, other: &McpRegistry) -> Result<Self, McpMergeError> {
        if self.bound_services.is_some() || other.bound_services.is_some() {
            return Err(McpMergeError::FrozenBinding);
        }
        let mut descriptors = self.descriptor_map();
        for descriptor in other.descriptors() {
            if let Some(existing) = descriptors.get(&descriptor.provider_name) {
                if !existing.equivalent_to(&descriptor) {
                    return Err(ToolCollision {
                        provider_name: descriptor.provider_name.clone(),
                    }
                    .into());
                }
            } else {
                descriptors.insert(descriptor.provider_name.clone(), descriptor);
            }
        }

        let mut services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        services.extend(
            other
                .services
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );

        let mut server_config = self.server_entries();
        server_config.extend(other.server_entries());

        let mut tool_index = self
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for (name, target) in other
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
        {
            tool_index
                .entry(name.clone())
                .or_insert_with(|| target.clone());
        }

        let mut tools = self.tools().into_iter().collect::<BTreeMap<_, _>>();
        for (name, tool) in other.tools() {
            tools.entry(name).or_insert(tool);
        }
        let tools = tools.into_iter().collect();

        let mut native_tools = (*self.native_tools).clone();
        for (name, tool) in other.native_tools.iter() {
            native_tools
                .entry(name.clone())
                .or_insert_with(|| Arc::clone(tool));
        }

        Ok(Self {
            services: Arc::new(RwLock::new(services)),
            bound_services: None,
            shutting_down: Arc::new(AtomicBool::new(
                self.shutting_down.load(Ordering::Acquire)
                    || other.shutting_down.load(Ordering::Acquire),
            )),
            admission: Arc::new(RegistryAdmission::default()),
            server_config: Arc::new(RwLock::new(server_config)),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(tools)),
            descriptors: Arc::new(RwLock::new(descriptors)),
            validator_compiler: Arc::clone(&self.validator_compiler),
            native_tools: Arc::new(native_tools),
        })
    }

    /// Return a registry narrowed to the server and tool sets allowed by the
    /// resolved run policy.
    ///
    /// `None` means "all currently registered values" while `Some(empty)`
    /// means "none". Native tools are governed by `allowed_tools` because they
    /// do not belong to an MCP server.
    #[must_use]
    pub fn filtered(
        &self,
        allowed_servers: Option<&std::collections::HashSet<String>>,
        allowed_tools: Option<&std::collections::HashSet<String>>,
    ) -> Self {
        let server_allowed =
            |server: &str| allowed_servers.is_none_or(|allowed| allowed.contains(server));
        let tool_allowed = |tool: &str| allowed_tools.is_none_or(|allowed| allowed.contains(tool));

        let services = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|(name, _)| server_allowed(name))
            .map(|(name, service)| (name.clone(), Arc::clone(service)))
            .collect();
        let server_config = self
            .server_entries()
            .into_iter()
            .filter(|(name, _)| server_allowed(name))
            .map(|(name, config)| (name.clone(), config.clone()))
            .collect();
        let tool_index = self
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|(name, (server, _))| server_allowed(server) && tool_allowed(name))
            .map(|(name, target)| (name.clone(), target.clone()))
            .collect::<HashMap<_, _>>();
        let native_tools = self
            .native_tools
            .iter()
            .filter(|(name, _)| tool_allowed(name))
            .map(|(name, tool)| (name.clone(), Arc::clone(tool)))
            .collect::<HashMap<_, _>>();
        let tools: Vec<(String, Tool)> = self
            .tools()
            .into_iter()
            .filter(|(name, _)| {
                tool_allowed(name)
                    && (native_tools.contains_key(name) || tool_index.contains_key(name))
            })
            .collect();
        let retained_names = tools
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<std::collections::HashSet<_>>();
        let descriptors = self
            .descriptors()
            .into_iter()
            .filter(|descriptor| retained_names.contains(&descriptor.provider_name))
            .map(|descriptor| (descriptor.provider_name.clone(), descriptor))
            .collect();

        Self {
            services: Arc::new(RwLock::new(services)),
            bound_services: self.bound_services.as_ref().map(|bindings| {
                Arc::new(BoundServices {
                    connections: bindings
                        .connections
                        .iter()
                        .filter(|(name, _)| server_allowed(name))
                        .map(|(name, service)| (name.clone(), Arc::clone(service)))
                        .collect(),
                    closed: bindings.closed.child_token(),
                })
            }),
            shutting_down: Arc::clone(&self.shutting_down),
            admission: Arc::clone(&self.admission),
            server_config: Arc::new(RwLock::new(server_config)),
            tool_index: Arc::new(RwLock::new(tool_index)),
            tools: Arc::new(RwLock::new(tools)),
            descriptors: Arc::new(RwLock::new(descriptors)),
            validator_compiler: Arc::clone(&self.validator_compiler),
            native_tools: Arc::new(native_tools),
        }
    }

    /// Add an in-process runtime tool and its compiled descriptor.
    ///
    /// # Errors
    ///
    /// Returns an assembly error for an invalid schema or a provider-name
    /// collision.
    pub fn with_native_tool(self, tool: Arc<dyn NativeTool>) -> Result<Self, ToolAssemblyError> {
        if self.bound_services.is_some() {
            return Err(ToolAssemblyError::FrozenBinding);
        }
        let ns_name = Self::sanitize_tool_name(&format!("native__{}", tool.name()));
        let descriptor = native_tool_descriptor(&self.validator_compiler, &ns_name, tool.as_ref())?;
        let mut descriptors = self.descriptor_map();
        insert_descriptor(&mut descriptors, descriptor)?;

        let mut tools = self.tools();
        let mcp_tool = Tool::new(
            tool.name().to_string(),
            tool.description().to_string(),
            tool.schema()
                .as_object()
                .unwrap_or(&serde_json::Map::new())
                .clone(),
        );
        tools.push((ns_name.clone(), mcp_tool));

        let mut native_tools = (*self.native_tools).clone();
        native_tools.insert(ns_name, tool);

        Ok(Self {
            services: self.services, // Keep ref
            bound_services: self.bound_services,
            shutting_down: self.shutting_down,
            admission: self.admission,
            server_config: self.server_config,
            tool_index: self.tool_index, // Keep ref
            tools: Arc::new(RwLock::new(tools)),
            descriptors: Arc::new(RwLock::new(descriptors)),
            validator_compiler: self.validator_compiler,
            native_tools: Arc::new(native_tools),
        })
    }

    pub fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        self.descriptors()
            .into_iter()
            .filter(|descriptor| descriptor.exposure == Exposure::Eager)
            .map(|descriptor| descriptor.openai_tool_json())
            .collect()
    }

    /// Execute a namespaced tool, e.g. "`time__now`" or "`tavily__search`".
    #[tracing::instrument(
        name = "tool.call",
        skip(self, arguments),
        fields(tool = %namespaced_tool),
    )]
    pub async fn call_namespaced_tool(
        &self,
        namespaced_tool: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        if self.bound_services.as_ref().is_some_and(|bindings| {
            bindings.closed.is_cancelled() || self.shutting_down.load(Ordering::Acquire)
        }) {
            anyhow::bail!("Frozen MCP view is closed");
        }
        if namespaced_tool == "mirror" && self.bound_services.is_none() {
            return Ok(arguments);
        }

        if let Some(tool) = self.native_tools.get(namespaced_tool) {
            // Native tools were the one execution path with no time bound;
            // MCP tools have had one (30s, below) all along. A native tool
            // that never returns held the whole run open with no way to
            // recover, which is the same class of failure as a hung MCP
            // handshake — just later in the turn.
            return match tokio::time::timeout(MCP_TOOL_CALL_TIMEOUT, tool.call(arguments)).await {
                Ok(result) => result,
                Err(_elapsed) => Err(anyhow!(
                    "native tool {namespaced_tool} timed out after {}s",
                    MCP_TOOL_CALL_TIMEOUT.as_secs()
                )),
            };
        }

        // 1. Lookup server + raw_tool_name
        let (server_name, raw_tool_name) = self
            .tool_index
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(namespaced_tool)
            .ok_or_else(|| anyhow!("unknown tool: {namespaced_tool}"))?
            .clone();

        if server_name == "test" && self.bound_services.is_none() {
            return Ok(serde_json::json!({
                "result": format!("executed test tool {} with args {:?}", raw_tool_name, arguments)
            }));
        }

        // 2. Lookup service
        let service_slot = self
            .services
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&server_name)
            .cloned()
            .ok_or_else(|| anyhow!("missing server handle: {server_name}"))?;
        let service = match &self.bound_services {
            Some(bindings) => {
                let bound = bindings
                    .connections
                    .get(&server_name)
                    .ok_or_else(|| anyhow!("Missing frozen MCP binding: {server_name}"))?;
                validate_bound_service(&service_slot, bound)?;
                Arc::clone(bound)
            }
            None => current_service(&service_slot),
        };

        // 3. Call tool
        let args_obj = arguments.as_object().cloned();
        let input_size = serde_json::to_string(&arguments)
            .map(|s| s.len())
            .unwrap_or(0);
        let start = std::time::Instant::now();
        // rmcp 1.8: CallToolRequestParams is #[non_exhaustive] -- use the
        // provided new()/with_arguments() builder instead of a struct literal.
        let mut call_params = CallToolRequestParams::new(raw_tool_name.clone());
        if let Some(args) = args_obj {
            call_params = call_params.with_arguments(args);
        }
        let res = tokio::time::timeout(MCP_TOOL_CALL_TIMEOUT, service.call_tool(call_params))
            .await
            .map_err(|_| {
                anyhow!("tools/call timed out after 30 seconds for {server_name}::{raw_tool_name}")
            })
            .and_then(|result| result.map_err(anyhow::Error::from));
        let duration = start.elapsed();
        let success = res.is_ok();

        let output_size = if let Ok(ref r) = res {
            serde_json::to_string(r).map(|s| s.len()).unwrap_or(0)
        } else {
            0
        };

        tracing::info!(
            target: "mcp.tool.execution",
            tool = %namespaced_tool,
            duration_ms = %duration.as_millis(),
            input_size_bytes = %input_size,
            output_size_bytes = %output_size,
            success = %success,
            "MCP tool executed"
        );

        // Emit raw metrics via the metrics crate
        #[cfg(feature = "telemetry")]
        metrics::counter!("mcp_tool_calls_total", "tool" => namespaced_tool.to_string(), "success" => success.to_string()).increment(1);
        #[allow(clippy::cast_precision_loss)]
        #[cfg(feature = "telemetry")]
        metrics::histogram!("mcp_tool_duration_ms", "tool" => namespaced_tool.to_string())
            .record(duration.as_millis() as f64);

        // Record normalized tool call metrics via telemetry module
        crate::uar::telemetry::metrics::record_tool_call(namespaced_tool, success);

        let res = match res {
            Ok(result) => result,
            Err(error) => {
                {
                    let state = service_slot
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    // Projected bindings record only through their ordered publisher.
                    if state.lifecycle.is_none() {
                        crate::uar::telemetry::metrics::set_mcp_server_status(&server_name, false);
                    }
                }
                // A child cannot re-read auth/env/config after transport loss.
                // The owner may establish a new binding for a later root turn.
                if self.bound_services.is_some() {
                    return Err(error);
                }
                // Re-establish the transport for future calls. Never replay the
                // failed call: it may have completed remotely before transport loss.
                if let Some(attempt) = begin_reconnect(&service_slot) {
                    let reconnect = async {
                        if let Some(snapshot) = &attempt.snapshot {
                            reconnect_snapshot(snapshot, &self.validator_compiler).await
                        } else {
                            tokio::time::timeout(
                                MCP_CONNECT_TIMEOUT,
                                connect_server(&server_name, &attempt.entry),
                            )
                            .await
                            .context("MCP reconnect handshake timed out")?
                        }
                    };
                    match reconnect.await {
                        Ok(replacement) => {
                            let installed = install_reconnected_service(
                                &service_slot,
                                attempt.generation,
                                replacement,
                            );
                            // The slot retains replaced and rejected services,
                            // so cancellation here leaves shutdown a joinable
                            // cleanup record instead of a detached transport.
                            reap_retired_services(&service_slot).await;
                            if installed {
                                {
                                    let state = service_slot
                                        .read()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                                    if state.lifecycle.is_none()
                                        && !state.shutting_down
                                        && state.generation == attempt.generation
                                    {
                                        crate::uar::telemetry::metrics::set_mcp_server_status(
                                            &server_name,
                                            true,
                                        );
                                    }
                                }
                                tracing::info!(server = %server_name, "MCP transport reconnected for subsequent calls");
                            } else {
                                tracing::info!(server = %server_name, "Discarded MCP reconnect because shutdown began or a newer server configuration was installed");
                            }
                        }
                        Err(reconnect_error) => {
                            record_reconnect_failure(
                                &service_slot,
                                attempt.generation,
                                &reconnect_error,
                            );
                            tracing::warn!(
                                server = %server_name,
                                error = %reconnect_error,
                                "MCP transport reconnect failed; a later call may retry"
                            );
                        }
                    }
                }
                return Err(error).with_context(|| {
                    format!("tools/call failed for {server_name}::{raw_tool_name}")
                });
            }
        };

        // 4. Return content (simplified)
        Ok(serde_json::to_value(res)?)
    }
}

fn resolve_mcp_config_path(path: &str) -> PathBuf {
    if let Ok(env_path) = std::env::var("MCP_CONFIG_PATH") {
        let candidate = PathBuf::from(env_path);
        if candidate.exists() {
            return candidate;
        }
    }

    let path_buf = PathBuf::from(path);
    if path_buf.is_relative()
        && let Ok(dir) = std::env::var("MCP_CONFIG_DIR")
    {
        let candidate = PathBuf::from(dir).join(&path_buf);
        if candidate.exists() {
            return candidate;
        }
    }

    path_buf
}

fn resolve_mcp_command(command: &str) -> PathBuf {
    let path = Path::new(command);
    if path.is_absolute() {
        return path.to_path_buf();
    }

    if let Ok(dir) = std::env::var("MCP_SERVER_DIR") {
        let candidate = PathBuf::from(dir).join(path);
        if candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::config::McpConfig;
    use serde::Deserialize;
    use std::collections::HashSet;

    const FIXTURE: &str = r#"
import json
import os
import sys

trace_path = sys.argv[1]

for line in sys.stdin:
    try:
        request = json.loads(line)
    except json.JSONDecodeError:
        continue
    request_id = request.get("id")
    if request_id is None:
        continue
    method = request.get("method")
    if method == "initialize":
        result = {
            "protocolVersion": request.get("params", {}).get("protocolVersion", "2024-11-05"),
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "uar-registry-test", "version": "1.0.0"},
        }
    elif method == "tools/list":
        result = {"tools": [{
            "name": "echo",
            "description": "MCP registry recovery fixture",
            "inputSchema": {
                "type": "object",
                "properties": {"mode": {"type": "string"}},
                "required": ["mode"],
            },
        }]}
    elif method == "tools/call":
        mode = request.get("params", {}).get("arguments", {}).get("mode", "echo")
        with open(trace_path, "a", encoding="utf-8") as trace:
            trace.write(json.dumps({"pid": os.getpid(), "mode": mode}) + "\n")
            trace.flush()
            os.fsync(trace.fileno())
        if mode == "crash":
            sys.exit(23)
        result = {"content": [{"type": "text", "text": f"mcp-{mode}"}], "isError": False}
    else:
        print(json.dumps({
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {"code": -32601, "message": "method not found"},
        }), flush=True)
        continue
    print(json.dumps({"jsonrpc": "2.0", "id": request_id, "result": result}), flush=True)

with open(trace_path, "a", encoding="utf-8") as trace:
    trace.write(json.dumps({"pid": os.getpid(), "mode": "stdin_closed"}) + "\n")
    trace.flush()
    os.fsync(trace.fileno())
"#;

    #[derive(Deserialize)]
    struct TraceEntry {
        pid: u32,
        mode: String,
    }

    fn fixture_config() -> (tempfile::TempDir, PathBuf, McpConfig) {
        let fixture_dir = tempfile::tempdir().expect("create MCP fixture directory");
        let script = fixture_dir.path().join("mock-mcp.py");
        let trace = fixture_dir.path().join("trace.jsonl");
        std::fs::write(&script, FIXTURE).expect("write MCP fixture");
        let config = McpConfig {
            mcp_servers: HashMap::from([(
                "resilience".to_string(),
                McpServerEntry::Stdio {
                    command: "python3".to_string(),
                    args: vec![script.display().to_string(), trace.display().to_string()],
                    env: HashMap::new(),
                    sandboxed: false,
                },
            )]),
        };
        (fixture_dir, trace, config)
    }

    fn trace_entries(path: &Path) -> Vec<TraceEntry> {
        std::fs::read_to_string(path)
            .expect("read MCP trace")
            .lines()
            .map(|line| serde_json::from_str(line).expect("parse MCP trace entry"))
            .collect()
    }

    #[tokio::test]
    async fn reconnect_replacement_is_shared_without_widening_filtered_views() {
        let (_fixture_dir, trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        let allowed_servers = HashSet::from(["resilience".to_string()]);
        let allowed_tools = HashSet::from(["resilience__echo".to_string()]);
        let first_view = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));
        let second_view = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));
        let merged_view = McpRegistry::empty()
            .merge(&registry)
            .expect("descriptors do not collide");
        let denied_server_view = registry.filtered(Some(&HashSet::new()), None);
        let denied_tool_view = registry.filtered(Some(&allowed_servers), Some(&HashSet::new()));

        first_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "crash"}))
            .await
            .expect_err("crashed MCP call must fail without replay");

        second_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "echo"}))
            .await
            .expect("independent filtered view must use replacement service");
        merged_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "echo"}))
            .await
            .expect("pre-existing merged view must use replacement service");

        assert!(denied_server_view.server_names().is_empty());
        assert!(denied_server_view.tools().is_empty());
        assert!(denied_tool_view.tools().is_empty());
        denied_tool_view
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "echo"}))
            .await
            .expect_err("replacement must not restore a policy-excluded tool");

        registry.shutdown().await;

        let trace = trace_entries(&trace);
        assert_eq!(
            trace.iter().filter(|entry| entry.mode == "crash").count(),
            1,
            "failed call must execute exactly once"
        );
        let calls = trace
            .iter()
            .filter(|entry| entry.mode != "stdin_closed")
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 3);
        assert_ne!(calls[0].pid, calls[1].pid);
        assert_eq!(calls[1].pid, calls[2].pid);
    }

    #[tokio::test]
    async fn upsert_reconnect_uses_new_config_in_an_existing_filtered_view() {
        let (fixture_dir, original_trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        let allowed_servers = HashSet::from(["resilience".to_string()]);
        let allowed_tools = HashSet::from(["resilience__echo".to_string()]);
        let filtered = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));

        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "before"}))
            .await
            .expect("initial service must respond");
        let replacement_trace = fixture_dir.path().join("replacement-trace.jsonl");
        let replacement_entry = match config
            .mcp_servers
            .get("resilience")
            .expect("fixture server config")
            .clone()
        {
            McpServerEntry::Stdio {
                command,
                mut args,
                env,
                sandboxed,
            } => {
                *args.last_mut().expect("fixture trace argument") =
                    replacement_trace.display().to_string();
                McpServerEntry::Stdio {
                    command,
                    args,
                    env,
                    sandboxed,
                }
            }
            McpServerEntry::RemoteHttp { .. } => panic!("fixture must use stdio"),
        };
        registry
            .upsert_server("resilience".to_string(), replacement_entry)
            .await
            .expect("upsert replacement service");
        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "crash"}))
            .await
            .expect_err("post-upsert crash must fail without replay");
        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "after"}))
            .await
            .expect("existing filtered view must reconnect with the upserted config");

        registry.shutdown().await;

        let original_trace = trace_entries(&original_trace);
        let original_calls = original_trace
            .iter()
            .filter(|entry| entry.mode != "stdin_closed")
            .collect::<Vec<_>>();
        assert_eq!(original_calls.len(), 1);
        assert_eq!(original_calls[0].mode, "before");

        let replacement_trace = trace_entries(&replacement_trace);
        let replacement_calls = replacement_trace
            .iter()
            .filter(|entry| entry.mode != "stdin_closed")
            .collect::<Vec<_>>();
        assert_eq!(replacement_calls.len(), 2);
        assert_eq!(replacement_calls[0].mode, "crash");
        assert_eq!(replacement_calls[1].mode, "after");
        assert_ne!(replacement_calls[0].pid, replacement_calls[1].pid);
    }

    #[tokio::test]
    async fn shutdown_waits_for_stdio_eof_and_blocks_filtered_view_reconnect() {
        let (_fixture_dir, trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        let allowed_servers = HashSet::from(["resilience".to_string()]);
        let allowed_tools = HashSet::from(["resilience__echo".to_string()]);
        let filtered = registry.filtered(Some(&allowed_servers), Some(&allowed_tools));

        registry.shutdown().await;

        filtered
            .call_namespaced_tool("resilience__echo", serde_json::json!({"mode": "after"}))
            .await
            .expect_err("a pre-existing filtered view must not reconnect after shutdown");
        let trace = trace_entries(&trace);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].mode, "stdin_closed");
    }

    #[tokio::test]
    async fn shutdown_blocks_new_server_upsert() {
        let (_fixture_dir, trace, config) = fixture_config();
        let registry = McpRegistry::from_config(&config)
            .await
            .expect("connect MCP fixture");
        registry.shutdown().await;

        registry
            .upsert_server(
                "replacement".to_string(),
                config.mcp_servers["resilience"].clone(),
            )
            .await
            .expect_err("shutdown must reject a newly configured MCP transport");

        let trace = trace_entries(&trace);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].mode, "stdin_closed");
    }
}
