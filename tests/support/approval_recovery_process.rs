//! Dot07 uses the actual binary and independent API/readback child processes.

#[path = "fresh_process_resume_process.rs"]
mod resume;

use resume::{Database, Node, Peer, Seen, OWNER, TENANT, ISSUER, credentials, effects,
    private_dir, private_file, private_write, token};
use axum::{Json, Router, extract::State, response::{IntoResponse, Response}, routing::post};
use reqwest::Method;
use serde_json::{Value, json};
use std::{collections::BTreeSet, io::Write, path::{Path, PathBuf}, process::{Child, Command, Stdio},
    sync::Arc, time::Duration};
use universal_agent_runtime::uar::persistence::{PersistenceLayer,
    approval_decisions::{ApprovalRecord, ApprovalState}, checkpoint_resume::CheckpointResumeEnvelope};

const WAIT: Duration = Duration::from_secs(90);
const CASES: &[&str] = &["pending-restart", "race", "deny", "expiration", "revoked", "tightened",
    "binding", "abrupt", "unknown", "siblings", "legacy", "legacy-deny", "invalidation-outage"];

fn owner_key() -> String {
    format!("v1:k:issuer:i:{}:{}:t:{}:{}:s:{}:{}", ISSUER.len(), ISSUER, TENANT.len(), TENANT, OWNER.len(), OWNER)
}

async fn completion(State(seen): State<Arc<Seen>>, Json(body): Json<Value>) -> Response {
    seen.requests.lock().unwrap().push(body.clone());
    let mut log = std::fs::OpenOptions::new().append(true).open(&seen.log).unwrap();
    serde_json::to_writer(&mut log, &body).unwrap(); log.write_all(b"\n").unwrap(); log.sync_all().unwrap();
    let messages = body["messages"].as_array().unwrap();
    let input = messages.iter().rfind(|message| message["role"] == "user").unwrap()["content"].as_str().unwrap();
    let tool_results = messages.iter().filter(|message| message["role"] == "tool").count();
    let chunk = |delta: Value, finish: Value| format!("data: {}\n\n", json!({"id":"dot07-response",
        "object":"chat.completion.chunk","created":1,"model":"gpt-4o-mini",
        "choices":[{"index":0,"delta":delta,"finish_reason":finish}]}));
    let mut frames = String::new();
    if tool_results > 0 {
        assert_eq!(tool_results, if input == "siblings" { 2 } else { 1 }, "only complete real results reach model continuation");
        frames.push_str(&chunk(json!({"role":"assistant","content":"approval-recovery-complete"}), Value::Null));
        frames.push_str(&chunk(json!({}), json!("stop")));
    } else {
        let tool = body["tools"].as_array().unwrap().iter().find_map(|tool|
            tool["function"]["name"].as_str().filter(|name| name.ends_with("__effect"))).unwrap();
        let calls = (0..if input == "siblings" { 2 } else { 1 }).map(|index| json!({"index":index,
            "id":format!("dot07-{input}-{index}"),"type":"function",
            "function":{"name":tool,"arguments":json!({"mode":if input == "unknown" { "unknown" } else { "seed" }}).to_string()}})).collect::<Vec<_>>();
        frames.push_str(&chunk(json!({"role":"assistant","tool_calls":calls}), Value::Null));
        frames.push_str(&chunk(json!({}), json!("tool_calls")));
    }
    frames.push_str("data: [DONE]\n\n");
    ([("content-type","text/event-stream")], frames).into_response()
}

