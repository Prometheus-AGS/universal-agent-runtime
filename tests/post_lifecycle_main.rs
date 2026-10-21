//! Candidate-bound main-process boundary driver. Not a mock server or a new
//! certification campaign. Invoked only by the separately approved packet.
//!
//! Missing fixture operations are BLOCKED, never ignored tests or synthetic PASS.
//! The manifest is reviewed after the whole implementation inventory is complete.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use futures::StreamExt;
#[cfg(feature = "surreal-backend")]
#[path = "support/post_lifecycle_store.rs"]
mod post_lifecycle_store;
#[cfg(feature = "surreal-backend")]
use universal_agent_runtime::uar::persistence::{PersistenceLayer,
    providers::surreal::SurrealDbProvider,
    accounting::{CallAdmission, CallIdentity, CallUsage, ScopeBinding, ScopeIdentity, ScopeLimits}};

#[path = "support/post_lifecycle_count.rs"]
mod post_lifecycle_count;

const IDS: [&str; 10] = ["LIAB-01", "LIAB-02", "ERROR-01", "ERROR-02", "COUNT-01",
    "HISTORY-01", "KB-01", "CAS-01", "BTHREE-01", "IDENTITY-01"];

type Result<T> = std::result::Result<T, String>;

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

fn save(path: &Path, value: &Value) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|_| "receipt_create_failed".to_owned())?;
    file.write_all(value.to_string().as_bytes()).map_err(|_| "receipt_write_failed".to_owned())
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value[name].as_str().ok_or_else(|| format!("missing_field:{name}"))
}

fn expand(value: &Value, variables: &BTreeMap<String, Value>) -> Result<Value> {
    match value {
        Value::String(text) => {
            if let Some(name) = text.strip_prefix('$') {
                if !name.contains('/') && !name.contains(' ') {
                    return variables.get(name).cloned().ok_or_else(|| format!("missing_capture:{name}"));
                }
            }
            let mut text = text.clone();
            for (name, value) in variables {
                if let Some(value) = value.as_str() { text = text.replace(&format!("{{{name}}}"), value); }
            }
            Ok(Value::String(text))
        }
        Value::Array(items) => items.iter().map(|item| expand(item, variables)).collect(),
        Value::Object(items) => items.iter().map(|(key, value)| Ok((key.clone(), expand(value, variables)?))).collect(),
        _ => Ok(value.clone()),
    }
}

fn contains(value: &Value, expected: &Value) -> bool {
    if value == expected { return true; }
    match value {
        Value::Array(items) => items.iter().any(|item| contains(item, expected)),
        Value::Object(items) => items.values().any(|item| contains(item, expected)),
        _ => false,
    }
}

struct OwnedProcess {
    child: Child,
    launch: usize,
}

struct Boundary {
    manifest: Value,
    evidence: PathBuf,
    client: reqwest::Client,
    processes: BTreeMap<String, OwnedProcess>,
    launches: BTreeMap<String, usize>,
    variables: BTreeMap<String, Value>,
    events: Vec<Value>,
    #[cfg(feature = "surreal-backend")]
    store: Option<SurrealDbProvider>,
    owned_rows: Vec<Value>,
    schema_obligations: Vec<Value>,
    #[cfg(feature = "surreal-backend")]
    accounting_calls: Vec<CallIdentity>,
    #[cfg(feature = "surreal-backend")]
    accounting_scopes: Vec<ScopeIdentity>,
}

