//! Simulator-only host ABI. Built as the SDK's ios_acceptance staticlib example.
//! The trusted host owns the executor, real SDK runtime, and durable store.
use futures::FutureExt;
use serde_json::{Value, json};
use std::{collections::BTreeMap, ffi::{CStr, CString, c_char}, panic::{AssertUnwindSafe, catch_unwind}, sync::{Arc, Mutex, OnceLock}, time::Duration};
use universal_agent_runtime::{config::LlmConfig, llm::{LiterLlmDriver, ProviderConfig}, uar::{domain::{artifact::AgentArtifact, policy::RunPolicy, skills::Skill}, governance::engine::GovernanceEngine, persistence::providers::surreal::SurrealDbProvider, runtime::{manager::StreamEvent, tool_admission::StandaloneToolAdmissionPort}}};
use universal_agent_runtime_sdk::runtime::Runtime;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
struct Host {
    executor: tokio::runtime::Runtime,
    runtime: Runtime,
    store: Arc<SurrealDbProvider>,
    receivers: BTreeMap<String, tokio::sync::broadcast::Receiver<StreamEvent>>,
}
static HOST: OnceLock<Mutex<Option<Host>>> = OnceLock::new();
fn field<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value.get(key).and_then(Value::as_str).ok_or_else(|| format!("missing {key}").into())
}
fn event_value(event: &StreamEvent) -> Result<Value> {
    Ok(json!({"id":event.id,"event":serde_json::to_value(&event.event)?}))
}