async fn peer(root: &Path) -> Peer {
    let log = root.join("provider-requests.jsonl"); private_write(&log, b"");
    let seen = Arc::new(Seen { log, ..Seen::default() });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/v1", listener.local_addr().unwrap());
    let app = Router::new().route("/v1/chat/completions", post(completion)).with_state(seen.clone());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
    Peer { url, seen, task }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ReadbackJob { database: Database, run: String, result: PathBuf, unbind: bool, reject_resolution: bool }

#[derive(serde::Serialize, serde::Deserialize)]
struct Readback {
    records: Vec<ApprovalRecord>, envelope: Option<CheckpointResumeEnvelope>,
    epoch_state: Option<String>, claims: Vec<Value>, executor_states: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ClientJob { base: String, path: String, body: Value, result: PathBuf, ready: PathBuf, barrier: PathBuf }

#[derive(serde::Serialize, serde::Deserialize)]
struct DeadlineJob { profile: String, database: Option<Database>, postgres_url: Option<String>, result: PathBuf }

fn deadline_record(approved: bool) -> ApprovalRecord {
    let now = chrono::Utc::now();
    ApprovalRecord { version: 1, issuer_id: "dot07-deadline-issuer".into(), challenge_id: uuid::Uuid::new_v4().to_string(),
        owner_key: owner_key(), workspace_id: None, root_run_id: uuid::Uuid::new_v4().to_string(), admission_id: None,
        admission_owner: universal_agent_runtime::uar::persistence::tool_admission::AdmissionOwner::UarRuntime,
        tool_call_id: format!("deadline-{approved}"), tool_name: "resume__effect".into(), created_at: now,
        expires_at: now + chrono::Duration::seconds(2), state: ApprovalState::Pending, decision: None,
        updated_at: now, continuation: None }
}

fn deadline_decision(record: &ApprovalRecord, approved: bool) -> ApprovalRecord {
    record.finish(if approved { ApprovalState::Approved } else { ApprovalState::Denied },
        Some(universal_agent_runtime::uar::persistence::approval_decisions::ApprovalDecision {
            decision_id: uuid::Uuid::new_v4().to_string(), actor: record.owner_key.clone(), approved,
            decided_at: chrono::Utc::now(),
        }))
}

async fn unbound_denial_control(store: &dyn PersistenceLayer) -> Value {
    let record = deadline_record(false); store.create_approval_record(&record).await.unwrap();
    let intent = deadline_decision(&record, false);
    assert!(store.transition_approval_record(&record, &intent).await.unwrap());
    let rows = store.list_approval_records(&record.owner_key, &record.root_run_id).await.unwrap();
    assert_eq!(rows.len(), 1); assert_eq!(rows[0].state, ApprovalState::Denied);
    assert_eq!(rows[0].decision.as_ref().unwrap().approved, false);
    json!({"challengeId":record.challenge_id,"approvedIntent":false,"continuation":null,
        "committedState":rows[0].state,"requestedTransitionWon":true})
}

async fn stale_deadline_intents(store: &dyn PersistenceLayer) -> Value {
    let mut evidence = vec![unbound_denial_control(store).await];
    for approved in [true, false] {
        let record = deadline_record(approved); store.create_approval_record(&record).await.unwrap();
        let intent = deadline_decision(&record, approved);
        assert!(intent.decision.as_ref().unwrap().decided_at < record.expires_at);
        let remaining = record.expires_at.signed_duration_since(chrono::Utc::now()).to_std().unwrap();
        tokio::time::sleep(remaining + Duration::from_millis(50)).await;
        assert!(!store.transition_approval_record(&record, &intent).await.unwrap(), "late intent cannot win its requested state");
        let rows = store.list_approval_records(&record.owner_key, &record.root_run_id).await.unwrap();
        assert_eq!(rows.len(), 1); assert_eq!(rows[0].state, ApprovalState::Expired); assert!(rows[0].decision.is_none());
        assert!(!store.transition_approval_record(&record, &intent).await.unwrap());
        evidence.push(json!({"challengeId":record.challenge_id,"approvedIntent":approved,"expiresAt":record.expires_at,
            "decidedAt":intent.decision.as_ref().unwrap().decided_at,"committedState":rows[0].state,"requestedTransitionWon":false}));
    }
    json!(evidence)
}

#[cfg(feature = "postgres-backend")]
async fn postgres_locked_deadlines(url: &str) -> Value {
    let store = Arc::new(universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider::new(url).await.unwrap());
    let mut evidence = vec![unbound_denial_control(store.as_ref()).await];
    for approved in [true, false] {
        let record = deadline_record(approved); store.create_approval_record(&record).await.unwrap();
        let intent = deadline_decision(&record, approved);
        assert!(intent.decision.as_ref().unwrap().decided_at < record.expires_at);
        let mut lock = store.get_pool().begin().await.unwrap();
        sqlx::query("SELECT data FROM approval_records WHERE owner_key = $1 AND issuer_id = $2 AND challenge_id = $3 FOR UPDATE")
            .bind(&record.owner_key).bind(&record.issuer_id).bind(&record.challenge_id).fetch_one(&mut *lock).await.unwrap();
        let writer = Arc::clone(&store); let before = record.clone(); let after = intent.clone();
        let pending = tokio::spawn(async move { writer.transition_approval_record(&before, &after).await });
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(!pending.is_finished(), "production CAS must wait for the held row lock");
        let remaining = record.expires_at.signed_duration_since(chrono::Utc::now()).to_std().unwrap();
        tokio::time::sleep(remaining + Duration::from_millis(50)).await;
        lock.rollback().await.unwrap();
        assert!(!pending.await.unwrap().unwrap(), "deadline after a lock wait cannot use transaction-start time");
        let rows = store.list_approval_records(&record.owner_key, &record.root_run_id).await.unwrap();
        assert_eq!(rows.len(), 1); assert_eq!(rows[0].state, ApprovalState::Expired); assert!(rows[0].decision.is_none());
        evidence.push(json!({"challengeId":record.challenge_id,"approvedIntent":approved,"expiresAt":record.expires_at,
            "decidedAt":intent.decision.as_ref().unwrap().decided_at,"committedState":rows[0].state,"rowLockReleasedAfterExpiry":true}));
    }
    json!(evidence)
}

/// Child modes are fixture I/O, not separate verification gates.
pub async fn run_child() -> bool {
    if let Some(path) = std::env::var_os("UAR_DOT07_DEADLINE_JOB") {
        let job: DeadlineJob = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let decisions = if job.profile == "postgres" {
            #[cfg(feature = "postgres-backend")]
            { postgres_locked_deadlines(job.postgres_url.as_deref().unwrap()).await }
            #[cfg(not(feature = "postgres-backend"))]
            { panic!("Postgres deadline qualification requires postgres-backend"); }
        } else {
            let store = job.database.as_ref().unwrap().connect().await;
            stale_deadline_intents(&store).await
        };
        private_write(&job.result, &serde_json::to_vec_pretty(&json!({"schemaVersion":1,"profile":job.profile,
            "case":"atomic-deadline","decisions":decisions,"effects":0,"verified":true,"genuineInference":false})).unwrap());
        return true;
    }
    if let Some(path) = std::env::var_os("UAR_DOT07_READBACK_JOB") {
        let job: ReadbackJob = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let db = job.database.connect().await;
        let mut records = db.list_approval_records(&owner_key(), &job.run).await.unwrap();
        if job.unbind {
            assert_eq!(records.len(), 1);
            records[0].continuation = None;
            db.client().query("UPDATE type::record('approval_records', $key) SET data = $data")
                .bind(("key", records[0].storage_key())).bind(("data", serde_json::to_string(&records[0]).unwrap()))
                .await.unwrap().check().unwrap();
        }
        if job.reject_resolution {
            db.client().query("DEFINE EVENT dot07_reject_resolution ON TABLE approval_records
                WHEN $after.state = 'resolved' THEN { THROW 'dot07_approval_write_unavailable'; };")
                .await.unwrap().check().unwrap();
        }
        let envelope = match records.first().and_then(|record| record.continuation.as_ref()) {
            Some(continuation) => db.load_checkpoint_resume(&owner_key(), &job.run, &continuation.checkpoint_id).await.unwrap(),
            None => None,
        };
        let epoch_state = if let Some(envelope) = &envelope {
            envelope.validate().unwrap();
            let mut response = db.client().query("SELECT VALUE state FROM checkpoint_runtime_epochs WHERE epoch = $epoch")
                .bind(("epoch", envelope.source_runtime_epoch.clone())).await.unwrap().check().unwrap();
            let states: Vec<String> = response.take(0).unwrap(); assert_eq!(states.len(), 1); Some(states[0].clone())
        } else { None };
        let mut response = db.client().query("SELECT VALUE data FROM checkpoint_resume_claims WHERE source_run_id = $run")
            .bind(("run", job.run.clone())).await.unwrap().check().unwrap();
        let claims: Vec<String> = response.take(0).unwrap();
        let mut executor_states = Vec::new();
        for data in &claims {
            let claim: universal_agent_runtime::uar::persistence::checkpoint_resume::CheckpointContinuationClaim = serde_json::from_str(data).unwrap();
            claim.validate(envelope.as_ref().expect("claimed approval has protected source envelope")).unwrap();
            let mut response = db.client().query("SELECT VALUE state FROM checkpoint_runtime_epochs WHERE epoch = $epoch")
                .bind(("epoch", claim.executor_runtime_epoch)).await.unwrap().check().unwrap();
            let states: Vec<String> = response.take(0).unwrap(); assert_eq!(states.len(), 1); executor_states.push(states[0].clone());
        }
        let claims = claims.iter().map(|data| serde_json::from_str::<Value>(data).unwrap()).collect();
        private_write(&job.result, &serde_json::to_vec_pretty(&Readback { records, envelope, epoch_state, claims, executor_states }).unwrap());
        return true;
    }
    if let Some(path) = std::env::var_os("UAR_DOT07_CLIENT_JOB") {
        let job: ClientJob = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        private_write(&job.ready, b"ready");
        let deadline = tokio::time::Instant::now() + WAIT;
        while !job.barrier.exists() {
            assert!(tokio::time::Instant::now() < deadline, "client barrier deadline");
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let response = reqwest::Client::builder().timeout(WAIT).build().unwrap()
            .post(format!("{}{}", job.base, job.path)).bearer_auth(token(OWNER, TENANT, ISSUER))
            .header("x-uar-workspace-id", "dot06-workspace").json(&job.body).send().await.unwrap();
        let status = response.status().as_u16(); let body: Value = response.json().await.unwrap();
        private_write(&job.result, &serde_json::to_vec(&json!({"status":status,"body":body})).unwrap());
        return true;
    }
    false
}

struct OwnedChild(Child);
impl Drop for OwnedChild { fn drop(&mut self) { if self.0.try_wait().ok().flatten().is_none() { let _ = self.0.kill(); let _ = self.0.wait(); } } }

fn child(root: &Path, role: &str, variable: &str, job: &impl serde::Serialize, remote: bool) -> OwnedChild {
    private_dir(root); let job_path = root.join("job.json");
    private_write(&job_path, &serde_json::to_vec(job).unwrap());
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--nocapture","--test-threads=1"]).env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default()).env("HOME", root)
        .env(variable, job_path).stdin(Stdio::null()).stdout(private_file(&root.join("stdout.log")))
        .stderr(private_file(&root.join("stderr.log")));
    for name in ["DYLD_LIBRARY_PATH", "LD_LIBRARY_PATH"] { if let Some(value) = std::env::var_os(name) { command.env(name, value); } }
    if remote { command.env("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED", "1"); }
    let child = command.spawn().unwrap();
    private_write(&root.join("process.json"), &serde_json::to_vec(&json!({"pid":child.id(),"role":role,"joined":false})).unwrap());
    OwnedChild(child)
}

async fn join(child: &mut OwnedChild, root: &Path) {
    let deadline = tokio::time::Instant::now() + WAIT;
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() { break status; }
        assert!(tokio::time::Instant::now() < deadline, "owned child deadline");
        tokio::time::sleep(Duration::from_millis(25)).await;
    };
    private_write(&root.join("joined.json"), &serde_json::to_vec(&json!({"pid":child.0.id(),"joined":true,"exitCode":status.code()})).unwrap());
    assert!(status.success(), "fixture child failed; private logs retained");
}

async fn readback(database: &Database, root: &Path, label: &str, run: &str, unbind: bool, reject_resolution: bool) -> Readback {
    let directory = root.join(label); private_dir(&directory); let result = directory.join("result.json");
    let job = ReadbackJob { database: database.clone(), run: run.to_owned(), result: result.clone(), unbind, reject_resolution };
    let mut process = child(&directory, "durable-readback", "UAR_DOT07_READBACK_JOB", &job, database.remote);
    join(&mut process, &directory).await;
    serde_json::from_slice(&std::fs::read(result).unwrap()).unwrap()
}

async fn pending(node: &Node, run: &str) -> Value {
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        let value = node.api(Method::GET, &format!("/api/uar/runs/{run}/tool-approval/pending"), None, 200).await;
        if !value["pending"].is_null() { return value["pending"].clone(); }
        assert!(tokio::time::Instant::now() < deadline, "pending approval deadline");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

async fn start(node: &Node, peer: &Peer, mode: &str, binding: bool) -> (String, String) {
    let mut body = json!({"artifact":resume::artifact(),"input":mode,"run_credentials":credentials(peer)});
    if binding { body["service_placement"] = json!({"intent":"new","expectedInstanceId":"urn:uar:dot06:fixture",
        "expectedProfile":universal_agent_runtime::uar::service_instance::SERVICE_PROFILE}); }
    let created = node.api(Method::POST, "/api/uar/runs", Some(body), 200).await;
    let run = created["run_id"].as_str().unwrap().to_owned(); let snapshot = pending(node, &run).await;
    let id = snapshot["approvalId"].as_str().or_else(|| snapshot["approval_id"].as_str()).unwrap().to_owned();
    (run, id)
}

async fn race(root: &Path, first: &Node, second: &Node, path: &str, body: &Value) -> Vec<Value> {
    let barrier = root.join("clients-go"); let mut jobs = Vec::new(); let mut processes = Vec::new();
    for (index, node) in [first, second].into_iter().enumerate() {
        let directory = root.join(format!("client-{index}")); private_dir(&directory);
        let job = ClientJob { base: node.base.clone(), path: path.to_owned(), body: body.clone(),
            result: directory.join("result.json"), ready: directory.join("ready"), barrier: barrier.clone() };
        processes.push(child(&directory, "independent-authenticated-client", "UAR_DOT07_CLIENT_JOB", &job, false));
        jobs.push((directory, job));
    }
    let deadline = tokio::time::Instant::now() + WAIT;
    while jobs.iter().any(|(_, job)| !job.ready.exists()) {
        assert!(tokio::time::Instant::now() < deadline); tokio::time::sleep(Duration::from_millis(25)).await;
    }
    private_write(&barrier, b"go");
    let mut results = Vec::new();
    for (mut process, (directory, job)) in processes.into_iter().zip(jobs) {
        join(&mut process, &directory).await;
        results.push(serde_json::from_slice(&std::fs::read(job.result).unwrap()).unwrap());
    }
    results
}

async fn scenario(root: &Path, profile: &str, case: &str) {
    private_dir(root); let effect_trace = root.join("effects.jsonl"); private_write(&effect_trace, b"");
    let database = Database::new(root, profile, &format!("approval_{case}")); let peer = peer(root).await;
    let mut predecessor = Node::boot(root, "predecessor", &database, &peer, &effect_trace, true).await;
    let (run, id) = start(&predecessor, &peer, if case == "unknown" || case == "siblings" { case } else { "seed" }, case == "binding").await;
    assert_eq!(effects(&effect_trace), 0, "pending approval cannot dispatch");
    let predecessor_pid = predecessor.child.id();
    if case == "abrupt" { predecessor.kill(); } else { predecessor.stop().await; }
    let source = readback(&database, root, "source-readback", &run, matches!(case, "legacy" | "legacy-deny"), case == "invalidation-outage").await;
    assert_eq!(source.records.len(), 1); assert_eq!(source.records[0].challenge_id, id);
    assert_eq!(source.records[0].state, ApprovalState::Pending); assert!(source.claims.is_empty());
    if !matches!(case, "legacy" | "legacy-deny") {
        let envelope = source.envelope.as_ref().unwrap();
        assert_eq!(envelope.owner_key, owner_key());
        assert_eq!(envelope.checkpoint().unwrap().node_id, "pending_tool_approval");
        assert_eq!(source.epoch_state.as_deref(), Some(if case == "abrupt" { "active" } else { "orderly_excluded" }));
        assert!(!serde_json::to_string(&source.records).unwrap().contains("dot06-synthetic-provider-key"));
    }
    if case == "expiration" {
        let remaining = source.records[0].expires_at.signed_duration_since(chrono::Utc::now()).to_std().unwrap_or_default();
        tokio::time::sleep(remaining + Duration::from_millis(50)).await;
    }
    let mut current = Node::boot(root, "current", &database, &peer, &effect_trace, true).await;
    assert_ne!(predecessor_pid, current.child.id());
    let path = format!("/api/uar/runs/{run}/tool-approval");
    let history = current.api(Method::GET, &path, None, 200).await;
    assert_eq!(history["records"][0]["challengeId"], id);
    assert_eq!(history["records"][0]["state"], if case == "expiration" { "expired" } else { "pending" });
    let body = json!({"approval_id":id,"approved":true,"resume":{"run_credentials":credentials(&peer)}});
    let before_requests = peer.count(); let mut admitted = Vec::new(); let mut statuses = Vec::new();
    let mut expected_claims = 0;
    let expected_state;
    match case {
        "expiration" | "legacy" => {
            let response = current.api(Method::POST, &path, Some(body), 409).await;
            assert_eq!(response["code"], if case == "expiration" { "approval_not_pending" } else { "approval_continuation_unavailable" });
            expected_state = if case == "expiration" { ApprovalState::Expired } else { ApprovalState::Invalidated };
            statuses.push(409);
        }
        "deny" | "legacy-deny" => {
            let response = current.api(Method::POST, &path, Some(json!({"approval_id":id,"approved":false})), 200).await;
            assert_eq!(response["decision"], "deny"); expected_state = ApprovalState::Denied; statuses.push(200);
            current.api(Method::POST, &path, Some(body), 409).await;
        }
        "abrupt" => {
            let response = current.api(Method::POST, &path, Some(body), 409).await;
            assert_eq!(response["code"], "checkpoint_predecessor_uncertain"); expected_state = ApprovalState::Pending; statuses.push(409);
        }
        "binding" => {
            let response = current.api(Method::POST, &path, Some(body), 409).await;
            assert_eq!(response["code"], "service_instance_incompatible"); expected_state = ApprovalState::Invalidated; statuses.push(409);
        }
        "revoked" | "tightened" | "invalidation-outage" => {
            let conversation = source.envelope.as_ref().unwrap().source_run().unwrap().conversation_id.unwrap();
            current.api(Method::PUT, &format!("/api/uar/conversations/{conversation}/policy"), Some(if case != "tightened" {
                json!({"tool_approval":"deny"}) } else { json!({"tools":{"mode":"none","denied_ids":["resume__effect"]}}) }), 200).await;
            let status = if case == "invalidation-outage" { 503 } else { 409 };
            let response = current.api(Method::POST, &path, Some(body), status).await;
            assert_eq!(response["code"], if case == "invalidation-outage" { "approval_persistence_unavailable" } else { "checkpoint_admission_denied" });
            expected_claims = 1;
            expected_state = if case == "invalidation-outage" { ApprovalState::Pending } else { ApprovalState::Invalidated }; statuses.push(status);
        }
        "race" => {
            let mut second = if profile == "shared" { Some(Node::boot(root, "second", &database, &peer, &effect_trace, true).await) } else { None };
            let results = race(root, &current, second.as_ref().unwrap_or(&current), &path, &body).await;
            assert_eq!(results.iter().map(|row| row["status"].as_u64().unwrap()).collect::<BTreeSet<_>>(), BTreeSet::from([200,409]));
            for (index, result) in results.iter().enumerate() {
                statuses.push(result["status"].as_u64().unwrap() as u16);
                if result["status"] == 200 {
                    let continuation = result["body"]["run_id"].as_str().unwrap(); admitted.push(continuation.to_owned());
                    if index == 1 && profile == "shared" { second.as_ref().unwrap().finish(continuation, true).await; }
                    else { current.finish(continuation, true).await; }
                }
            }
            if let Some(second) = &mut second { second.stop().await; }
            expected_claims = 1; expected_state = ApprovalState::Approved;
        }
        _ => {
            let snapshot = pending(&current, &run).await;
            assert_eq!(snapshot["approvalId"], id); assert_eq!(snapshot["resolvable"], false); assert_eq!(snapshot["recoveryAvailable"], true);
            if case == "pending-restart" {
                for (subject, tenant, issuer, expected) in [("other",TENANT,ISSUER,404),(OWNER,"other",ISSUER,404),(OWNER,TENANT,"other",401)] {
                    let (status, _) = current.request(&token(subject,tenant,issuer), Method::POST, &path, Some(body.clone())).await;
                    assert_eq!(status, expected);
                }
                current.api(Method::POST, &path, Some(json!({"approved":true})), 400).await;
                let missing = current.api(Method::POST, &path, Some(json!({"approval_id":id,"approved":true})), 422).await;
                assert_eq!(missing["code"], "run_credential_required");
                current.api(Method::POST, &path, Some(json!({"approval_id":id,"approved":true,
                    "resume":{"run_credentials":credentials(&peer),"history":{"session_id":"forged","messages":[]}}})), 422).await;
            }
            let response = current.api(Method::POST, &path, Some(body.clone()), 200).await;
            assert_eq!(response["continuation_admitted"], true);
            let continuation = response["run_id"].as_str().unwrap().to_owned(); assert_ne!(continuation, run);
            admitted.push(continuation.clone()); expected_claims = 1; expected_state = ApprovalState::Approved; statuses.push(200);
            current.finish(&continuation, case != "unknown").await;
            current.api(Method::POST, &path, Some(body), 409).await;
        }
    }
    let expected_effects = if admitted.is_empty() { 0 } else if case == "siblings" { 2 } else { 1 };
    assert_eq!(effects(&effect_trace), expected_effects);
    if admitted.is_empty() { assert_eq!(peer.count(), before_requests, "rejection cannot reach model inference"); }
    if case == "unknown" { assert!(!current.stop_status().await.success(), "unknown effect must hold executor exclusion"); }
    else { current.stop().await; }
    let final_state = readback(&database, root, "final-readback", &run, false, false).await;
    assert_eq!(final_state.records[0].state, expected_state); assert_eq!(final_state.claims.len(), expected_claims);
    if expected_claims == 1 {
        assert_eq!(final_state.executor_states, vec![if case == "unknown" { "active" } else { "orderly_excluded" }]);
        assert_eq!(final_state.claims[0]["checkpoint_id"], source.envelope.as_ref().unwrap().checkpoint_id);
        assert_eq!(final_state.claims[0]["checkpoint_revision"], source.envelope.as_ref().unwrap().checkpoint_revision);
    }
    if case == "unknown" || case == "invalidation-outage" {
        let mut second = Node::boot(root, "held-retry", &database, &peer, &effect_trace, true).await;
        let requests = peer.count(); let count = effects(&effect_trace);
        second.api(Method::POST, &path, Some(json!({"approval_id":id,"approved":true,"resume":{"run_credentials":credentials(&peer)}})), 409).await;
        assert_eq!(effects(&effect_trace), count); assert_eq!(peer.count(), requests); second.stop().await;
    }
    private_write(&root.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"schemaVersion":1,"profile":profile,"case":case,
        "sourceProcess":predecessor_pid,"sourceRun":run,"challengeId":id,
        "checkpointId":source.envelope.as_ref().map(|envelope| &envelope.checkpoint_id),
        "checkpointRevision":source.envelope.as_ref().map(|envelope| &envelope.checkpoint_revision),
        "sourceEpoch":source.envelope.as_ref().map(|envelope| &envelope.source_runtime_epoch),
        "continuationRuns":admitted,"decisionState":expected_state,"claimCount":expected_claims,
        "statuses":statuses,"effects":effects(&effect_trace),"providerRequests":peer.count(),
        "verified":true,"genuineInference":false})).unwrap());
}

async fn deadline_profiles(root: &Path) {
    for profile in ["local", "shared", "postgres"] {
        let directory = root.join(format!("{profile}-atomic-deadline")); private_dir(&directory);
        let postgres_url = if profile == "postgres" {
            let manifest: Value = serde_json::from_slice(&std::fs::read(std::env::var("UAR_DOT06_INFRA_FILE").unwrap()).unwrap()).unwrap();
            Some(manifest["postgres"]["url"].as_str().unwrap().to_owned())
        } else { None };
        let database = if profile == "postgres" { None } else { Some(Database::new(&directory, profile, "approval_atomic_deadline")) };
        let job = DeadlineJob { profile: profile.to_owned(), postgres_url, database, result: directory.join("acceptance.json") };
        let mut process = child(&directory, "production-deadline-cas", "UAR_DOT07_DEADLINE_JOB", &job, profile == "shared");
        join(&mut process, &directory).await;
        eprintln!("dot07 profile={profile} case=atomic-deadline status=passed");
    }
    let directory = root.join("memory-atomic-deadline"); private_dir(&directory);
    let store = universal_agent_runtime::uar::persistence::providers::memory::InMemoryProvider::new();
    let decisions = stale_deadline_intents(&store).await;
    private_write(&directory.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"schemaVersion":1,"profile":"memory",
        "case":"atomic-deadline","decisions":decisions,"effects":0,"verified":true,"genuineInference":false})).unwrap());
}