impl Boundary {
    fn launch(&mut self, name: &str) -> Result<()> {
        if self.processes.contains_key(name) { return Err("process_already_owned".into()); }
        let spec = &self.manifest["processes"][name];
        let argv: Vec<String> = spec["argv"].as_array().ok_or("process_argv_missing")?
            .iter().map(|arg| arg.as_str().map(str::to_owned).ok_or("invalid_process_arg".to_owned())).collect::<Result<_>>()?;
        if argv.is_empty() { return Err("process_argv_empty".into()); }
        let launch = self.launches.get(name).copied().unwrap_or(0) + 1;
        let max = spec["maxStarts"].as_u64().ok_or("process_start_budget_missing")? as usize;
        if launch > max { return Err("process_start_budget_exhausted".into()); }
        let executable = Path::new(&argv[0]);
        let bytes = fs::read(executable).map_err(|_| "process_executable_unreadable")?;
        let actual_hash = hash(&bytes);
        if name == "main" {
            let built = std::env::var("UAR_BOUNDARY_BUILT_SHA256").map_err(|_| "built_hash_missing")?;
            let candidate = std::env::var("UAR_MAIN_CANDIDATE").map_err(|_| "candidate_path_missing")?;
            if argv[0] != candidate || actual_hash != built { return Err("main_build_boot_identity_mismatch".into()); }
        } else if spec["sha256"].as_str() != Some(actual_hash.as_str()) {
            return Err("peer_identity_mismatch".into());
        }
        let log_dir = self.evidence.join("raw").join(format!("{name}-{launch}"));
        fs::create_dir(&log_dir).map_err(|_| "process_log_directory_failed")?;
        let stdout = OpenOptions::new().write(true).create_new(true).open(log_dir.join("stdout.log")).map_err(|_| "process_log_failed")?;
        let stderr = OpenOptions::new().write(true).create_new(true).open(log_dir.join("stderr.log")).map_err(|_| "process_log_failed")?;
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]).current_dir(field(&self.manifest, "projectRoot")?)
            .stdin(Stdio::null()).stdout(stdout).stderr(stderr);
        if let Some(environment) = spec["environment"].as_object() {
            for (key, value) in environment {
                if ["TOKEN", "SECRET", "PASSWORD", "API_KEY"].iter().any(|term| key.to_uppercase().contains(term)) {
                    return Err("secret_literal_environment_forbidden_use_existing_inheritance".into());
                }
                command.env(key, value.as_str().ok_or("invalid_process_environment")?);
            }
        }
        let child = command.spawn().map_err(|_| "process_launch_failed")?;
        self.events.push(json!({"kind":"launch", "process":name,"launch":launch,"pid":child.id(),"bootSha256":actual_hash}));
        self.launches.insert(name.to_owned(), launch);
        self.processes.insert(name.to_owned(), OwnedProcess { child, launch });
        Ok(())
    }

    fn stop(&mut self, name: &str) -> Result<()> {
        let mut process = self.processes.remove(name).ok_or("process_not_owned")?;
        let pid = process.child.id();
        // This exact control argv must be present in the reviewed manifest.
        let signal = self.manifest["signalArgv"].as_array().ok_or("signal_argv_missing")?;
        if signal != &vec![json!("/bin/kill"), json!("-TERM"), json!("{ownedPid}")] {
            self.processes.insert(name.to_owned(), process);
            return Err("signal_argv_not_reviewed".into());
        }
        let signal_status = match Command::new("/bin/kill").args(["-TERM", &pid.to_string()])
            .stdout(Stdio::null()).stderr(Stdio::null()).status() {
                Ok(status) => status,
                Err(_) => { self.processes.insert(name.to_owned(), process); return Err("signal_failed".into()); }
            };
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            match process.child.try_wait() {
                Ok(Some(status)) => {
                    self.events.push(json!({"kind":"exit","process":name,"launch":process.launch,
                        "signalExit":signal_status.code(),"exitCode":status.code(),"success":status.success()}));
                    return if status.success() { Ok(()) } else { Err("process_nonzero_teardown".into()) };
                }
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(25)),
                _ => {
                    let _ = process.child.kill();
                    let exit = process.child.wait().ok().and_then(|status| status.code());
                    self.events.push(json!({"kind":"exit","process":name,"launch":process.launch,
                        "exitCode":exit,"outcome":"BLOCKED","reason":"graceful_exit_unconfirmed"}));
                    return Err("graceful_exit_unconfirmed".into());
                }
            }
        }
    }

    fn request_builder(&self, step: &Value) -> Result<reqwest::RequestBuilder> {
        let step = expand(step, &self.variables)?;
        let base_key = if step["target"].as_str() == Some("peer") { "peerBaseUrl" } else { "baseUrl" };
        if base_key == "peerBaseUrl" && !self.processes.contains_key("peer") { return Err("peer_not_owned".into()); }
        let base = field(&self.manifest, base_key)?;
        if !base.starts_with("http://127.0.0.1:") { return Err("non_loopback_fixture_forbidden".into()); }
        let path = field(&step, "path")?;
        if !path.starts_with('/') || path.starts_with("//") { return Err("invalid_relative_path".into()); }
        let method = reqwest::Method::from_bytes(field(&step, "method")?.as_bytes()).map_err(|_| "invalid_method")?;
        let mut request = self.client.request(method, format!("{base}{path}"));
        if let Some(headers) = step["headers"].as_object() {
            for (key, value) in headers {
                // Manifest headers contain environment variable NAMES, never credential values.
                let env_name = value.as_str().ok_or("header_environment_name_missing")?;
                let env_value = std::env::var(env_name).map_err(|_| "existing_header_environment_unavailable")?;
                request = request.header(key, env_value);
            }
        }
        if let Some(body) = step.get("json") { request = request.json(body); }
        if let Some(upload) = step.get("upload") {
            let file = Path::new(field(upload, "path")?);
            let bytes = fs::read(file).map_err(|_| "upload_fixture_unreadable")?;
            if hash(&bytes) != field(upload, "sha256")? { return Err("upload_fixture_changed".into()); }
            let part = reqwest::multipart::Part::bytes(bytes).file_name(field(upload, "filename")?.to_owned())
                .mime_str("text/plain").map_err(|_| "multipart_failed")?;
            request = request.multipart(reqwest::multipart::Form::new().part("file", part));
        }
        Ok(request)
    }

    async fn request(&self, step: &Value) -> Result<(u16, String, Value)> {
        let response = self.request_builder(step)?.send().await.map_err(|error| if error.is_timeout() {"request_timeout"} else {"request_transport_failed"})?;
        let status = response.status().as_u16();
        let text = if let Some(allowed)=step["approveTools"].as_array() {
            let authorized=self.manifest["authorizedToolApprovals"].as_array().ok_or("fixture_tool_approvals_not_reviewed")?;
            if allowed.iter().any(|tool| !authorized.contains(tool)) { return Err("fixture_tool_approval_outside_review".into()); }
            let mut stream=response.bytes_stream(); let mut bytes=Vec::new(); let mut approved=BTreeSet::new();
            while let Some(chunk)=stream.next().await {
                bytes.extend_from_slice(&chunk.map_err(|_|"stream_body_failed")?);
                let text=String::from_utf8_lossy(&bytes);
                for packet in text.lines().filter_map(|line|line.strip_prefix("data:")).filter_map(|data|serde_json::from_str::<Value>(data.trim()).ok()) {
                    if packet["phase"]!="approval_required" { continue; }
                    let name=field(&packet,"name")?;
                    if !allowed.iter().any(|tool|tool.as_str()==Some(name)) { return Err("unexpected_fixture_tool_approval".into()); }
                    let approval=field(&packet,"approval_id")?;
                    if !approved.insert(approval.to_owned()) { continue; }
                    let run=field(&packet,"request_id")?;
                    let decision=json!({"method":"POST","path":format!("/api/uar/runs/{run}/tool-approval"),
                        "headers":step["headers"],"json":{"approved":true,"approval_id":approval}});
                    let status=self.request_builder(&decision)?.send().await.map_err(|_|"fixture_approval_transport_failed")?.status();
                    save(&self.evidence.join("raw").join(format!("tool-approval-{}.json",hash(approval.as_bytes()))),
                        &json!({"tool":name,"approvalIdSha256":hash(approval.as_bytes()),"status":status.as_u16()}))?;
                    if status.as_u16()!=200 { return Err("fixture_approval_rejected".into()); }
                }
                if bytes.len()>4*1024*1024 { return Err("fixture_stream_observation_bound_exceeded".into()); }
            }
            String::from_utf8(bytes).map_err(|_|"fixture_stream_not_utf8")?
        } else { response.text().await.map_err(|_| "response_body_failed")? };
        let value = serde_json::from_str(&text).unwrap_or(Value::Null);
        Ok((status, text, value))
    }

    #[cfg(feature = "surreal-backend")]
    async fn open_store(&mut self) -> Result<()> {
        if self.store.is_some() { return Ok(()); }
        if self.manifest["storeFixtureApprovalRef"].as_str().is_none_or(str::is_empty) {
            return Err("isolated_store_permission_missing".into());
        }
        let spec = &self.manifest["store"];
        let endpoint = field(spec, "endpoint")?;
        if !endpoint.starts_with("ws://127.0.0.1:") || endpoint.contains('@') {
            return Err("unchanged_loopback_store_endpoint_required".into());
        }
        let namespace = field(spec, "namespace")?;
        let database = field(spec, "database")?;
        if !namespace.starts_with("uar_post_lifecycle_") || !database.starts_with("uar_post_lifecycle_") {
            return Err("dedicated_fixture_namespace_required".into());
        }
        let user = std::env::var(field(spec, "userEnv")?).map_err(|_| "existing_store_user_unavailable")?;
        let password = std::env::var(field(spec, "passwordEnv")?).map_err(|_| "existing_store_password_unavailable")?;
        let provider = SurrealDbProvider::new(endpoint, Some(&user), Some(&password), Some(namespace), Some(database))
            .await.map_err(|_| "store_connect_failed")?;
        if !provider.durable_store_eligibility().permits_durable_work() {
            let report = provider.close_scope().await;
            self.events.push(json!({"kind":"store_close","scopeReleased":report.scope_released}));
            return Err("store_durability_unproven_no_mode_substitution".into());
        }
        self.store = Some(provider);
        Ok(())
    }

    #[cfg(feature = "surreal-backend")]
    async fn accounting_fixture(&mut self, step: &Value) -> Result<()> {
        self.open_store().await?;
        let step = expand(step, &self.variables)?;
        let store = self.store.as_ref().ok_or("store_missing")?;
        let identity = CallIdentity { owner: field(&step, "owner")?.to_owned(),
            run_id: field(&step, "runId")?.to_owned(), call_id: field(&step, "callId")?.to_owned(), attempt: 1 };
        let scope_identity = ScopeIdentity { owner: identity.owner.clone(), scope: "session".into(),
            scope_id: field(&step, "scopeId")?.to_owned() };
        let scope = store.ensure_accounting_scope(&scope_identity, &ScopeLimits { cost_usd: Some(2.0), tokens: Some(100) })
            .await.map_err(|_| "accounting_scope_failed")?;
        let request = CallAdmission { identity: identity.clone(), model: "fixture/provider".into(),
            scopes: vec![ScopeBinding { identity: scope_identity, generation: scope.generation }],
            reserved_cost_usd: Some(step["reserve"].as_f64().ok_or("reserve_missing")?),
            reserved_tokens: Some(10), source_reservation: None };
        let admitted = store.admit_accounting_call(&request).await;
        if step["expectAdmitted"].as_bool() != Some(admitted.is_ok()) { return Err("accounting_admission_mismatch".into()); }
        if !self.accounting_scopes.contains(&request.scopes[0].identity) {
            self.accounting_scopes.push(request.scopes[0].identity.clone());
        }
        if admitted.is_ok() && !self.accounting_calls.contains(&identity) { self.accounting_calls.push(identity.clone()); }
        if admitted.is_ok() && step["observeNonterminal"].as_bool() == Some(true) {
            store.record_accounting_usage(&CallUsage { identity, cost_usd: Some(0.25), tokens: 2,
                terminal: false, evidence: None }).await.map_err(|_| "nonterminal_observation_failed")?;
        }
        self.events.push(json!({"kind":"accounting_fixture","admitted":admitted.is_ok(),
            "scopeGeneration":scope.generation,"label":"direct real-store collaborator; not provider-dispatch proof"}));
        Ok(())
    }

    #[cfg(feature = "surreal-backend")]
    async fn cas_fixture(&mut self, step: &Value) -> Result<()> {
        let step = expand(step, &self.variables)?;
        let mut manifest = self.manifest.clone();
        manifest["casFixture"] = json!({"owner":field(&step,"owner")?,"fixtureId":field(&step,"fixtureId")?,"unrelatedConflictSchedule":true});
        let report = post_lifecycle_store::cas_discriminator(&manifest, &self.evidence).await?;
        if let Some(rows) = report["ownedRows"].as_array() { self.owned_rows.extend(rows.iter().cloned()); }
        if let Some(events)=report["retainedSchemaEvents"].as_array() { self.schema_obligations.extend(events.iter().cloned()); }
        self.events.push(report.clone());
        if report["outcome"] == "PASS" { Ok(()) } else if report["outcome"] == "FAIL" {
            Err("assertion_mismatch:cas_discriminator_see_actual_report".into())
        } else { Err("cas_discriminator_incomplete_see_actual_report".into()) }
    }

    fn observe_response(&mut self, step: &Value, status: u16, text: &str, value: &Value) -> Result<()> {
        self.events.push(json!({"kind":"response","status":status,"bodySha256":hash(text.as_bytes()),
            "bodyBytes":text.len(),"control":step["control"]}));
        if step["status"].as_u64() != Some(u64::from(status)) { return Err("http_status_mismatch".into()); }
        if let Some(captures) = step["capture"].as_object() {
            for (name,pointer) in captures {
                let captured = value.pointer(pointer.as_str().ok_or("capture_pointer_missing")?).ok_or("capture_value_missing")?;
                if captured.is_null() { return Err("capture_value_null".into()); }
                self.variables.insert(name.clone(),captured.clone());
            }
        }
        let assertions = step["assert"].as_array().ok_or("assertions_missing")?;
        if assertions.is_empty() { return Err("assertions_empty".into()); }
        for assertion in assertions {
            let assertion = expand(assertion,&self.variables)?;
            let kind = field(&assertion,"kind")?;
            let good = match kind {
                "json_equals" => value.pointer(field(&assertion,"pointer")?) == assertion.get("value"),
                "json_array_length" => value.pointer(field(&assertion,"pointer")?).and_then(Value::as_array)
                    .is_some_and(|items| Some(items.len() as u64) == assertion["length"].as_u64()),
                "contains_json" => contains(value,&assertion["value"]),
                "excludes_json" => !contains(value,&assertion["value"]),
                "text_count" => { let needle=field(&assertion,"text")?;
                    !needle.is_empty() && Some(text.matches(needle).count() as u64) == assertion["count"].as_u64() },
                "sse_object_count" => {
                    let expected=assertion["match"].as_object().ok_or("sse_match_missing")?;
                    let count=text.lines().filter_map(|line| line.strip_prefix("data:")).filter_map(|data|
                        serde_json::from_str::<Value>(data.trim()).ok()).filter(|packet|
                        expected.iter().all(|(pointer,wanted)| packet.pointer(pointer)==Some(wanted))).count();
                    Some(count as u64)==assertion["count"].as_u64()
                }
                _ => return Err("unsupported_assertion".into()),
            };
            self.events.push(json!({"kind":"assertion","assertionKind":kind,"control":step["control"],"matched":good}));
            if !good { return Err(format!("assertion_mismatch:{kind}")); }
        }
        Ok(())
    }

    async fn step(&mut self, step: &Value) -> Result<()> {
        let action = field(step, "action")?;
        match action {
            #[cfg(feature = "surreal-backend")]
            "accounting_fixture" => self.accounting_fixture(step).await,
            #[cfg(feature = "surreal-backend")]
            "cas_fixture" => self.cas_fixture(step).await,
            "launch" => self.launch(field(step, "process")?),
            "stop" => self.stop(field(step, "process")?),
            "prepare_artifact" => {
                let step=expand(step,&self.variables)?;
                let mut artifact=serde_json::to_value(universal_agent_runtime::uar::defaults::default_agent()).map_err(|_|"default_artifact_failed")?;
                artifact["id"]=step["id"].clone();
                artifact["metadata"]["title"]=json!("Owned post-lifecycle fixture");
                artifact["memory"]["kb"]["enabled"]=json!(false);
                if let Some(tools)=step.get("tools") { artifact["policy"]["tools"]["allow"]=tools.clone(); }
                if let Some(budgets)=step.get("budgets") { artifact["extensions"]["budgets"]=budgets.clone(); }
                if let Some(provider)=step.get("provider") { artifact["policy"]["provider"]["default"]=provider.clone(); }
                artifact["extensions"]["uar.run_policy"]=json!({"version":1,"tool_approval":"auto"});
                self.variables.insert(field(&step,"capture")?.to_owned(),artifact);
                Ok(())
            }
            "request" => {
                let (status,text,value)=self.request(step).await?;
                self.observe_response(step,status,&text,&value)
            }
            "concurrent_requests" => {
                let requests=step["requests"].as_array().ok_or("concurrent_requests_missing")?;
                if requests.len()!=2 { return Err("concurrent_request_count_must_be_two".into()); }
                let (first,second)=tokio::join!(self.request(&requests[0]),self.request(&requests[1]));
                let first=first?; let second=second?;
                let one=self.observe_response(&requests[0],first.0,&first.1,&first.2);
                let two=self.observe_response(&requests[1],second.0,&second.1,&second.2);
                one.and(two)
            }
            "disconnect_stream" => {
                let response=self.request_builder(&step["request"])?.send().await.map_err(|_|"disconnect_open_failed")?;
                if response.status().as_u16()!=200 { return Err("disconnect_status_mismatch".into()); }
                let mut stream=response.bytes_stream();
                let first=tokio::time::timeout(Duration::from_secs(30),stream.next()).await.map_err(|_|"disconnect_prefix_timeout")?
                    .ok_or("disconnect_prefix_absent")?.map_err(|_|"disconnect_prefix_transport_failure")?;
                self.events.push(json!({"kind":"client_disconnect","prefixSha256":hash(&first),"prefixBytes":first.len(),
                    "note":"response dropped after actual prefix; no unseen server terminal outcome inferred"}));
                drop(stream);
                Ok(())
            }
            "poll" => {
                let attempts = step["attempts"].as_u64().ok_or("poll_attempts_missing")?;
                if attempts == 0 || attempts > 30 { return Err("poll_attempts_exceed_bound".into()); }
                let expected=expand(&step["value"],&self.variables)?;
                let pointer=expand(&step["pointer"],&self.variables)?;
                let mut last = "poll_condition_unobserved".to_owned();
                for _ in 0..attempts {
                    let (status, _, value) = self.request(&step["request"]).await?;
                    self.events.push(json!({"kind":"poll","status":status,"control":step["control"],
                        "observedState":value["status"],"errorMessageSha256":value["error_message"].as_str().map(|text|hash(text.as_bytes()))}));
                    if value["status"]=="failed" { return Err("assertion_mismatch:typed_ingestion_failure_see_poll_receipt".into()); }
                    if status == step["status"].as_u64().ok_or("poll_status_missing")? as u16
                        && value.pointer(pointer.as_str().ok_or("poll_pointer_missing")?) == Some(&expected) { return Ok(()); }
                    last = "poll_condition_unobserved".into();
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
                Err(last)
            }
            "wait" => {
                let millis = step["milliseconds"].as_u64().ok_or("wait_duration_missing")?;
                if millis > 30_000 { return Err("wait_exceeds_bound".into()); }
                tokio::time::sleep(Duration::from_millis(millis)).await;
                Ok(())
            }
            "required_unimplemented_control" => Err("required_scenario_control_not_authored".into()),
            _ => Err(format!("unsupported_action:{action}")),
        }
    }

    fn cleanup(&mut self) -> Value {
        let names: Vec<String> = self.processes.keys().cloned().collect();
        let mut failures = Vec::new();
        for name in names { if let Err(error) = self.stop(&name) { failures.push(json!({"process":name,"reason":error})); } }
        json!({"outcome":if failures.is_empty(){"PASS"}else{"BLOCKED"},"failures":failures,"events":self.events})
    }
}

