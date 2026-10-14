//! Real persistence-boundary accounting acceptance. Authored during implementation;
//! execute only at the complete expanded Dot phase boundary.
#![cfg(any(feature="surreal-backend", feature="postgres-backend"))]

use std::{path::Path, process::Command, sync::Arc};
use universal_agent_runtime::uar::persistence::{PersistenceLayer, accounting::{AccountingState, CallAdmission, CallIdentity, CallUsage, ScopeBinding, ScopeIdentity, ScopeLimits, ScopeUpdate}};

fn child(mode: &str, backend: &str, endpoint: &str, namespace: &str, result: &Path) -> Command {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", "accounting_process", "--nocapture"])
        .env("UAR_ACCOUNTING_FIXTURE_MODE",mode).env("UAR_ACCOUNTING_FIXTURE_BACKEND",backend)
        .env("UAR_ACCOUNTING_FIXTURE_ENDPOINT",endpoint).env("UAR_ACCOUNTING_FIXTURE_NAMESPACE",namespace)
        .env("UAR_ACCOUNTING_FIXTURE_RESULT",result);
    command
}
fn restart(backend: &str, endpoint: &str) {
    let directory = tempfile::tempdir().unwrap();
    let namespace = uuid::Uuid::new_v4().to_string();
    let result = directory.path().join("receipt");
    assert!(child("pending",backend,endpoint,&namespace,&result).status().unwrap().success());
    assert!(child("settle",backend,endpoint,&namespace,&result).status().unwrap().success());
    assert_eq!(std::fs::read_to_string(&result).unwrap(),"one settlement; unknown liability retained; generation and owner enforced");
}
fn race(backend: &str, endpoint: &str) {
    let directory = tempfile::tempdir().unwrap();
    let namespace = uuid::Uuid::new_v4().to_string();
    let one = directory.path().join("one");
    let two = directory.path().join("two");
    assert!(child("initialize",backend,endpoint,&namespace,&one).status().unwrap().success());
    let mut first = child("race-one",backend,endpoint,&namespace,&one).spawn().unwrap();
    let mut second = child("race-two",backend,endpoint,&namespace,&two).spawn().unwrap();
    assert!(first.wait().unwrap().success());
    assert!(second.wait().unwrap().success());
    let receipts = [std::fs::read_to_string(one).unwrap(),std::fs::read_to_string(two).unwrap()];
    assert_eq!(receipts.iter().filter(|value| value.as_str()=="admitted").count(),1);
    assert_eq!(receipts.iter().filter(|value| value.as_str()=="refused").count(),1);
}

#[cfg(feature="surreal-backend")]
#[test]
fn surrealkv_pending_liability_survives_process_exit() {
    let directory = tempfile::tempdir().unwrap();
    restart("surreal",&format!("surrealkv://{}",directory.path().join("accounting").display()));
}
#[cfg(feature="surreal-backend")]
#[test]
#[ignore = "requires dedicated real durable networked SurrealDB and explicit supervisor attestation"]
fn shared_surreal_two_process_budget_admission() {
    let endpoint = std::env::var("UAR_ACCOUNTING_SURREAL_URL").expect("dedicated SurrealDB endpoint required");
    assert_eq!(std::env::var("UAR_REMOTE_SURREAL_DURABILITY_ATTESTED").as_deref(),Ok("1"));
    restart("surreal",&endpoint);
    race("surreal",&endpoint);
}
#[cfg(feature="postgres-backend")]
#[test]
#[ignore = "requires a dedicated real Postgres database with pgvector"]
fn postgres_restart_and_two_process_budget_admission() {
    let endpoint = std::env::var("UAR_ACCOUNTING_POSTGRES_URL").expect("dedicated Postgres endpoint required");
    restart("postgres",&endpoint);
    race("postgres",&endpoint);
}

async fn store(backend: &str, endpoint: &str, namespace: &str) -> Arc<dyn PersistenceLayer> {
    match backend {
        #[cfg(feature="surreal-backend")]
        "surreal" => Arc::new(universal_agent_runtime::uar::persistence::providers::surreal::SurrealDbProvider::new(
            endpoint,std::env::var("SURREAL_USER").ok().as_deref(),std::env::var("SURREAL_PASS").ok().as_deref(),
            Some("accounting_fixture"),Some(namespace)).await.unwrap()),
        #[cfg(feature="postgres-backend")]
        "postgres" => Arc::new(universal_agent_runtime::uar::persistence::providers::postgres::PostgresProvider::new(endpoint).await.unwrap()),
        _ => panic!("accounting backend feature is unavailable: {backend}, namespace={namespace}, endpoint configured={}",!endpoint.is_empty()),
    }
}
fn identity(namespace: &str, call: &str) -> CallIdentity {
    CallIdentity { owner: format!("verified-issuer-tenant-owner:{namespace}"), run_id: namespace.into(), call_id: call.into(), attempt: 1 }
}
async fn admission(store: &dyn PersistenceLayer, namespace: &str, call: &str, amount: f64) -> CallAdmission {
    let identity = identity(namespace,call);
    let mut scopes = Vec::new();
    for kind in ["run","session","agent"] {
        let id = ScopeIdentity { owner: identity.owner.clone(),scope:kind.into(),scope_id:namespace.into() };
        let row = store.ensure_accounting_scope(&id,&ScopeLimits { cost_usd:Some(2.0),tokens:Some(100) }).await.unwrap();
        scopes.push(ScopeBinding { identity:id,generation:row.generation });
    }
    CallAdmission { identity,model:"fixture/provider".into(),scopes,reserved_cost_usd:Some(amount),reserved_tokens:Some(10),source_reservation:None }
}

