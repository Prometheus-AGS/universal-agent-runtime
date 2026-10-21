//! Run-owned human input for MCP. The host remains the only tool authority.
use std::{collections::HashMap, sync::{Arc, Mutex, OnceLock, Weak}, time::Duration};
use chrono::Utc;
use rmcp::{ClientHandler, model::{ClientConfig, ElicitRequestParams, ElicitResult, ElicitationAction,
    ElicitationCapability, FormElicitationCapability, UrlElicitationCapability, ErrorData},
    service::{RequestContext, RoleClient}};
use tokio_util::sync::CancellationToken;
use crate::uar::{persistence::{PersistenceLayer, mcp_elicitations::{McpElicitationRecord, ElicitationState}},
    runtime::actor::messages::ActorOwner};

// Availability only. Persisted records own every decision; this map prevents a
// restarted host or different connection from consuming an old pending request.
static CHANNELS: OnceLock<Mutex<HashMap<String, Weak<McpInteractionChannel>>>> = OnceLock::new();

#[derive(Debug)]
pub struct McpInteractionChannel {
    id: String,
    store: Arc<dyn PersistenceLayer>,
    owner_key: String,
    run_id: String,
    session_id: String,
    workspace_id: Option<String>,
    cancellation: CancellationToken,
    pending: Mutex<HashMap<String, PendingAvailability>>,
    activity: tokio::sync::watch::Sender<usize>,
    durable_changes: tokio::sync::watch::Sender<u64>,
    scope: crate::uar::runtime::lifecycle::ResourceScope,
}
impl McpInteractionChannel {
    /// A host may install this only where authenticated interaction routes exist.
    /// Durable storage and a verified full principal are required capabilities.
    pub fn new(store: Arc<dyn PersistenceLayer>, owner: &ActorOwner, run_id: String,
        session_id: String, workspace_id: Option<String>, cancellation: CancellationToken,
        scope: crate::uar::runtime::lifecycle::ResourceScope) -> anyhow::Result<Arc<Self>> {
        anyhow::ensure!(store.supports_durable_mcp_elicitations(), "durable_elicitation_unsupported");
        let owner_key = owner.credential_principal().ok_or_else(||anyhow::anyhow!("verified_principal_required"))?.storage_key();
        let (durable_changes, _) = tokio::sync::watch::channel(0);
        let channel = Arc::new(Self { id: uuid::Uuid::new_v4().to_string(), store, owner_key,
            run_id, session_id, workspace_id, cancellation:cancellation.child_token(), pending: Mutex::new(HashMap::new()), activity: tokio::sync::watch::channel(0).0,
            durable_changes, scope });
        let mut channels = CHANNELS.get_or_init(Default::default).lock().map_err(|_|anyhow::anyhow!("interaction_channel_unavailable"))?;
        channels.retain(|_, channel| channel.strong_count() > 0);
        channels.insert(channel.id.clone(), Arc::downgrade(&channel));
        Ok(channel)
    }
    pub(crate) fn id(&self) -> &str { &self.id }
    pub(crate) fn close(&self) { self.cancellation.cancel(); }
    pub(crate) fn lifecycle_scope(&self) -> crate::uar::runtime::lifecycle::ResourceScope { self.scope.clone() }