impl Drop for Boundary {
    fn drop(&mut self) {
        // Emergency cancellation cannot earn cleanup credit. Normal path awaits exits.
        for process in self.processes.values_mut() { let _ = process.child.kill(); let _ = process.child.wait(); }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn post_lifecycle_main_boundary() {
    let manifest_path = std::env::var("UAR_BOUNDARY_MANIFEST").expect("approved manifest required; do not run stand-alone");
    let evidence = PathBuf::from(std::env::var("UAR_BOUNDARY_EVIDENCE").expect("runner evidence root required"));
    let manifest: Value = serde_json::from_slice(&fs::read(manifest_path).expect("read reviewed manifest")).expect("manifest JSON");
    assert_eq!(manifest["verificationApprovalRef"].as_str().map(str::is_empty), Some(false), "verification approval missing");
    let client = reqwest::Client::builder().timeout(Duration::from_secs(60)).redirect(reqwest::redirect::Policy::none())
        .build().expect("HTTP client");
    let variables: BTreeMap<String, Value> = manifest["fixtureVariables"].as_object()
        .map(|values| values.iter().map(|(key, value)| (key.clone(), value.clone())).collect()).unwrap_or_default();
    let mut boundary = Boundary { manifest, evidence: evidence.clone(), client,
        processes: BTreeMap::new(), launches: BTreeMap::new(), variables, events: Vec::new(), owned_rows:Vec::new(), schema_obligations:Vec::new(),
        #[cfg(feature = "surreal-backend")]
        accounting_calls:Vec::new(),
        #[cfg(feature = "surreal-backend")]
        accounting_scopes:Vec::new(),
        #[cfg(feature = "surreal-backend")]
        store: None };
    let mut cases = serde_json::Map::new();
    let setup_steps = boundary.manifest["setupSteps"].as_array().cloned().unwrap_or_default();
    let mut setup_error = None;
    for step in &setup_steps {
        if let Err(error) = boundary.step(step).await { setup_error = Some(error); break; }
    }
    for id in IDS {
        if let Some(error) = &setup_error {
            cases.insert(id.to_owned(), json!({"outcome":"BLOCKED","reason":error}));
            continue;
        }
        if id == "BTHREE-01" {
            let disposition = boundary.manifest["historicalB3"].clone();
            cases.insert(id.to_owned(), if disposition["located"].as_bool() == Some(false) {
                json!({"outcome":"NOT_EXERCISED","reason":"specific_original_archive_absent","disposition":disposition})
            } else { json!({"outcome":"BLOCKED","reason":"historical_archive_needs_candidate_bound_review"}) });
            continue;
        }
        if id == "COUNT-01" {
            // Owner-approved evidence split: the existing trusted Rust builder
            // is exercised separately. No standalone-main fit claim follows.
            let result = post_lifecycle_count::exercise_count_boundary(&evidence).await;
            let receipt = match result {
                Ok(receipt) if receipt["evidenceScope"] == "trusted_rust_builder"
                    && receipt["standaloneMainExactFit"] == "UNSUPPORTED" => receipt,
                Ok(_) => json!({"outcome":"BLOCKED","reason":"count_evidence_scope_mismatch"}),
                Err(error) => json!({"outcome":"BLOCKED","reason":error.to_string(),
                    "evidenceScope":"trusted_rust_builder","standaloneMainExactFit":"UNSUPPORTED"}),
            };
            cases.insert(id.to_owned(), receipt);
            continue;
        }
        if id == "IDENTITY-01" {
            let launches = boundary.events.iter().filter(|event| event["kind"] == "launch" && event["process"] == "main").count();
            cases.insert(id.to_owned(), json!({"outcome":if launches > 0 {"PASS"}else{"BLOCKED"},
                "observedMainStarts":launches,"note":"overall PASS additionally requires successful awaited teardown and runner input rehash"}));
            continue;
        }
        let recipe = boundary.manifest["scenarios"][id].clone();
        let mut outcome = "PASS";
        let mut reason = String::new();
        let steps = recipe["steps"].as_array();
        let controls = recipe["requiredControls"].as_array();
        if steps.is_none_or(Vec::is_empty) || controls.is_none_or(Vec::is_empty) {
            outcome = "BLOCKED"; reason = "reviewed_scenario_steps_or_controls_missing".into();
        } else {
            let mut observed = Vec::new();
            for step in steps.into_iter().flatten() {
                if let Err(error) = boundary.step(step).await {
                    outcome = if error.contains("mismatch") { "FAIL" } else { "BLOCKED" };
                    reason = error;
                    break;
                }
                if let Some(control) = step["control"].as_str() { observed.push(control.to_owned()); }
            }
            if outcome == "PASS" && controls.into_iter().flatten().any(|name| !observed.iter().any(|seen| Some(seen.as_str()) == name.as_str())) {
                outcome = "BLOCKED"; reason = "required_control_not_observed".into();
            }
        }
        cases.insert(id.to_owned(), json!({"outcome":outcome,"reason":reason}));
    }
    let cleanup_steps=boundary.manifest["cleanupSteps"].as_array().cloned().unwrap_or_default();
    let mut cleanup_failures=Vec::new();
    for step in &cleanup_steps { if let Err(error)=boundary.step(step).await { cleanup_failures.push(json!(error)); } }
    let mut teardown = boundary.cleanup();
    #[cfg(feature = "surreal-backend")]
    if let Some(store)=boundary.store.take() {
        for identity in &boundary.accounting_calls {
            match store.list_accounting_calls(&identity.owner,&identity.run_id).await {
                Ok(rows)=>if let Some(row)=rows.iter().find(|row| row.admission.identity==*identity) {
                    match serde_json::to_string(row) {
                        Ok(data)=>boundary.owned_rows.push(json!({"table":"accounting_calls","key":identity.key(),
                            "match":{"owner_key":identity.owner,"run_id":identity.run_id,"data":data}})),
                        Err(_)=>cleanup_failures.push(json!("accounting_call_snapshot_failed")),
                    }
                },
                Err(_)=>cleanup_failures.push(json!("accounting_call_cleanup_read_failed")),
            }
        }
        for identity in &boundary.accounting_scopes {
            match store.get_accounting_scope(identity).await {
                Ok(Some(row))=>match serde_json::to_string(&row) {
                    Ok(data)=>boundary.owned_rows.push(json!({"table":"accounting_scopes","key":identity.key(),"match":{"data":data}})),
                    Err(_)=>cleanup_failures.push(json!("accounting_scope_snapshot_failed")),
                },
                _=>cleanup_failures.push(json!("accounting_scope_cleanup_read_failed")),
            }
        }
        let report=store.close_scope().await;
        teardown["storeScopeReleased"]=json!(report.scope_released);
        if !report.scope_released { cleanup_failures.push(json!("accounting_provider_scope_unreleased")); }
    }
    if !boundary.schema_obligations.is_empty() {
        cleanup_failures.push(json!("owned_schema_event_cleanup_unconfirmed"));
        teardown["retainedSchemaEvents"]=Value::Array(boundary.schema_obligations.clone());
    }
    #[cfg(feature = "surreal-backend")]
    {
        let mut run_ids: Vec<String>=boundary.variables.iter().filter(|(name,_)|
            name.starts_with("run_") || name.ends_with("_run")).filter_map(|(_,value)|value.as_str().map(str::to_owned)).collect();
        run_ids.sort(); run_ids.dedup();
        if !run_ids.is_empty() && !boundary.processes.contains_key("main") {
            match post_lifecycle_store::history_cleanup_rows(&boundary.manifest,&run_ids).await {
                Ok(rows)=>boundary.owned_rows.extend(rows),
                Err(reason)=>cleanup_failures.push(json!(reason)),
            }
        }
    }
    #[cfg(feature = "surreal-backend")]
    if !boundary.owned_rows.is_empty() {
        let mut manifest=boundary.manifest.clone();
        let mut rows=manifest["cleanupOwnedRows"].as_array().cloned().unwrap_or_default();
        rows.extend(boundary.owned_rows.iter().cloned());
        manifest["cleanupOwnedRows"]=Value::Array(rows);
        manifest["ownedMainStopped"]=json!(!boundary.processes.contains_key("main") && teardown["outcome"]=="PASS");
        match post_lifecycle_store::cleanup_rows(&manifest,&evidence).await {
            Ok(report)=>{ if report["outcome"]!="PASS" { cleanup_failures.push(json!("exact_row_cleanup_incomplete")); }
                teardown["rowCleanup"]=report; },
            Err(reason)=>cleanup_failures.push(json!(reason)),
        }
    }
    if !cleanup_failures.is_empty() { teardown["outcome"]=json!("BLOCKED"); }
    teardown["cleanupFailures"]=Value::Array(cleanup_failures);
    if let Some(processes) = boundary.manifest["processes"].as_object() {
        for (name, spec) in processes {
            if spec["expectedStarts"].as_u64() != Some(boundary.launches.get(name).copied().unwrap_or(0) as u64) {
                teardown["outcome"] = json!("BLOCKED");
                teardown["processCountMismatch"] = json!(true);
            }
        }
    }
    save(&evidence.join("teardown.json"), &teardown).expect("append-only teardown receipt");
    save(&evidence.join("cases.json"), &Value::Object(cases.clone())).expect("append-only case receipt");
    let passed = IDS.iter().all(|id| cases[*id]["outcome"] == "PASS" || (*id == "BTHREE-01" && cases[*id]["outcome"] == "NOT_EXERCISED"));
    assert!(passed && teardown["outcome"] == "PASS", "boundary incomplete; inspect retained per-case/teardown receipts");
}