async fn unsupported(root: &Path, profile: &str) {
    private_dir(root); let effect_trace = root.join("effects.jsonl"); private_write(&effect_trace, b"");
    let database = Database::new(root, if profile == "unattested" { "shared" } else { "postgres" }, "approval_unsupported");
    let peer = peer(root).await;
    let mut original = Node::boot(root, "original", &database, &peer, &effect_trace, false).await;
    let (run, id) = start(&original, &peer, "seed", false).await; original.stop().await;
    let mut current = Node::boot(root, "current", &database, &peer, &effect_trace, false).await;
    let before = peer.count();
    let history = current.api(Method::GET, &format!("/api/uar/runs/{run}/tool-approval"), None, 200).await;
    assert!(history["records"][0]["continuation"].is_null());
    current.api(Method::POST, &format!("/api/uar/runs/{run}/tool-approval"),
        Some(json!({"approval_id":id,"approved":true,"resume":{"run_credentials":credentials(&peer)}})), 409).await;
    let response = current.api(Method::POST, &format!("/api/uar/runs/{run}/resume"), Some(json!({})), 503).await;
    assert_eq!(response["code"], "checkpoint_resume_unsupported"); assert_eq!(effects(&effect_trace), 0); assert_eq!(peer.count(), before);
    current.stop().await;
    private_write(&root.join("acceptance.json"), &serde_json::to_vec_pretty(&json!({"schemaVersion":1,"profile":profile,
        "case":"unsupported","sourceRun":run,"challengeId":id,"effects":0,"verified":true,"genuineInference":false})).unwrap());
}

pub async fn qualify() {
    assert!(cfg!(feature = "postgres-backend"), "Dot07 gate includes the explicit unsupported Postgres contract");
    let root = PathBuf::from(std::env::var("UAR_DOT07_EVIDENCE_ROOT").expect("private preserved Dot07 evidence root"));
    private_dir(&root);
    for profile in ["local", "shared"] { for case in CASES {
        scenario(&root.join(format!("{profile}-{case}")), profile, case).await;
        eprintln!("dot07 profile={profile} case={case} status=passed");
    } }
    unsupported(&root.join("unattested"), "unattested").await;
    unsupported(&root.join("postgres"), "postgres").await;
    deadline_profiles(&root).await;
}
