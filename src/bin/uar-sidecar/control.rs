//! Private inherited-pipe management; launch authentication precedes this owner.

use std::io::Write as _;
use std::sync::{Arc, Mutex, mpsc};
use futures::FutureExt as _;
use secrecy::zeroize::Zeroize as _;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use universal_agent_runtime::server::{LifecycleRequest, ServerLifecycleControl};

use super::inherited_input::AsyncInput;

const MAX_COMMAND_BYTES: usize = 8192;

pub async fn serve(mut input: AsyncInput, control: ServerLifecycleControl,
    shutdown: CancellationToken, stop: CancellationToken) -> anyhow::Result<()> {
    let _shutdown_on_exit = shutdown.clone().drop_guard();
    let commands = TaskTracker::new();
    let failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let (responses, receiver) = mpsc::channel::<serde_json::Value>();
    // One owned writer serializes whole response lines without blocking input.
    let writer_shutdown = shutdown.clone();
    let writer = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        let _shutdown_on_writer_exit = writer_shutdown.clone().drop_guard();
        for response in receiver {
            let outcome = (|| {
                let mut stdout = std::io::stdout().lock();
                writeln!(stdout, "LIFECYCLE:{response}")?;
                stdout.flush()
            })();
            if outcome.is_err() { writer_shutdown.cancel(); }
            outcome?;
        }
        Ok(())
    });
    let result = std::panic::AssertUnwindSafe(read_commands(&mut input, &control, &stop,
        &commands, &responses, &failures, &shutdown)).catch_unwind().await;
    // EOF requests shutdown even while an accepted reconnect remains pending.
    shutdown.cancel();
    let close = input.close().await;
    // This reader is the sole registration authority and has stopped. Every
    // accepted future was registered before dispatch to the executor.
    commands.close();
    commands.wait().await;
    drop(responses);
    let writer = writer.await;
    let mut errors = failures.lock().unwrap_or_else(|error| error.into_inner()).clone();
    match result {
        Ok(Ok(())) => {},
        Ok(Err(error)) => errors.push(error.to_string()),
        Err(_) => errors.push("sidecar control reader panicked".into()),
    }
    if let Err(error) = close { errors.push(format!("control input closure: {error}")); }
    match writer {
        Ok(Ok(())) => {},
        Ok(Err(error)) => errors.push(format!("control response closure: {error}")),
        Err(_) => errors.push("sidecar control response writer panicked".into()),
    }
    anyhow::ensure!(errors.is_empty(), "{}", errors.join("; "));
    Ok(())
}

async fn read_commands(input: &mut AsyncInput, control: &ServerLifecycleControl,
    stop: &CancellationToken, commands: &TaskTracker,
    responses: &mpsc::Sender<serde_json::Value>, failures: &Arc<Mutex<Vec<String>>>,
    shutdown: &CancellationToken) -> anyhow::Result<()> {
    let mut line = secrecy::zeroize::Zeroizing::new(Vec::new());
    let mut bytes = secrecy::zeroize::Zeroizing::new([0u8; 1024]);
    loop {
        let count = tokio::select! {
            biased;
            _ = stop.cancelled() => return Ok(()),
            result = input.read(&mut bytes[..]) => result?,
        };
        if count == 0 { return Ok(()); }
        for byte in &bytes[..count] {
            if *byte != b'\n' {
                anyhow::ensure!(line.len() < MAX_COMMAND_BYTES, "sidecar control command exceeds protocol limit");
                line.push(*byte);
                continue;
            }
            let request = serde_json::from_slice::<LifecycleRequest>(&line);
            line.zeroize();
            line.clear();
            let request = match request {
                Ok(request) => request,
                Err(_) => {
                    responses.send(serde_json::json!({"version": 1, "error": "invalid_lifecycle_command"}))
                        .map_err(|_| anyhow::anyhow!("sidecar control response writer closed"))?;
                    continue;
                }
            };
            let control = control.clone();
            let responses = responses.clone();
            let failures = Arc::clone(failures);
            let shutdown = shutdown.clone();
            let tracked = commands.track_future(async move {
                let request_id = request.request_id.clone();
                let outcome = std::panic::AssertUnwindSafe(control.execute(request)).catch_unwind().await;
                let response = match outcome {
                    Ok(Ok(response)) => response,
                    Ok(Err(_)) => serde_json::json!({"version": 1, "request_id": request_id,
                        "error": "lifecycle_command_failed"}),
                    Err(_) => {
                        failures.lock().unwrap_or_else(|error| error.into_inner())
                            .push("sidecar lifecycle command panicked".into());
                        shutdown.cancel();
                        serde_json::json!({"version": 1, "request_id": request_id,
                            "error": "lifecycle_command_panicked"})
                    }
                };
                if responses.send(response).is_err() {
                    failures.lock().unwrap_or_else(|error| error.into_inner())
                        .push("sidecar control response writer closed".into());
                    shutdown.cancel();
                }
            });
            drop(tokio::spawn(tracked));
        }
        bytes.zeroize();
    }
}
