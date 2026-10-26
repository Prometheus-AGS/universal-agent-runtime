//! Lifecycle admission around a domain persistence capability.
//!
//! Closing this facade drains its accepted operations and leaves a borrowed
//! host provider usable. The supervisor separately owns physical engine close.

use super::*;
use std::sync::Arc;
use crate::uar::runtime::lifecycle::{CloseReport, ResourceScope};

#[derive(Debug)]
pub struct ScopedPersistence {
    inner: Arc<dyn PersistenceLayer>,
    scope: ResourceScope,
}

impl ScopedPersistence {
    pub fn new(inner: Arc<dyn PersistenceLayer>, scope: ResourceScope) -> Self {
        Self { inner, scope }
    }

    pub async fn close(&self) -> CloseReport { self.scope.close().await }
    pub fn scope(&self) -> ResourceScope { self.scope.clone() }

    /// Run accepted persistence work in the scope's retained task tracker.
    /// Losing the caller's response waiter cannot cancel an operation after it
    /// has crossed persistence admission. The one-shot only delivers its
    /// actual result to that caller; scope closure joins the work itself.
    async fn dispatch<T, F, Fut>(&self, method: &'static str, work: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(Arc<dyn PersistenceLayer>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T>> + Send + 'static,
    {
        let operation = uuid::Uuid::new_v4().to_string();
        let worker_scope = self.scope.clone();
        let inner = Arc::clone(&self.inner);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.scope.spawn(format!("persistence:{method}:{operation}"), async move {
            let mut completion = PersistenceCompletion::new(worker_scope, method, &operation);
            let result = work(inner).await;
            completion.observe_result(&result);
            let _ = sender.send(result);
            Ok(())
        })?;
        receiver.await.map_err(|_| anyhow::anyhow!(
            "retained persistence operation '{method}' ended before reporting its outcome"
        ))?
    }
}

/// A retained worker records an uncertain durable outcome when its transport
/// response is lost or it ends before acknowledging a mutating operation.
struct PersistenceCompletion {
    scope: ResourceScope,
    operation: String,
    may_mutate: bool,
    acknowledged: bool,
}

impl PersistenceCompletion {
    fn new(scope: ResourceScope, method: &'static str, identity: &str) -> Self {
        Self { scope, operation: format!("{method}:{identity}"),
            may_mutate: !read_only_operation(method), acknowledged: false }
    }
    fn observe_result<T>(&mut self, result: &Result<T>) {
        if self.may_mutate && result.as_ref().err().is_some_and(transport_outcome_unknown) {
            self.scope.record_unresolved_effect(self.operation.clone());
        }
        self.acknowledged = true;
    }
}

impl Drop for PersistenceCompletion {
    fn drop(&mut self) {
        if !self.acknowledged && self.may_mutate { self.scope.record_unresolved_effect(self.operation.clone()); }
    }
}

fn transport_outcome_unknown(error: &anyhow::Error) -> bool {
    for cause in error.chain() {
        #[cfg(feature = "surreal-backend")]
        if cause.downcast_ref::<surrealdb::types::Error>().is_some_and(|error| error.is_connection()) {
            return true;
        }
        #[cfg(feature = "postgres-backend")]
        if matches!(cause.downcast_ref::<sqlx::Error>(), Some(sqlx::Error::Io(_) | sqlx::Error::Tls(_) | sqlx::Error::Protocol(_))) {
            return true;
        }
    }
    false
}

// The explicit read set prevents cancellation of a snapshot from being reported
// as an uncertain mutation. New operations conservatively retain uncertainty.
fn read_only_operation(method: &str) -> bool {
    matches!(method,
        "load_client_tool_continuation" | "load_client_tool_continuation_for_run" | "load_client_tool_continuation_for_session" |
        "list_mcp_elicitations" | "load_agui_run" | "list_agui_runs" |
        "load_a2a_binding" | "list_a2a_bindings" | "load_run_history_record" |
        "load_run_history_events" | "get_accounting_delegation" | "get_accounting_scope" |
        "list_accounting_calls" | "discover_ingestion" | "next_ingestion_due_at" |
        "get_ingestion_job" | "get_ingestion_source" | "load_channel_subscription" |
        "list_channel_subscriptions" | "load_channel_inbox_entry" | "list_channel_inbox_entries" |
        "load_observer_subscription" | "list_observer_subscriptions" | "list_all_observer_subscriptions" |
        "list_instance_occurrences" | "instance_occurrence_bounds" | "load_agent_instance" |
        "list_agent_instances" | "get_presentation" | "list_presentations" |
        "load_session" | "list_sessions" | "list_expired_sessions" | "next_persisted_session_expiry" |
        "load_conversation_policy" | "load_principal_conversation_policy" |
        "load_user_prompt_caching_settings" | "search_skills" | "list_skills" |
        "get_knowledge_base" | "get_knowledge_base_by_name" | "list_knowledge_bases" |
        "search_knowledge" | "search_knowledge_scoped" | "search_knowledge_scoped_in_space" | "get_document" |
        "list_documents" | "count_documents" | "load_agent_thread" | "list_agent_threads" |
        "list_agent_edges" | "list_canonical_tool_receipts" | "list_approval_records" |
        "list_tool_admission_evidence" | "load_agent" | "load_agent_by_name" |
        "list_agents" | "search_memory" | "list_settings_types" | "get_settings_type" |
        "get_setting" | "list_settings" | "get_attachment" | "list_attachments_for_session" |
        "load_supervisor_domain" | "load_supervisor_challenge" | "load_checkpoint_resume" |
        "list_checkpoint_resumes" | "load_checkpoint" | "list_checkpoints" | "list_cost_history"
    )
}

#[async_trait]
impl PersistenceLayer for ScopedPersistence {
    fn supports_durable_client_tools(&self) -> bool { self.inner.supports_durable_client_tools() }

    async fn create_client_tool_continuation(&self, row: &client_tools::ClientToolContinuation) -> Result<bool> {
        let row = row.clone();
        self.dispatch("create_client_tool_continuation", move |inner| async move {
            inner.create_client_tool_continuation(&row).await
        }).await
    }

    async fn load_client_tool_continuation(&self, owner: &str, tenant: Option<&str>, agent: &str, batch: &str) -> Result<Option<client_tools::ClientToolContinuation>> {
        let (owner, tenant, agent, batch) = (owner.to_owned(), tenant.map(str::to_owned), agent.to_owned(), batch.to_owned());
        self.dispatch("load_client_tool_continuation", move |inner| async move {
            inner.load_client_tool_continuation(&owner, tenant.as_deref(), &agent, &batch).await
        }).await
    }

    async fn load_client_tool_continuation_for_run(&self, owner: &str, tenant: Option<&str>, agent: &str, run: &str) -> Result<Option<client_tools::ClientToolContinuation>> {
        let (owner, tenant, agent, run) = (owner.to_owned(), tenant.map(str::to_owned), agent.to_owned(), run.to_owned());
        self.dispatch("load_client_tool_continuation_for_run", move |inner| async move {
            inner.load_client_tool_continuation_for_run(&owner, tenant.as_deref(), &agent, &run).await
        }).await
    }

