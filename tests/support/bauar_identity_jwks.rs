//! Controlled JWKS responder and real-clock timing support.
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[derive(Clone)]
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub header_delay: Duration,
    pub body_delay: Duration,
}

impl Reply {
    pub fn keys(keys: Vec<Value>) -> Self {
        Self {
            status: 200,
            body: json!({"keys":keys}).to_string(),
            header_delay: Duration::ZERO,
            body_delay: Duration::ZERO,
        }
    }
    pub fn failure() -> Self {
        Self {
            status: 503,
            ..Self::keys(Vec::new())
        }
    }
}

struct JwksState {
    reply: Mutex<Reply>,
    starts: Mutex<Vec<Instant>>,
    completions: Mutex<Vec<Instant>>,
    active: AtomicUsize,
    maximum: AtomicUsize,
}

struct Active(Arc<JwksState>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct JwksPeer {
    pub url: String,
    state: Arc<JwksState>,
    task: tokio::task::JoinHandle<()>,
}

impl JwksPeer {
    pub async fn start(keys: Vec<Value>) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!(
            "http://{}/jwks?fixture-endpoint-sentinel",
            listener.local_addr().unwrap()
        );
        let state = Arc::new(JwksState {
            reply: Mutex::new(Reply::keys(keys)),
            starts: Mutex::new(Vec::new()),
            completions: Mutex::new(Vec::new()),
            active: AtomicUsize::new(0),
            maximum: AtomicUsize::new(0),
        });
        let owned = state.clone();
        let task = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let (mut stream, _) = accepted.unwrap();
                        let state = owned.clone();
                        connections.spawn(async move {
                            let mut request = [0u8;4096];
                            stream.read(&mut request).await.unwrap();
                            state.starts.lock().unwrap().push(Instant::now());
                            let active = state.active.fetch_add(1, Ordering::SeqCst)+1;
                            state.maximum.fetch_max(active, Ordering::SeqCst);
                            let _active = Active(state.clone());
                            let reply = state.reply.lock().unwrap().clone();
                            tokio::time::sleep(reply.header_delay).await;
                            let head = format!("HTTP/1.1 {} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", reply.status, reply.body.len());
                            if stream.write_all(head.as_bytes()).await.is_err() { return; }
                            tokio::time::sleep(reply.body_delay).await;
                            if stream.write_all(reply.body.as_bytes()).await.is_ok() {
                                state.completions.lock().unwrap().push(Instant::now());
                            }
                        });
                    }
                    _ = connections.join_next(), if !connections.is_empty() => {}
                }
            }
        });
        Self { url, state, task }
    }
    pub fn set(&self, reply: Reply) {
        *self.state.reply.lock().unwrap() = reply;
    }
    pub fn starts(&self) -> Vec<Instant> {
        self.state.starts.lock().unwrap().clone()
    }
    pub fn completed_at(&self) -> Instant {
        *self
            .state
            .completions
            .lock()
            .unwrap()
            .last()
            .expect("JWKS completed")
    }
    pub fn maximum(&self) -> usize {
        self.state.maximum.load(Ordering::SeqCst)
    }
}

impl Drop for JwksPeer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Wait on the real clock; standalone-server time cannot be paused by this test.
pub async fn after(origin: Instant, age: Duration) {
    tokio::time::sleep(age.saturating_sub(origin.elapsed())).await;
}