#[tokio::test]
async fn accounting_process() {
    let Ok(mode) = std::env::var("UAR_ACCOUNTING_FIXTURE_MODE") else { return; };
    let namespace = std::env::var("UAR_ACCOUNTING_FIXTURE_NAMESPACE").unwrap();
    let store = store(&std::env::var("UAR_ACCOUNTING_FIXTURE_BACKEND").unwrap(),&std::env::var("UAR_ACCOUNTING_FIXTURE_ENDPOINT").unwrap(),&namespace).await;
    let receipt = std::env::var("UAR_ACCOUNTING_FIXTURE_RESULT").unwrap();
    match mode.as_str() {
        "initialize" => { admission(store.as_ref(),&namespace,"initialize",1.5).await; }
        "pending" => {
            let mut request = admission(store.as_ref(),&namespace,"call",1.5).await;
            request.model = "remote-delegation".into();
            store.admit_accounting_call(&request).await.unwrap();
            store.bind_accounting_delegation(&universal_agent_runtime::uar::persistence::accounting::AccountingDelegation {
                contract_digest:format!("contract-{namespace}"),source:request.identity.clone(),
            }).await.unwrap();
            store.record_accounting_usage(&CallUsage { identity:request.identity,cost_usd:Some(0.25),tokens:2,terminal:false,evidence:None }).await.unwrap();
            // Leave with an admitted partial call; no same-handle reconstruction.
        }
        "settle" => {
            let request = admission(store.as_ref(),&namespace,"call",1.5).await;
            let calls = store.list_accounting_calls(&request.identity.owner,&namespace).await.unwrap();
            assert_eq!(calls.len(),1); assert_eq!(calls[0].state,AccountingState::Pending);
            let shared = store.get_accounting_delegation(&format!("contract-{namespace}")).await.unwrap().unwrap();
            assert_eq!(shared.admission.identity,request.identity);
            assert!(store.get_accounting_delegation(&format!("foreign-contract-{namespace}")).await.unwrap().is_none());
            let blocked = admission(store.as_ref(),&namespace,"blocked",1.0).await;
            assert!(store.admit_accounting_call(&blocked).await.is_err());
            let usage = CallUsage { identity:request.identity.clone(),cost_usd:Some(0.5),tokens:5,terminal:true,evidence:Some("confirmed provider final receipt".into()) };
            store.record_accounting_usage(&usage).await.unwrap();
            store.record_accounting_usage(&usage).await.unwrap();
            for binding in &request.scopes {
                let row = store.get_accounting_scope(&binding.identity).await.unwrap().unwrap();
                assert_eq!(row.settled_cost_usd,0.5); assert_eq!(row.settled_tokens,5);
                assert_eq!(row.pending_calls,0); assert_eq!(row.reserved_cost_usd,0.0);
            }
            let scope = &request.scopes[0];
            store.update_accounting_scope(&ScopeUpdate { identity:scope.identity.clone(),expected_generation:scope.generation,
                limits:ScopeLimits{cost_usd:Some(0.5),tokens:Some(100)},opening_cost_usd:None,opening_tokens:None,evidence:"operator policy update".into() }).await.unwrap();
            assert!(store.admit_accounting_call(&blocked).await.is_err());
            let mut foreign = usage.clone(); foreign.identity.owner="other verified issuer".into();
            assert!(store.record_accounting_usage(&foreign).await.is_err());
            let legacy_id = format!("legacy-{namespace}");
            store.record_cost_entry("session",&legacy_id,0.7).await.unwrap();
            let legacy = ScopeIdentity{owner:request.identity.owner.clone(),scope:"session".into(),scope_id:legacy_id};
            let row = store.ensure_accounting_scope(&legacy,&ScopeLimits::default()).await.unwrap();
            assert!(!row.reconciled); assert_eq!(row.settled_cost_usd,0.0);
            assert_eq!(store.list_cost_history("session",&legacy.scope_id).await.unwrap().len(),1);
            std::fs::write(receipt,"one settlement; unknown liability retained; generation and owner enforced").unwrap();
        }
        "race-one" | "race-two" => {
            let request = admission(store.as_ref(),&namespace,&mode,1.5).await;
            let result = if store.admit_accounting_call(&request).await.is_ok() { "admitted" } else { "refused" };
            std::fs::write(receipt,result).unwrap();
        }
        _ => panic!("unknown accounting fixture mode"),
    }
}
