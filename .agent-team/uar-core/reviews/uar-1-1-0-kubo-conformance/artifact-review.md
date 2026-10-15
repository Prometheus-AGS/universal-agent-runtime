# UAR v1.1.0 Kubo artifact review

## Verdict and scope

**Certification withheld.** One CRITICAL trust-boundary defect and three WARNING protocol defects are established by source inspection of the candidate based on `c9dca17084ab30a7c602df9fe99e5618fa42349e`, branch `codex/uar-1-1-0-kubo-conformance`. Runtime reproduction is not claimed. This is the initial review; fixes require a bounded corrective review and the lead's production acceptance.

Phase: Execute, `uar-1-1-0-kubo-conformance`. Inputs: applicable OpenSpec proposal/design/tasks/spec delta, release contract, cumulative production diff, untracked production files and acceptance fixtures. Producer conversation and handoff self-assessments were excluded. The scope includes A2A admission and projection, host skill roles, MCP grant and continuation authority, manifest import persistence, configuration changes and installed-image acceptance source. This is a bounded review, not exhaustive certification of the repository.

## CRITICAL C1 — alternate URL spelling bypasses restricted MCP destination identity

Evidence: `src/uar/runtime/turn/host/mcp.rs:185-202` compares the whole parsed URL against restricted global entries. `src/mcp/registry.rs:1186-1200` repeats that comparison for live authority. `src/uar/runtime/turn/host/mcp.rs:307-340` accepts loopback HTTP URLs with fragments and creates the resulting direct destination with no grant policy or credential lease.

Reproduction reasoning: register `http://127.0.0.1:PORT/mcp` with `allowed_agent_ids: [A]`. Agent B submits a direct run destination named `different-name`, URL `http://127.0.0.1:PORT/mcp#alias`, and any required peer credential in its request headers. Both name checks and complete-URL comparisons differ, but the URI fragment is not part of the HTTP request target. The direct declaration passes to `connect_http_snapshot` (`src/mcp/registry.rs:855-914`) without an agent-bound lease. The protected peer can therefore receive initialization despite B's exclusion. This violates the explicit no-peer-contact requirement even if a later tool approval prevents invocation.

The existing `direct_endpoint_alias` fixture in `tests/fixtures/uar_1_1_mcp_contract.py` changes only the name and retains the exact URL; it does not exercise this case.

Correction: use one consistent destination identity at initial admission and every live/reconnect check, excluding non-wire URL fragments; address accepted loopback host aliases that identify the same destination. Add a fragment-alias no-contact case to the real-runtime acceptance. The trust owner has been assigned the correction; no fix is accepted by this initial report.

## WARNING W1 — official tenant field means different things across bindings

Evidence: `src/uar/api/a2a/grpc_v1.rs:105-115` overwrites the selected agent ID with nonempty request `tenant`. `src/uar/api/a2a/v1.rs:135-136,171-174` instead compares that same field to the verified caller's tenant and uses the HTTP route for agent selection. The local normative proto labels it a tenant ID (`proto/a2a_v1.proto:644`). gRPC authentication already supports agent selection through `x-uar-agent-id` (`src/uar/api/a2a/grpc.rs:130-145`).

Falsifying observation: for verified tenant T and registered agent A, JSON-RPC to A with `tenant:T` admits the correct target; gRPC with agent metadata A and `tenant:T` targets T instead. Conversely `tenant:A` is accepted as a gRPC selector but rejected by JSON-RPC when A differs from T. This is demonstrated routing divergence, not a claimed cross-tenant data disclosure. `tests/fixtures/a2a_v1_approval_governance.py:145-164` encodes the divergence by sending tenant=agent only in the gRPC branch.

Correction: align tenant validation and agent routing across bindings and exercise identical tenant semantics in both transport fixtures.

## WARNING W2 — gRPC loses durable artifact identity/content and changes history identity

Evidence: `src/uar/api/a2a/grpc_v1.rs:287-295` replaces persisted artifact IDs with positional IDs and drops description and metadata. In contrast `src/uar/api/a2a/v1.rs:327-334` preserves those fields and exposes a completed-output artifact. gRPC `task_to_pb` emits only stored artifacts, so the completed output artifact is missing there. At `grpc_v1.rs:301-310`, truncated history IDs use requested history length instead of the original history index.

