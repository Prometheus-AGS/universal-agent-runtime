# Behavioral controls — `wire-orchestrator-delegation`

This change introduces no fail-closed authentication or authorization guard.
The controls below prove that a configured graph remains orchestrator-specific
and that empty specialist output cannot be misreported as successful delegation.

## A configured graph does not divert default-agent

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_chat_completion \
  attached_graph_does_not_change_default_agent_path -- --exact
```

Observed output, exit 0:

```text
running 1 test
test attached_graph_does_not_change_default_agent_path ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.06s
```

The control requires exactly one driver call and the direct `default answer`
without a delegated-agent prefix. If the attached graph applied globally, the
router would make a second call and the assertion would fail.

## Empty specialist output fails closed

Command:

```bash
cargo test --locked -p universal-agent-runtime --no-default-features \
  --features server-full --test test_agent_node \
  test_agent_node_local_empty_output_is_error -- --exact
```

Observed output, exit 0:

```text
running 1 test
test test_agent_node_local_empty_output_is_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

The control drives the specialist stream directly to `Done` with no text and
requires the graph error `AgentNode 'rust-reviewer' returned empty output` plus
the absence of `_agent_output_rust-reviewer`. If empty output were accepted,
both assertions would fail.