    /// Wake interaction waiters after a committed durable elicitation change.
    /// The generation carries no decision; waiters reread their own exact row.
    pub(crate) fn notify_durable_change() {
        let Some(channels) = CHANNELS.get() else { return; };
        let Ok(channels) = channels.lock() else { return; };
        for channel in channels.values().filter_map(Weak::upgrade) {
            let next = (*channel.durable_changes.borrow()).saturating_add(1);
            channel.durable_changes.send_replace(next);
        }
    }
    pub(crate) async fn settle_closed(&self) -> anyhow::Result<()> {
        self.close();
        let mut activity = self.activity.subscribe();
        loop {
            if *activity.borrow_and_update() == 0 { break; }
            activity.changed().await.map_err(|_|anyhow::anyhow!("interaction_settlement_unavailable"))?;
        }
        for row in self.store.list_mcp_elicitations(&self.owner_key,&self.run_id).await?
            .into_iter().filter(|row|row.channel_id == self.id && row.state == ElicitationState::Pending) {
            let next = row.finish(ElicitationState::Cancelled);
            let _ = self.store.compare_and_swap_mcp_elicitation(&row,&next).await?;
        }
        Ok(())
    }
    pub(crate) fn matches_owner(&self, owner: &ActorOwner) -> bool {
        owner.credential_principal().is_some_and(|principal|principal.storage_key() == self.owner_key)
    }
    pub(crate) fn available(&self) -> bool { !self.cancellation.is_cancelled() }
    pub(crate) fn active_for(row: &McpElicitationRecord) -> bool {
        CHANNELS.get().and_then(|channels|channels.lock().ok())
            .and_then(|channels|channels.get(&row.channel_id).and_then(Weak::upgrade))
            .is_some_and(|channel| channel.available() && channel.owner_key == row.owner_key
                && channel.run_id == row.run_id && channel.session_id == row.session_id
                && channel.workspace_id == row.workspace_id
                && channel.pending.lock().is_ok_and(|pending|pending.get(&row.id).is_some_and(|waiter|waiter.available(&row.server_name))))
    }

    async fn wait(self: Arc<Self>, server: String, connection: String, request: ElicitRequestParams,
        context: RequestContext<RoleClient>, abandoned: CancellationToken,
        credential: Option<super::runtime::RunMcpCredentialLease>,
        binding: Option<Arc<super::binding_cache::McpBindingRequest>>) -> anyhow::Result<ElicitResult> {
        anyhow::ensure!(self.available(), "interaction_channel_closed");
        if let Some(binding) = &binding { binding.authorize_current_definition()?; }
        if let Some(credential) = &credential { credential.authorize_current(&server).await?; }
        // Validate the supported request before persistence. URLs are presented
        // as untrusted links; neither the host nor an agent opens them.
        match &request {
            ElicitRequestParams::FormElicitationParams { requested_schema, .. } => {
                jsonschema::options().should_validate_formats(true).build(&serde_json::to_value(requested_schema)?)
                    .map_err(|_|anyhow::anyhow!("invalid_elicitation_schema"))?;
            }
            ElicitRequestParams::UrlElicitationParams { url, .. } => {
                let parsed = url::Url::parse(url)?;
                anyhow::ensure!(matches!(parsed.scheme(), "https" | "http") && parsed.username().is_empty()
                    && parsed.password().is_none(), "invalid_elicitation_url");
            }
            _ => anyhow::bail!("unsupported_elicitation_mode"),
        }
        let now = Utc::now();
        let row = McpElicitationRecord { id:format!("mcp-{}",uuid::Uuid::new_v4()),
            owner_key:self.owner_key.clone(), run_id:self.run_id.clone(), session_id:self.session_id.clone(),
            workspace_id:self.workspace_id.clone(), channel_id:self.id.clone(), server_name:server,
            connection_id:connection, request_id:context.id.to_string(), request, created_at:now,
            expires_at:now + chrono::Duration::minutes(10), state:ElicitationState::Pending, response:None };
        {
            let mut pending = self.pending.lock().map_err(|_|anyhow::anyhow!("interaction_channel_unavailable"))?;
            pending.insert(row.id.clone(), PendingAvailability {credential:credential.clone(),
                binding:binding.clone(),
                cancellation:context.ct.clone(), abandoned:abandoned.clone(), peer:context.peer.clone()});
            self.activity.send_replace(pending.len());
        }
        let _pending = PendingRequest {channel:Arc::clone(&self),id:row.id.clone()};
        self.store.create_mcp_elicitation(&row).await?;
        let mut durable_changes = self.durable_changes.subscribe();
        loop {
            let credential_valid = match &credential {
                Some(lease) => lease.authorize_current(&row.server_name).await.is_ok(),
                None => true,
            } && binding.as_ref().is_none_or(|binding| binding.authorize_current_definition().is_ok());
            let current = self.store.list_mcp_elicitations(&self.owner_key,&self.run_id).await?
                .into_iter().find(|saved|saved.id == row.id).ok_or_else(||anyhow::anyhow!("elicitation_record_missing"))?;
            if current.state != ElicitationState::Pending {
                // A durable answer is consumed by its original handler only.
                if !self.available() || context.ct.is_cancelled() || abandoned.is_cancelled()
                    || context.peer.is_transport_closed() || !credential_valid {
                    return Ok(ElicitResult::new(ElicitationAction::Cancel));
                }
                return Ok(current.response.unwrap_or_else(||ElicitResult::new(ElicitationAction::Cancel)));
            }
            let disposition = if self.cancellation.is_cancelled() || context.ct.is_cancelled()
                || !credential_valid {
                Some(ElicitationState::Cancelled)
            } else if abandoned.is_cancelled() || context.peer.is_transport_closed() {
                Some(ElicitationState::Disconnected)
            } else if Utc::now() >= row.expires_at { Some(ElicitationState::Expired) } else { None };
            if let Some(disposition) = disposition {
                let next = current.finish(disposition);
                if self.store.compare_and_swap_mcp_elicitation(&current,&next).await? {
                    return Ok(ElicitResult::new(ElicitationAction::Cancel));
                }
                continue;
            }
            let expiry = row.expires_at
                .signed_duration_since(Utc::now())
                .to_std()
                .unwrap_or_default();
            tokio::select! {
                _ = self.cancellation.cancelled() => {},
                _ = context.ct.cancelled() => {},
                _ = abandoned.cancelled() => {},
                changed = durable_changes.changed() => {
                    if changed.is_err() {
                        return Err(anyhow::anyhow!("elicitation durable-change source closed"));
                    }
                },
                // This is the actual persisted expiry due time. It requests a
                // terminal cancellation transition; it never certifies any
                // other resource's completion.
                _ = tokio::time::sleep(expiry) => {},
            }
        }
    }
}