Reproduction reasoning: retrieve the same completed durable task through both bindings. An artifact with a non-positional ID, description and metadata loses those values in gRPC. A completed task with only a status output yields an output artifact in JSON-RPC and none in gRPC. For a five-message history, the last message is `task:history:4` in a full gRPC read but `task:history:0` with historyLength=1. Thus clients cannot reliably reconcile the same message or artifact across retrieval settings and transports.

Correction: preserve durable fields and stable original message indexes; share equivalent completion projection rules. Exercise the same task through both bindings and multiple history lengths.

## WARNING W3 — unsupported streaming RPC starts work and returns a one-event stream

Evidence: `src/uar/api/a2a/grpc_v1.rs:466-490` calls `send_message` from `send_streaming_message`, then emits exactly one result. `subscribe_to_task` likewise emits only one current status. The advertised card declares streaming false (`src/uar/api/a2a/v1.rs:24`), and JSON-RPC returns streaming-not-supported for both methods (`v1.rs:268`).

Reproduction reasoning: authenticated `SendStreamingMessage` on gRPC can create a durable task, while the same method on JSON-RPC rejects without submission. With returnImmediately=true, the gRPC stream ends after its initial task even while execution continues; SubscribeToTask also ends before subsequent state changes. This is a side-effect and lifecycle mismatch, not implemented streaming parity.

Correction: return an explicit unsupported error before task submission on both unsupported gRPC streaming methods, or implement the complete promised streaming lifecycle and advertise it consistently. The current release contract declares streaming unsupported.

## Evidence receipts and limits

Read-only commands actually run:

- `git diff --stat c9dca170`: inspected candidate inventory, initially 63 tracked changed files, 1,521 insertions and 215 deletions; untracked production/fixture files inspected separately. Shared checkout can change after this snapshot.
- `git diff c9dca170 -- <scoped source paths>`, `cat`, `sed -n`, `nl -ba`, and `rg -n`: inspected source and requirements. These are inspection receipts, not test results.
- `git rev-parse HEAD`: `c9dca17084ab30a7c602df9fe99e5618fa42349e`; `git branch --show-current`: `codex/uar-1-1-0-kubo-conformance`.
- `/usr/local/bin/compass query 'A2A admission and per-agent MCP authorization' --graph compass-out/graph.json --scope source:src/uar --text-budget 1200`: exit 1, `store_graph_snapshot_failed: snapshot format is unsupported: expected compass.store.graph-index/1, found compass.store.graph-index/2`. No graph rebuilt; exact source was used.

No tests, builds, formatters, linters, compiler probes or acceptance scripts were run by this reviewer. No supplied runtime receipt establishes acceptance of this candidate. Installed-image behavior, zero peer contact, restart durability, schema conformance, package output and publication digests remain unverified here. Lead owns all build/test execution.

Reviewer: native `uar-verifier` / configured `gpt-6-astra`; isolated fresh-context task packet with artifacts and requirements only. Producer identities were not independently attested in this packet. A different model family was not used or proven; this is the weaker native same-family fallback, not a cross-family adversarial acceptance receipt. Further cross-family review remains pending if required for certification.

The uncomfortable limit: source evidence can establish inconsistent branches but cannot prove an installed image behaves exactly as predicted. This report must not be presented as a successful production acceptance run.

## Report integrity

Sycophancy screening: `detect_sycophancy`, strict, returned score 0.01785714365541935; one low-severity S-07 length warning and `correction_mandatory:false`. Manual review retained the evidence and reproduction detail needed for independent action. No automatic correction was claimed. No praise or defect quota was applied.

Only assigned change: `.agent-team/uar-core/reviews/uar-1-1-0-kubo-conformance/artifact-review.md`. No interface, dependency pin, implementation guard, production source or fixture was changed by this reviewer. No unrequested implementation additions.


## Single corrective review — 2026-10-15

**Current disposition: C1, W1, W2 and W3 are addressed in inspected source. No unresolved finding remains among these four at the static source level. Release certification remains withheld pending runtime acceptance and the recorded model-family review limit.** The initial findings above remain historical evidence. This was one continuous corrective review; no additional review cycle or execution gate ran.