    async fn load_client_tool_continuation_for_session(&self, owner: &str, tenant: Option<&str>, session: &str, batch: &str) -> Result<Option<client_tools::ClientToolContinuation>> {
        let (owner, tenant, session, batch) = (owner.to_owned(), tenant.map(str::to_owned), session.to_owned(), batch.to_owned());
        self.dispatch("load_client_tool_continuation_for_session", move |inner| async move {
            inner.load_client_tool_continuation_for_session(&owner, tenant.as_deref(), &session, &batch).await
        }).await
    }

    async fn compare_and_swap_client_tool_continuation(&self, before: &client_tools::ClientToolContinuation, after: &client_tools::ClientToolContinuation) -> Result<bool> {
        let (before, after) = (before.clone(), after.clone());
        self.dispatch("compare_and_swap_client_tool_continuation", move |inner| async move {
            inner.compare_and_swap_client_tool_continuation(&before, &after).await
        }).await
    }

    fn supports_durable_mcp_elicitations(&self) -> bool {
        self.inner.supports_durable_mcp_elicitations()
    }

    async fn create_mcp_elicitation(&self, _row: &mcp_elicitations::McpElicitationRecord) -> Result<()> {
                let _row = _row.clone();
        self.dispatch("create_mcp_elicitation", move |inner| async move {
                inner.create_mcp_elicitation(&_row).await
            }).await
    }

    async fn list_mcp_elicitations(&self, _owner: &str, _run: &str) -> Result<Vec<mcp_elicitations::McpElicitationRecord>> {
                let _owner = _owner.to_owned();
        let _run = _run.to_owned();
        self.dispatch("list_mcp_elicitations", move |inner| async move {
                inner.list_mcp_elicitations(&_owner, &_run).await
            }).await
    }

    async fn compare_and_swap_mcp_elicitation(&self, _before: &mcp_elicitations::McpElicitationRecord, _after: &mcp_elicitations::McpElicitationRecord) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("compare_and_swap_mcp_elicitation", move |inner| async move {
                inner.compare_and_swap_mcp_elicitation(&_before, &_after).await
            }).await
    }

    fn supports_durable_tool_receipts(&self) -> bool {
        self.inner.supports_durable_tool_receipts()
    }

    fn supports_durable_agui_runs(&self) -> bool {
        self.inner.supports_durable_agui_runs()
    }

    async fn create_agui_run(&self, _row: &agui_runs::AguiRunBinding, _previous: Option<&agui_runs::AguiRunBinding>) -> Result<bool> {
                let _row = _row.clone();
        let _previous = _previous.cloned();
        self.dispatch("create_agui_run", move |inner| async move {
                inner.create_agui_run(&_row, _previous.as_ref()).await
            }).await
    }

    async fn load_agui_run(&self, _owner: &str, _agent: &str, _thread: &str, _run: &str) -> Result<Option<agui_runs::AguiRunBinding>> {
                let _owner = _owner.to_owned();
        let _agent = _agent.to_owned();
        let _thread = _thread.to_owned();
        let _run = _run.to_owned();
        self.dispatch("load_agui_run", move |inner| async move {
                inner.load_agui_run(&_owner, &_agent, &_thread, &_run).await
            }).await
    }

    async fn list_agui_runs(&self, _owner: &str, _agent: &str, _thread: &str) -> Result<Vec<agui_runs::AguiRunBinding>> {
                let _owner = _owner.to_owned();
        let _agent = _agent.to_owned();
        let _thread = _thread.to_owned();
        self.dispatch("list_agui_runs", move |inner| async move {
                inner.list_agui_runs(&_owner, &_agent, &_thread).await
            }).await
    }