fn invoke(request: Value) -> Result<Value> {
    let mut owner = HOST.get_or_init(|| Mutex::new(None)).lock().map_err(|_| "host poisoned")?;
    let op = field(&request, "op")?;
    if op == "open" {
        if owner.is_some() { return Err("host already open".into()); }
        let executor = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build()?;
        let result: Result<(Runtime, Arc<SurrealDbProvider>)> = executor.block_on(async {
            let store_path = field(&request, "store_path")?;
            if !std::path::Path::new(store_path).is_absolute() { return Err("absolute store path required".into()); }
            let model = field(&request, "model")?.to_owned();
            if model.is_empty() || model.contains('/') { return Err("bare exact wire model alias required".into()); }
            let endpoint = field(&request, "base_url")?.to_owned();
            let key = field(&request, "proxy_token")?.to_owned();
            let context_window = u32::try_from(request["context_window"].as_u64().ok_or("model context required")?)?;
            let output_cap = u32::try_from(request["model_output_cap"].as_u64().ok_or("proxy output ceiling required")?)?;
            if output_cap == 0 || context_window <= output_cap { return Err("invalid admitted model limits".into()); }
            let destination = url::Url::parse(&endpoint)?;
            if destination.scheme() != "http" || destination.host_str() != Some("127.0.0.1") || destination.port().is_none() || destination.path() != "/v1" || destination.query().is_some() || !destination.username().is_empty() || destination.password().is_some() { return Err("owned loopback proxy required".into()); }
            let mut client = liter_llm::ClientConfig::new(key.clone());
            client.base_url = Some(endpoint.clone());
            client.load_env = false;
            client.max_retries = 0;
            client.max_response_bytes = Some(2_097_152);
            client.timeout = Duration::from_secs(90);
            client.redact_base_url = true;
            client.transport = client.transport.with_redirects_disabled(true);
            let driver = Arc::new(LiterLlmDriver::new(client, model.clone(), Some(false))?);
            let provider: ProviderConfig = serde_json::from_value(json!({"id":"openai", "base_url":endpoint,"api_key":key,"protocol":"chat","default_model":model,"models":[{"id":model,"context_window":context_window,"max_output_tokens":output_cap}],"enabled":true}))?;
            let llm = LlmConfig { model: format!("openai/{model}"), base_url: Some(endpoint), api_key: Some(key), protocol: "chat".into(), ..Default::default() };
            let governance = GovernanceEngine::load_from_dir(field(&request,"policy_path")?).await?;
            let store = Arc::new(SurrealDbProvider::new(&format!("surrealkv://{store_path}"), None, None, Some("ios_acceptance"), Some("runtime")).await?);
            // Only the checked-in exact graph spawn/wait Cedar policy is installed.
            let built = Runtime::builder().llm_config(llm).local_provider(driver, provider)
                .persistence(store.clone()).governance_engine(Arc::new(governance))
                .host_tool_admission(Arc::new(StandaloneToolAdmissionPort::new(field(&request,"epoch_id")?)))
                .seed_defaults(true).build().await;
            let runtime = match built { Ok(runtime) => runtime, Err(error) => { let _ = store.close_scope().await; return Err(error.into()); } };
            let expected = Some(("openai".to_owned(), model.clone()));
            if runtime.provider_registry().default_model().await != expected || runtime.inner().llm_config().model != format!("openai/{model}") {
                let _ = runtime.inner().shutdown().await;
                let _ = store.close_scope().await;
                return Err("embedded provider model registration mismatch".into());
            }
            Ok((runtime, store))
        });
        let (runtime, store) = result?;
        *owner = Some(Host { executor, runtime, store, receivers: BTreeMap::new() });
        return Ok(json!({"opened":true,"persistence":"surrealkv","driver":"LiterLlmDriver","server":false,"provider_id":"openai","wire_model":field(&request,"model")?,"runtime_model":format!("openai/{}",field(&request,"model")?)}));
    }
    let host = owner.as_mut().ok_or("host not open")?;
    let runtime = host.runtime.clone();
    let store = host.store.clone();
    let receivers = &mut host.receivers;
    let result = host.executor.block_on(AssertUnwindSafe(async {
        match op {
            "agents" => Ok(serde_json::to_value(runtime.list_agents().await?)?),
            "agent" => Ok(serde_json::to_value(runtime.get_agent(field(&request,"id")?).await?)?),
            "upsert_agent" => { runtime.upsert_agent(serde_json::from_value::<AgentArtifact>(request["agent"].clone())?).await?; Ok(json!({"saved":true})) }
            "delete_agent" => Ok(json!({"refused":runtime.delete_agent(field(&request,"id")?).await.is_err()})),
            "get_setting" => Ok(json!({"value":runtime.get_setting(field(&request,"key")?).await?})),
            "set_setting" => { runtime.set_setting(field(&request,"key")?,request["value"].clone()).await?; Ok(json!({"saved":true})) }
            "policy" => { runtime.save_conversation_policy(field(&request,"conversation")?,serde_json::from_value::<RunPolicy>(request["policy"].clone())?).await?; Ok(json!({"saved":true})) }
            "effective" => Ok(serde_json::to_value(runtime.effective_config(field(&request,"conversation")?).await?)?),
            "skill" => {
                let skill = serde_json::from_value::<Skill>(request["skill"].clone())?;
                runtime.save_skill(&skill, &[]).await?;
                let refreshed = runtime.skill_service().refresh().await?;
                if !refreshed.iter().any(|loaded| loaded.skill_id == skill.skill_id && loaded.prompt_overlay == skill.prompt_overlay) { return Err("saved skill not loaded".into()); }
                Ok(json!({"saved":true}))
            }
            "start" => {
                let id = runtime.start_agent_run(field(&request,"agent_id")?, field(&request,"prompt")?, Some(field(&request,"conversation")?.into()), None, vec![]).await?;
                let receiver = runtime.subscribe(&id).await?;
                receivers.insert(id.clone(), receiver);
                Ok(json!({"run_id":id}))
            }
            "drain" => {
                let id = field(&request,"run_id")?;
                let receiver = receivers.get_mut(id).ok_or("missing live subscription")?;
                let mut events = runtime.history_since(id,None).await?;
                let mut terminal = events.iter().any(|e| matches!(serde_json::to_value(&e.event).ok().and_then(|v|v["type"].as_str().map(str::to_owned)).as_deref(),Some("RunDone"|"Cancelled"|"Error")));
                let mut bytes = serde_json::to_vec(&events.iter().map(event_value).collect::<Result<Vec<_>>>()?)?.len();
                if events.len() > 8192 || bytes > 2_097_152 { return Err("replay bound exceeded".into()); }
                let deadline = tokio::time::Instant::now() + Duration::from_secs(120);
                while !terminal {
                    let item = tokio::time::timeout_at(deadline,receiver.recv()).await??;
                    if events.iter().any(|e|e.id==item.id) { continue; }
                    let value = event_value(&item)?;
                    bytes += serde_json::to_vec(&value)?.len();
                    if events.len() >= 8192 || bytes > 2_097_152 { return Err("event bound exceeded".into()); }
                    terminal = matches!(value["event"]["type"].as_str(),Some("RunDone"|"Cancelled"|"Error"));
                    events.push(item);
                }
                Ok(Value::Array(events.iter().map(event_value).collect::<Result<Vec<_>>>()?))
            }
            "cancel" => Ok(json!({"acknowledged":runtime.cancel_run(field(&request,"run_id")?).await})),
            "run" => Ok(serde_json::to_value(runtime.run_manager().get_run(field(&request,"run_id")?).await)?),
            "history" => {
                use universal_agent_runtime::uar::persistence::PersistenceLayer;
                Ok(serde_json::to_value(store.load_run_history_events(field(&request,"run_id")?).await?)?)
            }
            "shutdown" => {
                let report = runtime.inner().shutdown().await;
                let store_report = store.close_scope().await;
                Ok(json!({"runtime_released":report.released,"runtime_failures":report.failures,"unresolved":report.unresolved_effects,
                    "store_released":store_report.scope_released,"engine_acknowledged":store_report.engine_close_acknowledged,"store_failure":store_report.failure}))
            }
            _ => Err("unknown operation".into()),
        }
    }).catch_unwind()).map_err(|_| "async operation panicked")?;
    if op == "shutdown" && result.is_ok() { *owner = None; }
    result
}

/// `request` must point to a valid NUL-terminated UTF-8 JSON string until return.
/// Calls are serialized. Returned allocation must be released with uar_ios_free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uar_ios_call(request: *const c_char) -> *mut c_char {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Value> {
        if request.is_null() { return Err("null input".into()); }
        // SAFETY: the Swift bridge supplies a live withCString allocation.
        let bytes = unsafe { CStr::from_ptr(request) }.to_bytes();
        if bytes.len() > 1_048_576 { return Err("input bound exceeded".into()); }
        invoke(serde_json::from_slice(bytes)?)
    }));
    // Never serialize provider errors: they can contain credentials or endpoint data.
    let value = match result { Ok(Ok(value)) => json!({"ok":true,"result":value}), Ok(Err(_)) => json!({"ok":false,"code":"operation_failed"}), Err(_) => json!({"ok":false,"code":"ffi_panic_contained"}) };
    match CString::new(value.to_string()) { Ok(output) => output.into_raw(), Err(_) => std::ptr::null_mut() }
}
/// `value` must be an unreleased pointer returned by uar_ios_call, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uar_ios_free(value: *mut c_char) {
    if !value.is_null() { drop(unsafe { CString::from_raw(value) }); }
}