The authoritative phase remains Execute. The corrected implementation inventory was read. The lead reported interrupted initial builds (exit 130), which are not accepted runtime evidence.

### C1 — addressed in source

`src/mcp/registry.rs:49-59` now compares scheme, effective port and host, treating accepted loopback host spellings conservatively as the same origin. Path, query and fragment cannot distinguish an unregistered declaration from a restricted origin. Initial admission invokes this helper at `src/uar/runtime/turn/host/mcp.rs:190-202`; live definition authorization uses it at `src/mcp/registry.rs:1204-1218`. Registered bindings retain their exact configuration-hash check and reject restricted entries without agent authority (`registry.rs:1192-1202`). That implements the documented distinction between administrator-registered sibling policies and unregistered aliases.

The fixture includes alternate path, query, fragment and loopback-host variants, requiring the specific denial code and unchanged peer trace (`tests/fixtures/uar_1_1_mcp_contract.py:175-221`). The supervisor invokes the full fixture against both an external gateway and a runtime-loopback relay in each sharing mode (`scripts/release/uar_1_1_acceptance.py:680-732`). The relay forwards to the same externally observed peer. Those are authored reachability controls, not observed successful connections. No residual bypass for the reported C1 cases was established in this source review.

### W1 — addressed in source

`src/uar/api/a2a/grpc_v1.rs:105-118` validates a nonempty tenant against the verified caller and never rewrites the agent ID. Send/Get/List/Cancel call that validation before task access. The fixture supplies the same explicit tenant to both transports and routes gRPC via `x-uar-agent-id`; mismatched-tenant checks require unchanged task inventory and tool-call count. `scripts/release/uar_1_1_loopback_client.py:61` supplies the actual configured tenant. The original tenant/agent conflation is removed.

### W2 — addressed after a residual mismatch was found during this corrective review

The first correction preserved artifact fields and original history indices but still generated `task:history:N` message IDs and exposed internal message-ID metadata. That discrepancy was reported and corrected before this review closed; the report does not accept the first correction as sufficient.

Final `src/uar/api/a2a/grpc_v1.rs:233-259` prefers the original internal message ID and removes that internal key from public metadata. Input conversion at lines 384-396 stores the actual incoming message ID, overriding any caller-supplied internal key. History enumeration occurs before truncation, and line 326 now uses `task:N`, matching JSON-RPC `src/uar/api/a2a/v1.rs:310-317`. Artifact ID, description and metadata are retained at `grpc_v1.rs:302-310`; completed-output projection starts at line 335.

The fixture compares complete and one-entry history/artifact projections for the same task across bindings (`tests/fixtures/a2a_v1_approval_governance.py:306-315`). The supervisor's transport matrix submits through each binding independently. The demonstrated original field-loss and message-identity mismatches are removed in final source; the fixture has not been executed by this reviewer.

### W3 — addressed in source

`src/uar/api/a2a/grpc_v1.rs:479-484,605-610` returns UNIMPLEMENTED immediately for both streaming methods, with no task-submission call. This agrees with the card's unsupported-streaming contract. The fixture checks SendStreamingMessage rejection, unchanged task inventory and no new tool calls.

### Corrective-review receipt and limits

Inspection used `cat`, `sed -n`, `nl -ba`, `rg -n`, scoped `git status --short`, and `git rev-parse HEAD`; commands returned source text, reported the review file as untracked, and retained baseline HEAD `c9dca17084ab30a7c602df9fe99e5618fa42349e`. No tests, builds, compiler checks, formatting or acceptance scripts ran. The graph-format and model-family independence limitations from the initial review remain.

Runtime no-contact, restart durability, elicitation, schema conformance and installed-image acceptance remain unverified here. The uncomfortable limit is that a corrected source path and an authored assertion are not a passing installed-image observation. The lead must retain actual production acceptance receipts before claiming the release passes.

Only this assigned report changed. No production interfaces, dependencies or guards were changed by the reviewer. The reviewed guards trace to the reported trust boundary and explicit protocol requirements; no unrequested implementation was added.

Sycophancy screening of the final corrective appendix: `detect_sycophancy`, strict, score **0.0**, no classifications, `correction_mandatory:false`. No automatic rewrite was claimed.