    async fn compare_and_swap_agui_run(&self, _before: &agui_runs::AguiRunBinding, _after: &agui_runs::AguiRunBinding) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("compare_and_swap_agui_run", move |inner| async move {
                inner.compare_and_swap_agui_run(&_before, &_after).await
            }).await
    }

    async fn reconcile_a2a_cancellation(&self, _owner: &str, _user: &str, _agent: &str, _task: &str) -> Result<Option<crate::uar::api::a2a::durable::A2aBinding>> {
                let _owner = _owner.to_owned();
        let _user = _user.to_owned();
        let _agent = _agent.to_owned();
        let _task = _task.to_owned();
        self.dispatch("reconcile_a2a_cancellation", move |inner| async move {
                inner.reconcile_a2a_cancellation(&_owner, &_user, &_agent, &_task).await
            }).await
    }

    fn supports_durable_a2a_bindings(&self) -> bool {
        self.inner.supports_durable_a2a_bindings()
    }

    async fn create_a2a_binding(&self, _record: &crate::uar::api::a2a::durable::A2aBinding) -> Result<bool> {
                let _record = _record.clone();
        self.dispatch("create_a2a_binding", move |inner| async move {
                inner.create_a2a_binding(&_record).await
            }).await
    }

    async fn load_a2a_binding(&self, _owner: &str, _agent: &str, _task: &str) -> Result<Option<crate::uar::api::a2a::durable::A2aBinding>> {
                let _owner = _owner.to_owned();
        let _agent = _agent.to_owned();
        let _task = _task.to_owned();
        self.dispatch("load_a2a_binding", move |inner| async move {
                inner.load_a2a_binding(&_owner, &_agent, &_task).await
            }).await
    }

    async fn compare_and_swap_a2a_binding(&self, _before: &crate::uar::api::a2a::durable::A2aBinding, _after: &crate::uar::api::a2a::durable::A2aBinding) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("compare_and_swap_a2a_binding", move |inner| async move {
                inner.compare_and_swap_a2a_binding(&_before, &_after).await
            }).await
    }

    async fn list_a2a_bindings(&self, _owner: &str, _agent: &str) -> Result<Vec<crate::uar::api::a2a::durable::A2aBinding>> {
                let _owner = _owner.to_owned();
        let _agent = _agent.to_owned();
        self.dispatch("list_a2a_bindings", move |inner| async move {
                inner.list_a2a_bindings(&_owner, &_agent).await
            }).await
    }

    fn supports_durable_run_history(&self) -> bool {
        self.inner.supports_durable_run_history()
    }

    async fn upsert_run_history_record(&self, _record: &run_history::RunHistoryRecord) -> Result<()> {
                let _record = _record.clone();
        self.dispatch("upsert_run_history_record", move |inner| async move {
                inner.upsert_run_history_record(&_record).await
            }).await
    }

    async fn load_run_history_record(&self, _run_id: &str) -> Result<Option<run_history::RunHistoryRecord>> {
                let _run_id = _run_id.to_owned();
        self.dispatch("load_run_history_record", move |inner| async move {
                inner.load_run_history_record(&_run_id).await
            }).await
    }

    async fn append_run_history_event(&self, _event: &run_history::RunHistoryEvent) -> Result<()> {
                let _event = _event.clone();
        self.dispatch("append_run_history_event", move |inner| async move {
                inner.append_run_history_event(&_event).await
            }).await
    }

    async fn load_run_history_events(&self, _run_id: &str) -> Result<Vec<run_history::RunHistoryEvent>> {
                let _run_id = _run_id.to_owned();
        self.dispatch("load_run_history_events", move |inner| async move {
                inner.load_run_history_events(&_run_id).await
            }).await
    }

    async fn bind_accounting_delegation(&self, _link: &accounting::AccountingDelegation) -> Result<()> {
                let _link = _link.clone();
        self.dispatch("bind_accounting_delegation", move |inner| async move {
                inner.bind_accounting_delegation(&_link).await
            }).await
    }

    async fn get_accounting_delegation(&self, _digest: &str) -> Result<Option<accounting::AccountingCall>> {
                let _digest = _digest.to_owned();
        self.dispatch("get_accounting_delegation", move |inner| async move {
                inner.get_accounting_delegation(&_digest).await
            }).await
    }

    fn supports_durable_accounting(&self) -> bool {
        self.inner.supports_durable_accounting()
    }

    fn durable_store_eligibility(&self) -> DurableStoreEligibility {
        self.inner.durable_store_eligibility()
    }

    async fn admit_remote_grant(&self, _owner: &str, _accounting_id: &str,
        _grant: &crate::uar::api::a2a::contract::UarUsageGrant,
        _activity: accounting::RemoteGrantActivity) -> Result<accounting::RemoteGrantState> {
                let _owner = _owner.to_owned();
        let _accounting_id = _accounting_id.to_owned();
        let _grant = _grant.clone();
        self.dispatch("admit_remote_grant", move |inner| async move {
                inner.admit_remote_grant(&_owner, &_accounting_id, &_grant, _activity).await
            }).await
    }

    async fn ensure_accounting_scope(&self, _id: &accounting::ScopeIdentity, _limits: &accounting::ScopeLimits) -> Result<accounting::AccountingScope> {
                let _id = _id.clone();
        let _limits = _limits.clone();
        self.dispatch("ensure_accounting_scope", move |inner| async move {
                inner.ensure_accounting_scope(&_id, &_limits).await
            }).await
    }

    async fn get_accounting_scope(&self, _id: &accounting::ScopeIdentity) -> Result<Option<accounting::AccountingScope>> {
                let _id = _id.clone();
        self.dispatch("get_accounting_scope", move |inner| async move {
                inner.get_accounting_scope(&_id).await
            }).await
    }

    async fn update_accounting_scope(&self, _change: &accounting::ScopeUpdate) -> Result<accounting::AccountingScope> {
                let _change = _change.clone();
        self.dispatch("update_accounting_scope", move |inner| async move {
                inner.update_accounting_scope(&_change).await
            }).await
    }

    async fn admit_accounting_call(&self, _admission: &accounting::CallAdmission) -> Result<accounting::AccountingCall> {
                let _admission = _admission.clone();
        self.dispatch("admit_accounting_call", move |inner| async move {
                inner.admit_accounting_call(&_admission).await
            }).await
    }

    async fn record_accounting_usage(&self, _usage: &accounting::CallUsage) -> Result<accounting::AccountingCall> {
                let _usage = _usage.clone();
        self.dispatch("record_accounting_usage", move |inner| async move {
                inner.record_accounting_usage(&_usage).await
            }).await
    }

    async fn list_accounting_calls(&self, _owner: &str, _run: &str) -> Result<Vec<accounting::AccountingCall>> {
                let _owner = _owner.to_owned();
        let _run = _run.to_owned();
        self.dispatch("list_accounting_calls", move |inner| async move {
                inner.list_accounting_calls(&_owner, &_run).await
            }).await
    }

    fn supports_durable_ingestion(&self) -> bool {
        self.inner.supports_durable_ingestion()
    }

    async fn accept_ingestion(&self, _document: &KnowledgeDocument, _source: &[u8]) -> Result<ingestion_records::IngestionJob> {
                let _document = _document.clone();
        let _source = _source.to_vec();
        self.dispatch("accept_ingestion", move |inner| async move {
                inner.accept_ingestion(&_document, &_source).await
            }).await
    }

    async fn discover_ingestion(&self, _limit: usize) -> Result<Vec<ingestion_records::IngestionJob>> {
                self.dispatch("discover_ingestion", move |inner| async move {
                inner.discover_ingestion(_limit).await
            }).await
    }

    async fn next_ingestion_due_at(&self) -> Result<Option<i64>> {
                self.dispatch("next_ingestion_due_at", move |inner| async move {
                inner.next_ingestion_due_at().await
            }).await
    }

    async fn get_ingestion_job(&self, _owner: &str, _document: &str) -> Result<Option<ingestion_records::IngestionJob>> {
                let _owner = _owner.to_owned();
        let _document = _document.to_owned();
        self.dispatch("get_ingestion_job", move |inner| async move {
                inner.get_ingestion_job(&_owner, &_document).await
            }).await
    }

    async fn get_ingestion_source(&self, _job: &ingestion_records::IngestionJob) -> Result<Option<Vec<u8>>> {
                let _job = _job.clone();
        self.dispatch("get_ingestion_source", move |inner| async move {
                inner.get_ingestion_source(&_job).await
            }).await
    }

    async fn transition_ingestion(&self, _before: &ingestion_records::IngestionJob, _after: &ingestion_records::IngestionJob) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("transition_ingestion", move |inner| async move {
                inner.transition_ingestion(&_before, &_after).await
            }).await
    }

    async fn publish_ingestion(&self, _job: &ingestion_records::IngestionJob, _publication: &ingestion_records::IngestionPublication) -> Result<()> {
                let _job = _job.clone();
        let _publication = _publication.clone();
        self.dispatch("publish_ingestion", move |inner| async move {
                inner.publish_ingestion(&_job, &_publication).await
            }).await
    }

    async fn reconcile_ingestion(&self) -> Result<()> {
                self.dispatch("reconcile_ingestion", move |inner| async move {
                inner.reconcile_ingestion().await
            }).await
    }

    fn supports_channel_observers(&self) -> bool {
        self.inner.supports_channel_observers()
    }

    fn channel_observer_unavailable_reason(&self) -> &'static str {
        self.inner.channel_observer_unavailable_reason()
    }

    async fn create_channel_subscription(&self, _record: &ChannelSubscription) -> Result<ChannelSubscription> {
                let _record = _record.clone();
        self.dispatch("create_channel_subscription", move |inner| async move {
                inner.create_channel_subscription(&_record).await
            }).await
    }

    async fn load_channel_subscription(&self, _owner: &str, _workspace: &str, _id: &str) -> Result<Option<ChannelSubscription>> {
                let _owner = _owner.to_owned();
        let _workspace = _workspace.to_owned();
        let _id = _id.to_owned();
        self.dispatch("load_channel_subscription", move |inner| async move {
                inner.load_channel_subscription(&_owner, &_workspace, &_id).await
            }).await
    }

    async fn list_channel_subscriptions(&self, _owner: &str, _workspace: &str) -> Result<Vec<ChannelSubscription>> {
                let _owner = _owner.to_owned();
        let _workspace = _workspace.to_owned();
        self.dispatch("list_channel_subscriptions", move |inner| async move {
                inner.list_channel_subscriptions(&_owner, &_workspace).await
            }).await
    }

    async fn compare_and_swap_channel_subscription(&self, _before: &ChannelSubscription, _after: &ChannelSubscription) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("compare_and_swap_channel_subscription", move |inner| async move {
                inner.compare_and_swap_channel_subscription(&_before, &_after).await
            }).await
    }

    async fn create_channel_inbox_entry(&self, _record: &ChannelInboxEntry) -> Result<ChannelInboxEntry> {
                let _record = _record.clone();
        self.dispatch("create_channel_inbox_entry", move |inner| async move {
                inner.create_channel_inbox_entry(&_record).await
            }).await
    }

    async fn load_channel_inbox_entry(&self, _owner: &str, _workspace: &str, _subscription: &str, _delivery: &str) -> Result<Option<ChannelInboxEntry>> {
                let _owner = _owner.to_owned();
        let _workspace = _workspace.to_owned();
        let _subscription = _subscription.to_owned();
        let _delivery = _delivery.to_owned();
        self.dispatch("load_channel_inbox_entry", move |inner| async move {
                inner.load_channel_inbox_entry(&_owner, &_workspace, &_subscription, &_delivery).await
            }).await
    }

    async fn list_channel_inbox_entries(&self, _owner: &str, _workspace: &str, _subscription: &str) -> Result<Vec<ChannelInboxEntry>> {
                let _owner = _owner.to_owned();
        let _workspace = _workspace.to_owned();
        let _subscription = _subscription.to_owned();
        self.dispatch("list_channel_inbox_entries", move |inner| async move {
                inner.list_channel_inbox_entries(&_owner, &_workspace, &_subscription).await
            }).await
    }

    async fn compare_and_swap_channel_inbox_entry(&self, _before: &ChannelInboxEntry, _after: &ChannelInboxEntry) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("compare_and_swap_channel_inbox_entry", move |inner| async move {
                inner.compare_and_swap_channel_inbox_entry(&_before, &_after).await
            }).await
    }

    fn supports_durable_observers(&self) -> bool {
        self.inner.supports_durable_observers()
    }

    async fn create_observer_subscription(
        &self,
        _subscription: &ObserverSubscription,
    ) -> Result<ObserverSubscription> {
                let _subscription = _subscription.clone();
        self.dispatch("create_observer_subscription", move |inner| async move {
                inner.create_observer_subscription(&_subscription).await
            }).await
    }

    async fn load_observer_subscription(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _subscription_id: &str,
    ) -> Result<Option<ObserverSubscription>> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        let _subscription_id = _subscription_id.to_owned();
        self.dispatch("load_observer_subscription", move |inner| async move {
                inner.load_observer_subscription(&_owner_id, &_workspace_id, &_subscription_id).await
            }).await
    }

    async fn list_observer_subscriptions(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
    ) -> Result<Vec<ObserverSubscription>> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        self.dispatch("list_observer_subscriptions", move |inner| async move {
                inner.list_observer_subscriptions(&_owner_id, &_workspace_id).await
            }).await
    }

    async fn list_all_observer_subscriptions(&self) -> Result<Vec<ObserverSubscription>> {
                self.dispatch("list_all_observer_subscriptions", move |inner| async move {
                inner.list_all_observer_subscriptions().await
            }).await
    }

    async fn compare_and_swap_observer_subscription(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _expected_revision: u64,
        _next: &ObserverSubscription,
    ) -> Result<bool> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        let _next = _next.clone();
        self.dispatch("compare_and_swap_observer_subscription", move |inner| async move {
                inner.compare_and_swap_observer_subscription(&_owner_id, &_workspace_id, _expected_revision, &_next).await
            }).await
    }

    async fn list_instance_occurrences(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _source_instance_id: &str,
        _after_sequence: Option<u64>,
        _limit: usize,
    ) -> Result<Vec<ObserverOccurrence>> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        let _source_instance_id = _source_instance_id.to_owned();
        self.dispatch("list_instance_occurrences", move |inner| async move {
                inner.list_instance_occurrences(&_owner_id, &_workspace_id, &_source_instance_id, _after_sequence, _limit).await
            }).await
    }

    async fn instance_occurrence_bounds(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _source_instance_id: &str,
    ) -> Result<ObserverOccurrenceBounds> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        let _source_instance_id = _source_instance_id.to_owned();
        self.dispatch("instance_occurrence_bounds", move |inner| async move {
                inner.instance_occurrence_bounds(&_owner_id, &_workspace_id, &_source_instance_id).await
            }).await
    }

    fn supports_durable_agent_instances(&self) -> bool {
        self.inner.supports_durable_agent_instances()
    }

    async fn create_agent_instance(
        &self,
        _record: &AgentInstanceRecord,
    ) -> Result<AgentInstanceRecord> {
                let _record = _record.clone();
        self.dispatch("create_agent_instance", move |inner| async move {
                inner.create_agent_instance(&_record).await
            }).await
    }

    async fn load_agent_instance(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _instance_id: &str,
    ) -> Result<Option<AgentInstanceRecord>> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        let _instance_id = _instance_id.to_owned();
        self.dispatch("load_agent_instance", move |inner| async move {
                inner.load_agent_instance(&_owner_id, &_workspace_id, &_instance_id).await
            }).await
    }

    async fn list_agent_instances(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
    ) -> Result<Vec<AgentInstanceRecord>> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        self.dispatch("list_agent_instances", move |inner| async move {
                inner.list_agent_instances(&_owner_id, &_workspace_id).await
            }).await
    }

    async fn compare_and_swap_agent_instance(
        &self,
        _owner_id: &str,
        _workspace_id: &str,
        _expected_revision: u64,
        _expected_epoch: u64,
        _next: &AgentInstanceRecord,
    ) -> Result<bool> {
                let _owner_id = _owner_id.to_owned();
        let _workspace_id = _workspace_id.to_owned();
        let _next = _next.clone();
        self.dispatch("compare_and_swap_agent_instance", move |inner| async move {
                inner.compare_and_swap_agent_instance(&_owner_id, &_workspace_id, _expected_revision, _expected_epoch, &_next).await
            }).await
    }

    async fn create_presentation(
        &self,
        owner_id: &str,
        draft: &PresentationDraft,
    ) -> Result<Presentation> {
                let owner_id = owner_id.to_owned();
        let draft = draft.clone();
        self.dispatch("create_presentation", move |inner| async move {
                inner.create_presentation(&owner_id, &draft).await
            }).await
    }

    async fn get_presentation(&self, owner_id: &str, id: &str) -> Result<Option<Presentation>> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("get_presentation", move |inner| async move {
                inner.get_presentation(&owner_id, &id).await
            }).await
    }

    async fn list_presentations(&self, owner_id: &str) -> Result<Vec<Presentation>> {
                let owner_id = owner_id.to_owned();
        self.dispatch("list_presentations", move |inner| async move {
                inner.list_presentations(&owner_id).await
            }).await
    }

    async fn update_presentation(
        &self,
        owner_id: &str,
        id: &str,
        expected_revision: u64,
        draft: &PresentationDraft,
    ) -> Result<Presentation> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        let draft = draft.clone();
        self.dispatch("update_presentation", move |inner| async move {
                inner.update_presentation(&owner_id, &id, expected_revision, &draft).await
            }).await
    }

    async fn delete_presentation(
        &self,
        owner_id: &str,
        id: &str,
        expected_revision: u64,
    ) -> Result<()> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("delete_presentation", move |inner| async move {
                inner.delete_presentation(&owner_id, &id, expected_revision).await
            }).await
    }

    async fn save_session(&self, session: &Session) -> Result<()> {
                let session = session.clone();
        self.dispatch("save_session", move |inner| async move {
                inner.save_session(&session).await
            }).await
    }

    async fn load_session(&self, owner_id: &str, id: &str) -> Result<Option<Session>> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("load_session", move |inner| async move {
                inner.load_session(&owner_id, &id).await
            }).await
    }

    async fn list_sessions(&self, owner_id: &str) -> Result<Vec<Session>> {
                let owner_id = owner_id.to_owned();
        self.dispatch("list_sessions", move |inner| async move {
                inner.list_sessions(&owner_id).await
            }).await
    }

    async fn delete_session(&self, owner_id: &str, id: &str) -> Result<bool> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("delete_session", move |inner| async move {
                inner.delete_session(&owner_id, &id).await
            }).await
    }

    async fn list_expired_sessions(
        &self,
        _cutoff: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<(String, String)>> {
                self.dispatch("list_expired_sessions", move |inner| async move {
                inner.list_expired_sessions(_cutoff).await
            }).await
    }

    fn subscribe_persisted_session_changes(&self) -> Option<tokio::sync::watch::Receiver<u64>> {
        self.inner.subscribe_persisted_session_changes()
    }

    async fn next_persisted_session_expiry(
        &self,
        _idle_timeout: chrono::Duration,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
                self.dispatch("next_persisted_session_expiry", move |inner| async move {
                inner.next_persisted_session_expiry(_idle_timeout).await
            }).await
    }

    async fn save_conversation_policy(&self, record: &ConversationPolicyRecord) -> Result<()> {
                let record = record.clone();
        self.dispatch("save_conversation_policy", move |inner| async move {
                inner.save_conversation_policy(&record).await
            }).await
    }

    async fn load_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>> {
                let owner_id = owner_id.to_owned();
        let conversation_id = conversation_id.to_owned();
        self.dispatch("load_conversation_policy", move |inner| async move {
                inner.load_conversation_policy(&owner_id, &conversation_id).await
            }).await
    }

    async fn save_principal_conversation_policy(
        &self,
        record: &ConversationPolicyRecord,
        expected: Option<&crate::uar::domain::policy::RunPolicy>,
    ) -> Result<bool> {
                let record = record.clone();
        let expected = expected.cloned();
        self.dispatch("save_principal_conversation_policy", move |inner| async move {
                inner.save_principal_conversation_policy(&record, expected.as_ref()).await
            }).await
    }

    async fn load_principal_conversation_policy(
        &self,
        owner_id: &str,
        conversation_id: &str,
    ) -> Result<Option<ConversationPolicyRecord>> {
                let owner_id = owner_id.to_owned();
        let conversation_id = conversation_id.to_owned();
        self.dispatch("load_principal_conversation_policy", move |inner| async move {
                inner.load_principal_conversation_policy(&owner_id, &conversation_id).await
            }).await
    }

    async fn delete_conversation_policy(&self, owner_id: &str, conversation_id: &str)
    -> Result<()> {
                let owner_id = owner_id.to_owned();
        let conversation_id = conversation_id.to_owned();
        self.dispatch("delete_conversation_policy", move |inner| async move {
                inner.delete_conversation_policy(&owner_id, &conversation_id).await
            }).await
    }

    async fn save_user_prompt_caching_settings(
        &self,
        settings: &UserPromptCachingSettings,
    ) -> Result<()> {
                let settings = settings.clone();
        self.dispatch("save_user_prompt_caching_settings", move |inner| async move {
                inner.save_user_prompt_caching_settings(&settings).await
            }).await
    }

    async fn load_user_prompt_caching_settings(
        &self,
        principal_id: &str,
    ) -> Result<Option<UserPromptCachingSettings>> {
                let principal_id = principal_id.to_owned();
        self.dispatch("load_user_prompt_caching_settings", move |inner| async move {
                inner.load_user_prompt_caching_settings(&principal_id).await
            }).await
    }

    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()> {
                let skill = skill.clone();
        let embedding = embedding.to_vec();
        self.dispatch("save_skill", move |inner| async move {
                inner.save_skill(&skill, &embedding).await
            }).await
    }

    async fn create_skill_if_absent(&self, _skill: &Skill, _embedding: &[f32]) -> Result<bool> {
                let _skill = _skill.clone();
        let _embedding = _embedding.to_vec();
        self.dispatch("create_skill_if_absent", move |inner| async move {
                inner.create_skill_if_absent(&_skill, &_embedding).await
            }).await
    }

    async fn update_skill_selection(
        &self,
        skill_id: &str,
        enabled: bool,
        scoped_config: &[crate::uar::domain::skills::ScopedSkillConfig],
    ) -> Result<bool> {
                let skill_id = skill_id.to_owned();
        let scoped_config = scoped_config.to_vec();
        self.dispatch("update_skill_selection", move |inner| async move {
                inner.update_skill_selection(&skill_id, enabled, &scoped_config).await
            }).await
    }

    async fn search_skills(&self, query_vec: &[f32], limit: usize) -> Result<Vec<SkillMatch>> {
                let query_vec = query_vec.to_vec();
        self.dispatch("search_skills", move |inner| async move {
                inner.search_skills(&query_vec, limit).await
            }).await
    }

    async fn list_skills(&self) -> Result<Vec<Skill>> {
                self.dispatch("list_skills", move |inner| async move {
                inner.list_skills().await
            }).await
    }

    async fn delete_skill(&self, _id: &str) -> Result<()> {
                let _id = _id.to_owned();
        self.dispatch("delete_skill", move |inner| async move {
                inner.delete_skill(&_id).await
            }).await
    }

    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> {
                let kb = kb.clone();
        self.dispatch("save_knowledge_base", move |inner| async move {
                inner.save_knowledge_base(&kb).await
            }).await
    }

    async fn begin_knowledge_regeneration(
        &self, owner: &str, kb: &str,
        target: &crate::uar::domain::knowledge::EmbeddingFingerprint,
    ) -> Result<crate::uar::domain::knowledge::KnowledgeRegeneration> {
        let owner = owner.to_owned();
        let kb = kb.to_owned();
        let target = target.clone();
        self.dispatch("begin_knowledge_regeneration", move |inner| async move {
            inner.begin_knowledge_regeneration(&owner, &kb, &target).await
        }).await
    }

    async fn search_knowledge_scoped_in_space(
        &self, owner_id: &str, kb_ids: &[&str], query_vec: &[f32],
        limit: usize, min_score: f32,
        space: &crate::uar::domain::knowledge::EmbeddingFingerprint,
    ) -> Result<Vec<KnowledgeMatch>> {
        let owner_id = owner_id.to_owned();
        let kb_ids = kb_ids.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>();
        let query_vec = query_vec.to_vec();
        let space = space.clone();
        self.dispatch("search_knowledge_scoped_in_space", move |inner| async move {
            let ids = kb_ids.iter().map(String::as_str).collect::<Vec<_>>();
            inner.search_knowledge_scoped_in_space(&owner_id, &ids, &query_vec, limit, min_score, &space).await
        }).await
    }

    async fn get_knowledge_base(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeBase>> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("get_knowledge_base", move |inner| async move {
                inner.get_knowledge_base(&owner_id, &id).await
            }).await
    }

    async fn get_knowledge_base_by_name(
        &self,
        owner_id: &str,
        name: &str,
    ) -> Result<Option<KnowledgeBase>> {
                let owner_id = owner_id.to_owned();
        let name = name.to_owned();
        self.dispatch("get_knowledge_base_by_name", move |inner| async move {
                inner.get_knowledge_base_by_name(&owner_id, &name).await
            }).await
    }

    async fn list_knowledge_bases(&self, owner_id: &str) -> Result<Vec<KnowledgeBase>> {
                let owner_id = owner_id.to_owned();
        self.dispatch("list_knowledge_bases", move |inner| async move {
                inner.list_knowledge_bases(&owner_id).await
            }).await
    }

    async fn delete_knowledge_base(&self, owner_id: &str, id: &str) -> Result<()> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("delete_knowledge_base", move |inner| async move {
                inner.delete_knowledge_base(&owner_id, &id).await
            }).await
    }

    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
                let chunk = chunk.clone();
        self.dispatch("save_chunk", move |inner| async move {
                inner.save_chunk(&chunk).await
            }).await
    }

    async fn search_knowledge(
        &self,
        owner_id: &str,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
                let owner_id = owner_id.to_owned();
        let query_vec = query_vec.to_vec();
        self.dispatch("search_knowledge", move |inner| async move {
                inner.search_knowledge(&owner_id, &query_vec, limit, min_score).await
            }).await
    }

    async fn search_knowledge_scoped(
        &self,
        owner_id: &str,
        kb_ids: &[&str],
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
                let owner_id = owner_id.to_owned();
        let kb_ids = kb_ids.iter().map(|value| (*value).to_owned()).collect::<Vec<_>>();
        let query_vec = query_vec.to_vec();
        self.dispatch("search_knowledge_scoped", move |inner| async move {
                let kb_ids_refs = kb_ids.iter().map(String::as_str).collect::<Vec<_>>();
                inner.search_knowledge_scoped(&owner_id, &kb_ids_refs, &query_vec, limit, min_score).await
            }).await
    }

    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> {
                let doc = doc.clone();
        self.dispatch("save_document", move |inner| async move {
                inner.save_document(&doc).await
            }).await
    }

    async fn get_document(&self, owner_id: &str, id: &str) -> Result<Option<KnowledgeDocument>> {
                let owner_id = owner_id.to_owned();
        let id = id.to_owned();
        self.dispatch("get_document", move |inner| async move {
                inner.get_document(&owner_id, &id).await
            }).await
    }

    async fn list_documents(&self, owner_id: &str, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
                let owner_id = owner_id.to_owned();
        let kb_id = kb_id.to_owned();
        self.dispatch("list_documents", move |inner| async move {
                inner.list_documents(&owner_id, &kb_id).await
            }).await
    }

    async fn count_documents(&self, owner_id: &str, kb_id: &str) -> Result<usize> {
                let owner_id = owner_id.to_owned();
        let kb_id = kb_id.to_owned();
        self.dispatch("count_documents", move |inner| async move {
                inner.count_documents(&owner_id, &kb_id).await
            }).await
    }

    async fn update_document_status(
        &self,
        owner_id: &str,
        doc_id: &str,
        status: &DocumentStatus,
    ) -> Result<()> {
                let owner_id = owner_id.to_owned();
        let doc_id = doc_id.to_owned();
        let status = status.clone();
        self.dispatch("update_document_status", move |inner| async move {
                inner.update_document_status(&owner_id, &doc_id, &status).await
            }).await
    }

    async fn delete_document(&self, owner_id: &str, doc_id: &str) -> Result<()> {
                let owner_id = owner_id.to_owned();
        let doc_id = doc_id.to_owned();
        self.dispatch("delete_document", move |inner| async move {
                inner.delete_document(&owner_id, &doc_id).await
            }).await
    }

    async fn create_agent_root(
        &self,
        owner_id: &str,
        thread: &AgentThread,
    ) -> Result<PersistedAgentThread> {
                let owner_id = owner_id.to_owned();
        let thread = thread.clone();
        self.dispatch("create_agent_root", move |inner| async move {
                inner.create_agent_root(&owner_id, &thread).await
            }).await
    }

    async fn create_agent_child(
        &self,
        owner_id: &str,
        thread: &AgentThread,
        edge: &AgentEdge,
    ) -> Result<PersistedAgentThread> {
                let owner_id = owner_id.to_owned();
        let thread = thread.clone();
        let edge = edge.clone();
        self.dispatch("create_agent_child", move |inner| async move {
                inner.create_agent_child(&owner_id, &thread, &edge).await
            }).await
    }

    async fn load_agent_thread(
        &self,
        owner_id: &str,
        thread_id: &str,
    ) -> Result<Option<PersistedAgentThread>> {
                let owner_id = owner_id.to_owned();
        let thread_id = thread_id.to_owned();
        self.dispatch("load_agent_thread", move |inner| async move {
                inner.load_agent_thread(&owner_id, &thread_id).await
            }).await
    }

    async fn update_agent_thread(
        &self,
        owner_id: &str,
        expected_revision: u64,
        thread: &AgentThread,
    ) -> Result<PersistedAgentThread> {
                let owner_id = owner_id.to_owned();
        let thread = thread.clone();
        self.dispatch("update_agent_thread", move |inner| async move {
                inner.update_agent_thread(&owner_id, expected_revision, &thread).await
            }).await
    }

    async fn list_agent_threads(
        &self,
        owner_id: &str,
        root_run_id: &str,
    ) -> Result<Vec<PersistedAgentThread>> {
                let owner_id = owner_id.to_owned();
        let root_run_id = root_run_id.to_owned();
        self.dispatch("list_agent_threads", move |inner| async move {
                inner.list_agent_threads(&owner_id, &root_run_id).await
            }).await
    }

    async fn list_agent_edges(&self, owner_id: &str, root_run_id: &str) -> Result<Vec<AgentEdge>> {
                let owner_id = owner_id.to_owned();
        let root_run_id = root_run_id.to_owned();
        self.dispatch("list_agent_edges", move |inner| async move {
                inner.list_agent_edges(&owner_id, &root_run_id).await
            }).await
    }

    async fn save_canonical_tool_receipt(
        &self,
        receipt: &CanonicalToolReceipt,
    ) -> Result<CanonicalToolReceipt> {
                let receipt = receipt.clone();
        self.dispatch("save_canonical_tool_receipt", move |inner| async move {
                inner.save_canonical_tool_receipt(&receipt).await
            }).await
    }

    async fn list_canonical_tool_receipts(
        &self,
        owner_id: &str,
        run_id: &str,
    ) -> Result<Vec<CanonicalToolReceipt>> {
                let owner_id = owner_id.to_owned();
        let run_id = run_id.to_owned();
        self.dispatch("list_canonical_tool_receipts", move |inner| async move {
                inner.list_canonical_tool_receipts(&owner_id, &run_id).await
            }).await
    }

    fn supports_durable_approvals(&self) -> bool {
        self.inner.supports_durable_approvals()
    }

    async fn create_approval_record(&self, _record: &approval_decisions::ApprovalRecord) -> Result<()> {
                let _record = _record.clone();
        self.dispatch("create_approval_record", move |inner| async move {
                inner.create_approval_record(&_record).await
            }).await
    }

    async fn transition_approval_record(&self, _before: &approval_decisions::ApprovalRecord, _after: &approval_decisions::ApprovalRecord) -> Result<bool> {
                let _before = _before.clone();
        let _after = _after.clone();
        self.dispatch("transition_approval_record", move |inner| async move {
                inner.transition_approval_record(&_before, &_after).await
            }).await
    }

    async fn list_approval_records(&self, _owner: &str, _run: &str) -> Result<Vec<approval_decisions::ApprovalRecord>> {
                let _owner = _owner.to_owned();
        let _run = _run.to_owned();
        self.dispatch("list_approval_records", move |inner| async move {
                inner.list_approval_records(&_owner, &_run).await
            }).await
    }

    async fn save_tool_admission_evidence(
        &self,
        _evidence: &ToolAdmissionEvidence,
    ) -> Result<ToolAdmissionEvidence> {
                let _evidence = _evidence.clone();
        self.dispatch("save_tool_admission_evidence", move |inner| async move {
                inner.save_tool_admission_evidence(&_evidence).await
            }).await
    }

    async fn list_tool_admission_evidence(
        &self,
        _owner_id: &str,
    ) -> Result<Vec<ToolAdmissionEvidence>> {
                let _owner_id = _owner_id.to_owned();
        self.dispatch("list_tool_admission_evidence", move |inner| async move {
                inner.list_tool_admission_evidence(&_owner_id).await
            }).await
    }

    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()> {
                let agent = agent.clone();
        self.dispatch("save_agent", move |inner| async move {
                inner.save_agent(&agent).await
            }).await
    }

    async fn save_agent_if_unchanged(
        &self,
        expected: &crate::uar::domain::artifact::AgentArtifact,
        updated: &crate::uar::domain::artifact::AgentArtifact,
    ) -> Result<bool> {
                let expected = expected.clone();
        let updated = updated.clone();
        self.dispatch("save_agent_if_unchanged", move |inner| async move {
                inner.save_agent_if_unchanged(&expected, &updated).await
            }).await
    }

    async fn load_agent(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
                let id = id.to_owned();
        self.dispatch("load_agent", move |inner| async move {
                inner.load_agent(&id).await
            }).await
    }

    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
                let name = name.to_owned();
        self.dispatch("load_agent_by_name", move |inner| async move {
                inner.load_agent_by_name(&name).await
            }).await
    }

    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>> {
                self.dispatch("list_agents", move |inner| async move {
                inner.list_agents().await
            }).await
    }

    async fn delete_agent(&self, _id: &str) -> Result<()> {
                let _id = _id.to_owned();
        self.dispatch("delete_agent", move |inner| async move {
                inner.delete_agent(&_id).await
            }).await
    }

    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()> {
                let memory = memory.clone();
        self.dispatch("save_memory", move |inner| async move {
                inner.save_memory(&memory).await
            }).await
    }

    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>> {
                let agent_id = agent_id.map(str::to_owned);
        let query_vec = query_vec.to_vec();
        self.dispatch("search_memory", move |inner| async move {
                inner.search_memory(agent_id.as_deref(), &query_vec, limit, min_score).await
            }).await
    }

    async fn upsert_settings_type(&self, _st: &SettingsType) -> Result<Uuid> {
                let _st = _st.clone();
        self.dispatch("upsert_settings_type", move |inner| async move {
                inner.upsert_settings_type(&_st).await
            }).await
    }

    async fn list_settings_types(&self) -> Result<Vec<SettingsType>> {
                self.dispatch("list_settings_types", move |inner| async move {
                inner.list_settings_types().await
            }).await
    }

    async fn get_settings_type(&self, _key: &str) -> Result<Option<SettingsType>> {
                let _key = _key.to_owned();
        self.dispatch("get_settings_type", move |inner| async move {
                inner.get_settings_type(&_key).await
            }).await
    }

    async fn upsert_setting(&self, _setting: &Settings) -> Result<()> {
                let _setting = _setting.clone();
        self.dispatch("upsert_setting", move |inner| async move {
                inner.upsert_setting(&_setting).await
            }).await
    }

    async fn get_setting(&self, _key: &str) -> Result<Option<Settings>> {
                let _key = _key.to_owned();
        self.dispatch("get_setting", move |inner| async move {
                inner.get_setting(&_key).await
            }).await
    }

    async fn compare_and_swap_global_presentation_policy(
        &self,
        expected: &serde_json::Value,
        selection: &crate::uar::domain::policy::ResourceSelection,
    ) -> Result<bool> {
                let expected = expected.clone();
        let selection = selection.clone();
        self.dispatch("compare_and_swap_global_presentation_policy", move |inner| async move {
                inner.compare_and_swap_global_presentation_policy(&expected, &selection).await
            }).await
    }

    async fn list_settings(
        &self,
        _type_key: Option<&str>,
        _parent_id: Option<Uuid>,
    ) -> Result<Vec<Settings>> {
                let _type_key = _type_key.map(str::to_owned);
        self.dispatch("list_settings", move |inner| async move {
                inner.list_settings(_type_key.as_deref(), _parent_id).await
            }).await
    }

    async fn delete_setting(&self, _key: &str) -> Result<()> {
                let _key = _key.to_owned();
        self.dispatch("delete_setting", move |inner| async move {
                inner.delete_setting(&_key).await
            }).await
    }

    async fn insert_attachment(&self, _meta: &AttachmentMeta) -> Result<()> {
                let _meta = _meta.clone();
        self.dispatch("insert_attachment", move |inner| async move {
                inner.insert_attachment(&_meta).await
            }).await
    }

    async fn get_attachment(&self, _id: &str) -> Result<Option<AttachmentMeta>> {
                let _id = _id.to_owned();
        self.dispatch("get_attachment", move |inner| async move {
                inner.get_attachment(&_id).await
            }).await
    }

    async fn list_attachments_for_session(&self, _session_id: &str) -> Result<Vec<AttachmentMeta>> {
                let _session_id = _session_id.to_owned();
        self.dispatch("list_attachments_for_session", move |inner| async move {
                inner.list_attachments_for_session(&_session_id).await
            }).await
    }

    async fn register_supervisor_domain(&self, _registration: &crate::uar::runtime::supervisor::SupervisorRegistration) -> Result<()> {
                let _registration = _registration.clone();
        self.dispatch("register_supervisor_domain", move |inner| async move {
                inner.register_supervisor_domain(&_registration).await
            }).await
    }

    async fn load_supervisor_domain(&self, _epoch: &str) -> Result<Option<crate::uar::runtime::supervisor::SupervisorRegistration>> {
                let _epoch = _epoch.to_owned();
        self.dispatch("load_supervisor_domain", move |inner| async move {
                inner.load_supervisor_domain(&_epoch).await
            }).await
    }

    async fn create_supervisor_challenge(&self, _challenge: &crate::uar::runtime::supervisor::SupervisorChallenge) -> Result<()> {
                let _challenge = _challenge.clone();
        self.dispatch("create_supervisor_challenge", move |inner| async move {
                inner.create_supervisor_challenge(&_challenge).await
            }).await
    }

    async fn load_supervisor_challenge(&self, _id: &str) -> Result<Option<crate::uar::runtime::supervisor::SupervisorChallenge>> {
                let _id = _id.to_owned();
        self.dispatch("load_supervisor_challenge", move |inner| async move {
                inner.load_supervisor_challenge(&_id).await
            }).await
    }

    async fn commit_supervisor_checkpoint_exclusion(&self, _proof: &crate::uar::runtime::supervisor::VerifiedSupervisorExclusion) -> Result<()> {
                let _proof = _proof.clone();
        self.dispatch("commit_supervisor_checkpoint_exclusion", move |inner| async move {
                inner.commit_supervisor_checkpoint_exclusion(&_proof).await
            }).await
    }

    fn supports_durable_checkpoint_resume(&self) -> bool {
        self.inner.supports_durable_checkpoint_resume()
    }

    async fn save_resumable_checkpoint(&self, _envelope: &checkpoint_resume::CheckpointResumeEnvelope) -> Result<()> {
                let _envelope = _envelope.clone();
        self.dispatch("save_resumable_checkpoint", move |inner| async move {
                inner.save_resumable_checkpoint(&_envelope).await
            }).await
    }

    async fn load_checkpoint_resume(&self, _owner_key: &str, _run_id: &str, _checkpoint_id: &str) -> Result<Option<checkpoint_resume::CheckpointResumeEnvelope>> {
                let _owner_key = _owner_key.to_owned();
        let _run_id = _run_id.to_owned();
        let _checkpoint_id = _checkpoint_id.to_owned();
        self.dispatch("load_checkpoint_resume", move |inner| async move {
                inner.load_checkpoint_resume(&_owner_key, &_run_id, &_checkpoint_id).await
            }).await
    }

    async fn list_checkpoint_resumes(&self, _owner_key: &str, _run_id: &str) -> Result<Vec<checkpoint_resume::CheckpointResumeEnvelope>> {
                let _owner_key = _owner_key.to_owned();
        let _run_id = _run_id.to_owned();
        self.dispatch("list_checkpoint_resumes", move |inner| async move {
                inner.list_checkpoint_resumes(&_owner_key, &_run_id).await
            }).await
    }

    async fn claim_checkpoint_resume(&self, _envelope: &checkpoint_resume::CheckpointResumeEnvelope, _claim: &checkpoint_resume::CheckpointContinuationClaim) -> Result<checkpoint_resume::CheckpointClaimOutcome> {
                let _envelope = _envelope.clone();
        let _claim = _claim.clone();
        self.dispatch("claim_checkpoint_resume", move |inner| async move {
                inner.claim_checkpoint_resume(&_envelope, &_claim).await
            }).await
    }

    async fn exclude_checkpoint_runtime_epoch(&self, _runtime_epoch: &str) -> Result<()> {
                let _runtime_epoch = _runtime_epoch.to_owned();
        self.dispatch("exclude_checkpoint_runtime_epoch", move |inner| async move {
                inner.exclude_checkpoint_runtime_epoch(&_runtime_epoch).await
            }).await
    }

    async fn save_checkpoint(
        &self,
        _checkpoint: &crate::uar::runtime::checkpoint::Checkpoint,
    ) -> Result<()> {
                let _checkpoint = _checkpoint.clone();
        self.dispatch("save_checkpoint", move |inner| async move {
                inner.save_checkpoint(&_checkpoint).await
            }).await
    }

    async fn load_checkpoint(
        &self,
        _id: &str,
    ) -> Result<Option<crate::uar::runtime::checkpoint::Checkpoint>> {
                let _id = _id.to_owned();
        self.dispatch("load_checkpoint", move |inner| async move {
                inner.load_checkpoint(&_id).await
            }).await
    }

    async fn list_checkpoints(
        &self,
        _run_id: &str,
    ) -> Result<Vec<crate::uar::runtime::checkpoint::Checkpoint>> {
                let _run_id = _run_id.to_owned();
        self.dispatch("list_checkpoints", move |inner| async move {
                inner.list_checkpoints(&_run_id).await
            }).await
    }

    async fn record_cost_entry(&self, _scope: &str, _scope_id: &str, _cost_usd: f64) -> Result<()> {
                let _scope = _scope.to_owned();
        let _scope_id = _scope_id.to_owned();
        self.dispatch("record_cost_entry", move |inner| async move {
                inner.record_cost_entry(&_scope, &_scope_id, _cost_usd).await
            }).await
    }

    async fn list_cost_history(&self, _scope: &str, _scope_id: &str) -> Result<Vec<CostEntry>> {
                let _scope = _scope.to_owned();
        let _scope_id = _scope_id.to_owned();
        self.dispatch("list_cost_history", move |inner| async move {
                inner.list_cost_history(&_scope, &_scope_id).await
            }).await
    }
}
