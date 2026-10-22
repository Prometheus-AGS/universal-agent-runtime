# Graph child caller-tool thread plumbing

Task 13.21, Execute. Verification CLOSED. This teammate performed source
inspection and authorized implementation/documentation writing only. No tests,
checks, formatting, compiler probes, builds, imports, network calls, or runtime
commands ran. Other writers share this checkout; preserve their changes.

## Contract agreed before implementation

Lead owns the opaque, Clone, non-Deserialize `GraphCallerTurnCapture` and its
`bind_executing(self, thread_id: String, run_id: String) ->
anyhow::Result<ClientToolInput>` method. It supplies the captured graph resume
state and root API agent/run/session correlation, then binds the service-minted
executing child IDs. Runtime graph writer consumes
`AgentToolContext::spawn_graph(request, capture) -> Result<AgentSummary,
AgentControlError>` and owns graph outcomes/manager finalization.

## Files changed

- `src/uar/runtime/thread/control.rs`: introduce the host-only graph spawn method.
  Ordinary and captured local spawn share one private validation/admission/result
  path. Both validate intent, require current `spawn_agent` authority, call the
  trusted host, and check the returned edge/artifact/path. No authority was added
  to the model-facing spawn request. `AgentThreadHost::spawn_graph` has an honest
  unsupported default for hosts without this capability; it cannot fabricate a
  child. Repository source currently has only the ThreadService host implementation.
- `src/uar/runtime/thread/service.rs`: this file defines `HostedThreadTurn`.
  Add optional normalized client-tool input to that host-only turn. Store a
  capture on a freshly admitted local child's Entry only. Require capture root
  run and API agent to match the service's retained root. After the normal
  child `begin_turn` mints its real run UUID, consume the capture exactly once
  with actual executing thread/run IDs into the first HostedThreadTurn. All
  ordinary/root/remote entries carry no capture. Later mailbox turns have no
  automatic inherited capture. The existing tracked job lifetime, spawn
  admission, current artifact resolution, policy intersection, cancellation,
  persistence-before-publication, and shared budget checks remain in their
  original paths.
- `src/uar/runtime/thread/kernel.rs`: pass the normalized captured input into
  `RunExecutionRequest.client_tools`. Set generic inherited history only when
  no captured input exists, allowing prepared canonical worker tool-paired
  history to remain authoritative. The current worker artifact/system policy
  and inherited root execution resources still use the existing executor path.

This handoff is the only additional file. No graph, manager, orchestrator,
capture-type, persistence, API initializer, or protocol adapter file was edited
by this teammate for 13.21.

## Boundary and remaining work

The new root identity check belongs to the real trusted-host boundary: a graph
capture cannot be applied to another root's service. The child identity check
and input validation are delegated to the lead-owned capture method. No
speculative timeout, retry, fallback, remote propagation, or automatic
grandchild propagation was added.

Source inspection located all production HostedThreadTurn construction in
service.rs and all AgentThreadHost implementations in that same file. This is
inspection evidence, not compilation evidence. Compatibility of the coordinated
capture API and borrow/type correctness are unverified. Graph child schema
dispatch, canonical-history continuation, durable root/child correlation,
policy readmission, real approval, cancellation, cold restart and producer
cleanup require the lead's later authorized whole-phase integration gate.