#[derive(Debug)]
struct PendingAvailability {
    credential: Option<super::runtime::RunMcpCredentialLease>,
    binding: Option<Arc<super::binding_cache::McpBindingRequest>>,
    cancellation: CancellationToken,
    abandoned: CancellationToken,
    peer: rmcp::service::Peer<RoleClient>,
}
impl PendingAvailability {
    fn available(&self, server: &str) -> bool {
        !self.cancellation.is_cancelled() && !self.abandoned.is_cancelled() && !self.peer.is_transport_closed()
            && self.credential.as_ref().is_none_or(|lease|lease.authorize(server).is_ok())
            && self.binding.as_ref().is_none_or(|binding|binding.authorize_current_definition().is_ok())
    }
}

struct PendingRequest { channel: Arc<McpInteractionChannel>, id: String }
impl Drop for PendingRequest {
    fn drop(&mut self) {
        if let Ok(mut pending) = self.channel.pending.lock() {
            pending.remove(&self.id);
            self.channel.activity.send_replace(pending.len());
        }
    }
}

/// Installed on every client construction path; ambient clients have no channel.
#[derive(Debug, Clone, Default)]
pub(crate) struct RunMcpClient {
    channel: Option<Arc<McpInteractionChannel>>,
    server: String,
    connection: String,
    credential: Option<super::runtime::RunMcpCredentialLease>,
    binding: Option<Arc<super::binding_cache::McpBindingRequest>>,
}
impl RunMcpClient {
    pub(crate) fn with_credential(mut self, credential: Option<super::runtime::RunMcpCredentialLease>) -> Self {
        self.credential = credential; self
    }
    pub(crate) fn channel(&self) -> Option<&Arc<McpInteractionChannel>> { self.channel.as_ref() }
    pub(crate) fn for_request(request: &super::binding_cache::McpBindingRequest) -> Self {
        Self { channel:request.interactions().cloned(), server:request.definition().name().to_owned(),
            connection:uuid::Uuid::new_v4().to_string(), credential:None,
            binding:Some(Arc::new(request.clone())) }
    }
}
impl ClientHandler for RunMcpClient {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default().with_protocol_version(rmcp::model::ProtocolVersion::V_2025_11_25);
        if self.channel.as_ref().is_some_and(|channel|channel.available()) {
            info.capabilities.elicitation = Some(ElicitationCapability::new()
                .with_form(FormElicitationCapability::new().with_schema_validation(true))
                .with_url(UrlElicitationCapability::new()));
        }
        info
    }
    async fn create_elicitation(&self, request: ElicitRequestParams, context: RequestContext<RoleClient>) -> Result<ElicitResult, ErrorData> {
        let Some(channel) = self.channel.as_ref().filter(|channel|channel.available()).cloned() else {
            return Err(ErrorData::invalid_request("Run interaction channel is unavailable",None));
        };
        let version = context.peer.peer_info().map(|info|info.protocol_version.clone());
        let supported = match &request {
            ElicitRequestParams::FormElicitationParams { .. } => version.as_ref().is_some_and(|version|version >= &rmcp::model::ProtocolVersion::V_2025_06_18),
            ElicitRequestParams::UrlElicitationParams { .. } => version.as_ref().is_some_and(|version|version >= &rmcp::model::ProtocolVersion::V_2025_11_25),
            _ => false,
        };
        if !supported { return Err(ErrorData::invalid_request("Elicitation mode was not negotiated",None)); }
        let abandoned = CancellationToken::new();
        let _guard = abandoned.clone().drop_guard();
        let server = self.server.clone(); let connection = self.connection.clone();
        let credential = self.credential.clone();
        let binding = self.binding.clone();
        // rmcp may drop the callback on disconnect. The trusted run scope owns
        // settlement, so dropping that callback cannot detach the durable
        // cancellation attempt from host shutdown accounting.
        let scope = channel.scope.clone();
        let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
        let cancellation = scope.cancellation_token();
        if scope.spawn("mcp-elicitation-settlement", async move {
            let result = tokio::select! {
                _ = cancellation.cancelled() => Ok(ElicitResult::new(ElicitationAction::Cancel)),
                result = channel.wait(server,connection,request,context,abandoned,credential,binding) => result,
            };
            let _ = result_sender.send(result);
            Ok(())
        }).is_err() {
            return Err(ErrorData::invalid_request("Run interaction channel is unavailable",None));
        }
        result_receiver
            .await.map_err(|_|ErrorData::internal_error("Elicitation settlement stopped",None))?
            .map_err(|_|ErrorData::internal_error("Elicitation persistence unavailable",None))
    }
}

