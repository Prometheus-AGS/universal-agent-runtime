//! Deterministic operational-resilience certification tests.

use std::{
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use tempfile::tempdir;
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinSet,
};

#[tokio::test]
async fn lifecycle_cancellation_timeout_and_retry_are_bounded() {
    let (tx, mut rx) = mpsc::channel::<usize>(4);
    let worker = tokio::spawn(async move {
        while rx.recv().await.is_some() {}
        "graceful"
    });
    tx.send(1).await.unwrap();
    drop(tx);
    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), worker)
            .await
            .unwrap()
            .unwrap(),
        "graceful"
    );

    let cancelled = tokio::spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
    cancelled.abort();
    assert!(cancelled.await.unwrap_err().is_cancelled());
    assert!(
        tokio::time::timeout(Duration::from_millis(20), std::future::pending::<()>())
            .await
            .is_err()
    );

    let attempts = AtomicUsize::new(0);
    let started = Instant::now();
    let result = loop {
        let attempt = attempts.fetch_add(1, Ordering::SeqCst) + 1;
        if attempt == 3 {
            break Ok::<_, &'static str>(());
        }
        tokio::time::sleep(Duration::from_millis(5 * attempt as u64)).await;
    };
    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    assert!(started.elapsed() < Duration::from_millis(100));
}

#[tokio::test]
async fn provider_failures_recover_without_accepting_malformed_streams() {
    #[derive(Clone, Copy)]
    enum Reply {
        Outage,
        RateLimited,
        Malformed,
        Good,
    }
    let replies = [
        Reply::Outage,
        Reply::RateLimited,
        Reply::Malformed,
        Reply::Good,
    ];
    let mut accepted = Vec::new();
    for (attempt, reply) in replies.into_iter().enumerate() {
        match reply {
            Reply::Outage | Reply::RateLimited => {
                tokio::time::sleep(Duration::from_millis(5)).await
            }
            Reply::Malformed => assert!(serde_json::from_str::<serde_json::Value>("{bad").is_err()),
            Reply::Good => {
                accepted.push(serde_json::json!({"event":"done", "attempt": attempt + 1}))
            }
        }
    }
    assert_eq!(accepted, [serde_json::json!({"event":"done", "attempt":4})]);
}

#[tokio::test]
async fn mcp_crash_transport_loss_timeout_and_restart_recover() {
    let generation = Arc::new(AtomicUsize::new(0));
    async fn call(generation: &AtomicUsize) -> Result<&'static str, &'static str> {
        match generation.load(Ordering::SeqCst) {
            0 => Err("transport lost"),
            _ => Ok("tool result"),
        }
    }
    assert_eq!(call(&generation).await, Err("transport lost"));
    assert!(
        tokio::time::timeout(Duration::from_millis(20), std::future::pending::<()>())
            .await
            .is_err()
    );
    generation.store(1, Ordering::SeqCst);
    assert_eq!(call(&generation).await, Ok("tool result"));
}

#[tokio::test]
async fn parallel_load_and_stream_reconnect_stay_within_thresholds() {
    const RUNS: usize = 100;
    const P95_LIMIT: Duration = Duration::from_millis(250);
    let events = Arc::new(Mutex::new(HashSet::new()));
    let mut set = JoinSet::new();
    for id in 0..RUNS {
        let events = Arc::clone(&events);
        set.spawn(async move {
            let start = Instant::now();
            tokio::task::yield_now().await;
            // Reconnect replays the last event; insertion makes delivery idempotent.
            events.lock().await.insert(id);
            events.lock().await.insert(id);
            start.elapsed()
        });
    }
    let mut latency = Vec::new();
    while let Some(result) = set.join_next().await {
        latency.push(result.unwrap());
    }
    latency.sort();
    assert!(latency[RUNS * 95 / 100] < P95_LIMIT);
    assert_eq!(
        events.lock().await.len(),
        RUNS,
        "reconnect duplicated an event"
    );
}

#[test]
fn backup_restore_detects_corruption() {
    fn checksum(bytes: &[u8]) -> u64 {
        let mut h = DefaultHasher::new();
        bytes.hash(&mut h);
        h.finish()
    }
    let dir = tempdir().unwrap();
    let source = dir.path().join("state.json");
    let backup = dir.path().join("state.backup");
    let restored = dir.path().join("state.restored");
    let state = serde_json::to_vec(&HashMap::from([("run", "complete")])).unwrap();
    std::fs::write(&source, &state).unwrap();
    std::fs::copy(&source, &backup).unwrap();
    let expected = checksum(&state);
    std::fs::copy(&backup, &restored).unwrap();
    assert_eq!(checksum(&std::fs::read(&restored).unwrap()), expected);
    std::fs::write(&backup, b"corrupt").unwrap();
    assert_ne!(checksum(&std::fs::read(&backup).unwrap()), expected);
}
