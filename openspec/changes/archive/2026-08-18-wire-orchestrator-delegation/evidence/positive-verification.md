# Positive verification — `wire-orchestrator-delegation`

## Local graph behavior

Commands:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_agent_node \
  test_agent_node_local_delegation_uses_graph_driver -- --exact
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_agent_node \
  test_agent_node_remote_delegation -- --exact
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_graph_execution \
  test_state_flows_through_multiple_nodes -- --exact
```

Observed results, exit 0:

```text
local AgentNode: 1 passed; 0 failed; 4 filtered out
remote AgentNode: 1 passed; 0 failed; 4 filtered out
graph trace: 1 passed; 0 failed; 3 filtered out
```

The remote assertion requires the A2A task ID and `remote contribution` text.
The graph assertion requires the exact trace `a`, `b`, `c`.

## Orchestrator-only routing and answer projection

Commands:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_chat_completion \
  orchestrator_run_routes_and_streams_delegated_answer -- --exact
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_chat_completion \
  attached_graph_does_not_change_default_agent_path -- --exact
```

Observed results, exit 0:

```text
orchestrator: 1 passed; 0 failed; 8 filtered out; finished in 0.08s
default control: 1 passed; 0 failed; 8 filtered out; finished in 0.06s
```

The orchestrator assertion requires a distinct descriptor, two driver calls,
the original request in the router prompt, the selected specialist identity in
the delegated prompt, four step events, and the exact attributed answer. The
control requires one driver call and an unprefixed direct answer.

## Recorded and live server path

Commands:

```bash
UAR_LIVE_INTEGRATION_BACKEND=recorded \
  cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test integration \
  live::capability_cases::l2_c06_orchestrator_delegates_with_trace \
  -- --exact --test-threads=1

UAR_LIVE_INTEGRATION_BACKEND=live \
  cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test integration \
  live::capability_cases::l2_c06_orchestrator_delegates_with_trace \
  -- --exact --test-threads=1
```

Observed outputs, exit 0:

```text
recorded: 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out; finished in 9.63s
live: 1 passed; 0 failed; 0 ignored; 0 measured; 92 filtered out; finished in 14.12s
```

The recorded path requires `[rust-reviewer]`, the exact fixture contribution,
and `runtime.step` start/finish events. The live path requires the same
attribution and events plus non-whitespace specialist content after the prefix
against `http://127.0.0.1:8181/v1`.

## Change-level checks

Commands:

```bash
cargo fmt --all -- --check
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
openspec validate wire-orchestrator-delegation --strict
git diff --check -- openspec/changes/wire-orchestrator-delegation src/embedded.rs src/server.rs src/uar/defaults.rs src/uar/runtime/graph src/uar/runtime/manager.rs tests/integration/live/capability_cases.rs tests/test_agent_node.rs tests/test_chat_completion.rs tests/test_graph_execution.rs
```

Observed results:

```text
cargo fmt: exit 0, no output
cargo check: exit 0; 3 pre-existing warnings
cargo clippy: exit 0; 571 warnings
OpenSpec: Change 'wire-orchestrator-delegation' is valid
git diff --check: exit 0, no output
```

Candidate SHA-256 values:

```text
0606af73ae2ce74211aa31bdad5412d8ecfdceed2b7163f0fe20a7f5682e26dd  src/embedded.rs
e3e628ee3d7a449b7ebff375cf1a80e6663629594abdb70d5f5bb0219ecb5ba8  src/server.rs
dca9a400364b3a6fbf88c72c5f758eecd0b2cc1ff299b6e03c28c32d58d72688  src/uar/defaults.rs
4edfd28c1fa8e4213f925fcc4753786f9fafdd09b965f3153c78e3247943c560  src/uar/runtime/graph/engine.rs
1066bc0e221adea194d161c427aa9558611b0c0bb2e2ffebc566ed6882770102  src/uar/runtime/graph/nodes/agent_node.rs
f60d96770a5a82367a9b96b165a81b6b3a33113b510a68a1168fde2bcaf0a863  src/uar/runtime/graph/nodes/router_node.rs
120fd13b6bc7c39ec13e95cb2e247f2dc428885f1ca651cde173fa73ad78925c  src/uar/runtime/manager.rs
862ebf285b995d6d3488c5ff0f4197e4e6443794c0202f08b28c7656e6168ab3  tests/integration/live/capability_cases.rs
0d3def122d0b4af4a73a2c541b597866b9d3cb31aa53f0669826750ef722c30c  tests/test_agent_node.rs
77f8793d5753f2238fb537c262e78770139e124a8f0007ef3862f187626e2a4a  tests/test_chat_completion.rs
a1c19e34ae1c6c59a609892917fa0244ad8ea6459cc3f06f1fa6bc6bcb89223a  tests/test_graph_execution.rs
```

Phase Tier 2 remains deferred until all active-phase changes are complete.