/// Observe an explicit execution budget without owning the accepted operation.
/// Expiry only asks the caller to cancel and join its selected connection. Human
/// input pauses this budget; durable elicitation expiry has its own contract.
pub(crate) async fn await_interaction_budget<F: std::future::Future>(channel: Option<&Arc<McpInteractionChannel>>,
    budget: Duration, mut future: std::pin::Pin<&mut F>) -> Result<F::Output, ()> {
    let Some(channel) = channel else { return tokio::time::timeout(budget, future.as_mut()).await.map_err(|_|()); };
    let mut activity = channel.activity.subscribe();
    let mut pending = *activity.borrow_and_update() > 0;
    let mut remaining = budget;
    let mut previous = tokio::time::Instant::now();
    loop {
        let now = tokio::time::Instant::now();
        if !pending { remaining = remaining.saturating_sub(now.duration_since(previous)); }
        previous = now;
        tokio::select! {
            result = future.as_mut() => return Ok(result),
            _ = tokio::time::sleep(remaining), if !pending => return Err(()),
            changed = activity.changed() => {
                if changed.is_err() { return Err(()); }
                let now = tokio::time::Instant::now();
                if !pending { remaining = remaining.saturating_sub(now.duration_since(previous)); }
                previous = now;
                pending = *activity.borrow_and_update() > 0;
            }
        }
    }
}
