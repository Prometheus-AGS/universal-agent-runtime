//! JSON Lines fixture host for the media effects behavior scenarios.
//! Keep the process alive across UAR restarts to retain independent evidence.
use std::io::{self, BufRead, Write};

use serde::Deserialize;
use serde_json::{Value, json};
use universal_agent_runtime::uar::compiler::collaboration::media_effect::{
    MediaDispatchClaim, MediaHostConnector, MockMediaPeer,
};

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", rename_all_fields = "camelCase", deny_unknown_fields)]
enum Request {
    Invoke {
        claim: Box<MediaDispatchClaim>,
        #[serde(default)]
        lose_result: bool,
    },
    Reconcile { attempt_id: String },
    Inspect,
}

async fn handle(peer: &mut MockMediaPeer, request: Request) -> serde_json::Result<Value> {
    match request {
        Request::Invoke { claim, lose_result } => {
            if lose_result {
                peer.lose_next_result();
            }
            serde_json::to_value(peer.invoke(*claim).await)
        }
        Request::Reconcile { attempt_id } => serde_json::to_value(peer.reconcile(&attempt_id).await),
        Request::Inspect => Ok(json!({ "invocations": peer.invocations(), "applied": peer.applied() })),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let mut peer = MockMediaPeer::default();
    let mut output = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        let response = match serde_json::from_str::<Request>(&line?) {
            Ok(request) => handle(&mut peer, request).await?,
            Err(_) => json!({ "error": "invalid_mock_media_request" }),
        };
        serde_json::to_writer(&mut output, &response)?;
        writeln!(output)?;
        output.flush()?;
    }
    Ok(())
}
