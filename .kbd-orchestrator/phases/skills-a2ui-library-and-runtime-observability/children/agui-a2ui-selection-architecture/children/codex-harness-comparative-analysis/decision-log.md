# Decision log — codex-harness-comparative-analysis

### 2026-09-02T03:20:00Z — Analyze: build-vs-adopt calls
Mode: stack specified (Rust/tokio/axum/liter-llm/rmcp/Cedar). No contested stack.
Adopt: jsonschema 0.49.4 (already pinned) for tool-argument validation; backon 1.6.0 (already transitive) for retry/jitter/Retry-After; wiremock 0.6.5 + insta 1.48.0 as dev-deps.
Port (Apache-2.0, with attribution): codex normalize_history invariants; codex output-truncation; codex FunctionCallError shape; codex RwLock parallelism.
Build from design: prompt fragments and world-state diff; typed turn/step assembly with contributor traits; projected MCP lifecycle; thread-native subagent kernel with codex governance rules as requirements; AGENTS.md discovery.
Keep: rmcp =3.1.2 (V_2026_07_28 present, LATEST 2025-11-25); tiktoken-rs 0.12.0 model-keyed; arc-swap; walkdir; tokio-util.
Defer (no named failure): failsafe/recloser circuit breaker; rmcp bump; MCP Apps; A2A v1.0.1; AG-UI vocabulary retirement.
Reject: name-prefix effect inference; second event bus; vendor model catalog for base instructions; codex agent-identity; immature ag-ui/a2a crates; json-patch (no merge-patch generator).
Ranking: G1 context integrity, G2 fail-closed tools, G3 deterministic prompt, G4 skill runtime, G5 resiliency (immediate); G6 typed assembly, G7 MCP projection, G8 subagents, G9 project instructions (structural); G10 protocols (later). Differs from the supplied analysis by putting three seam-cutting correctness changes before typed-turn-assembly.
Provenance: research (Tier 1 gh, Tier 2 Context7, Tier 3 cargo search over cap by 5, local registry reads). docfork unreachable; deep-research server defunct.

### 2026-09-02T03:50:00Z — Analyze: adversarial review outcome
Round 1: BLOCK (1 CRITICAL external Codex paths; 2 WARNING A2A/AG-UI entries) → excerpt appendix + G10 entries added.
Round 2: BLOCK (1 CRITICAL observability decision missing; 1 WARNING maintenance evidence) → G11 added, maintenance criterion + table added. Applied after the two-round cap; not re-vetted. Sycophancy detect score 0.0.
Maintenance concern recorded: wiremock last push 2025-08-24 (dev-only).

### 2026-09-02T04:30:00Z — Spec: ten OpenSpec changes written
Backend openspec (spec-driven). All strict-valid. Adversarial review 2 rounds BLOCK→addressed; round-2 fixes not re-vetted (see spec-review-notes.md). Decisions made in spec: implicit skill matching activates only in `legacy_overlay` mode; time section compares at 1-minute granularity from a substitutable clock; concurrency keys: same conflicts, distinct/absent compatible, non-read-only exclusive; per-skill attribution counters instead of labeling totals; default flip to typed is its own change gated on parity + live smoke evidence; versions.toml is an operator precondition, never in a change's scope.

### 2026-09-02T04:40:00Z — Plan: ten changes in five rounds
Order: R1 parallel {context-history-integrity, fail-closed-tool-arguments (gated on versions.toml jsonschema), deterministic-prompt-assembly} with manager.rs boundary at :1477/:1478 and merge order 3→1→2; R2 {model-path-resiliency (gated on liter-llm error-type read), progressive-skill-runtime}; R3 typed-turn-assembly; R4 {projected-mcp-runtime (gated on sandbox decision), thread-native-subagents, project-instructions-world-state}; R5 typed-turn-default-flip (gated on parity + live smoke). Complexity per routing task-count rule: nine High/frontier, one Medium. wiremock adoption conditional on liter base-URL override; insta adopted in change 3. Adversarial review: round 1 BLOCK (1 CRITICAL missing pre-Round-2 gate, 3 WARNING) all fixed; round 2 PASS with zero findings (anti-theater gate skipped because sycophancy.sh lib is absent; round-1 substance is the evidence the judge is not rubber-stamping). Sycophancy detect 0.0.

### 2026-09-02T05:10:00Z — Execute: task 3.1 deviation (enum collapse)
Task 3.1 said to make `uar::domain::context::ContextStrategy` the only enum. Static evidence shows the opposite dependency direction: `uar::context::ContextStrategy` is operator-facing and persisted (AgentPolicy/EffectiveRunPolicy `src/uar/domain/policy.rs:186,394,419`), mirrored by the compiler IR with a conformance check (`src/uar/compiler/ir.rs:805-820`), rendered on the A2UI policy surface (`policy_surface.rs:176-205`), published in the settings schema (`settings/manager.rs:1833`), and read from config (`config.rs:232`). `domain::context::ContextStrategy` is internal, built only from `ContextConfig::default()` (`manager.rs:495`), never persisted. Executing 3.1 literally would break every persisted policy and the CH-14 conformance harness.
Options: (A) unify the reducer PATH only, keeping the operator-facing enum as the single declared strategy and driving the token-budget stage from it; defer type collapse to typed-turn-assembly, which owns the policy surface. (B) execute 3.1 literally plus a policy/IR/schema migration, which exceeds this change's scope and its stated boundary. Recommendation: A. Paused for operator decision rather than absorbed silently.

### 2026-09-02 — Execute: model-path-resiliency classification boundary
Vendored `liter-llm` 1.18.2 exposes public `status_code()`, `retry_after()`, `is_transient()`, and `error_type()` accessors on `LiterLlmError`; its HTTP request layer parses `Retry-After` before consuming the body. UAR will classify at each concrete driver boundary before the shared `anyhow` interface erases the concrete type. The liter driver will also return established stream-item failures as typed errors instead of successful normalized error events. `ClientConfigBuilder::base_url` is client-wide rather than a `ChatCompletionRequest` field, but UAR constructs that client from the resolved run configuration, so wiremock can cover the actual driver boundary. The upstream Azure base-URL defect and fix context is https://github.com/xberg-io/liter-llm/issues/83. Firecrawl's installed CLI has no `developer` subcommand, so the decision is based on the pinned vendored source plus official Context7 documentation.

### 2026-09-02 — Execute: reject unsupported MCP stdio sandbox requests

Task projected-mcp-runtime::0.1 explicitly permits rejection instead of a port.
Choose rejection. At Codex reference commit
`986ff1cc7ced0081ec5014b700a376333d87f869`, inspected files under
`/Users/gqadonis/Projects/references/codex/codex-rs/` show:

- `sandboxing/Cargo.toml` and `sandboxing/src/manager.rs:133`: launcher inputs
  depend on Codex permission profiles, managed-network context and platform
  helper configuration, not just a command and boolean.
- `sandboxing/src/seatbelt.rs:21`: bundled policy profiles and a trusted
  `/usr/bin/sandbox-exec` launch path participate in macOS enforcement.
- `linux-sandbox/Cargo.toml`, `linux-sandbox/src/bwrap.rs:1` and
  `linux-sandbox/src/landlock.rs:1`: separate Linux helper and filesystem-view
  construction with seccomp; Landlock helpers are legacy/backup utilities.
- UAR `src/sandbox/runner.rs:20` provides create/execute/file-I/O/destroy, not a
  persistent stdio launch interface. A port requires a host launcher and policy
  translation, not relabeling the existing unsandboxed child process.

Implement shared entry validation in `src/mcp/config.rs`, at config
deserialization, full-registry preflight and both launch paths; apply it before
HTTP/embedded administration writes, deferral, removal and hydration effects.
The error names the server and unavailable OS-backed stdio backend, without
command arguments or environment values. Rejection applies even to disabled
stored entries so later enablement cannot revive an unsupported promise.

The uncomfortable thing: users requesting sandboxed MCP stdio cannot run it
with this implementation. There is no automatic downgrade, native port or new
dependency. Existing unrequested stdio and remote transports retain behavior.
Server boot may still log the rejected configuration and continue with its
existing empty-registry fallback; this is not a claim that the whole UAR process
exits. Child-thread filesystem/network enforcement remains separate unfinished
work. Compile-only T0 passed without warnings; behavioral acceptance is
unverified until phase-end task 1.8 and the integration suite.

### 2026-09-02 — Execute: immutable MCP catalog and declared-input identity

projected-mcp-runtime::2.1 stores all source-qualified declarations without
connections. Authority derives from Global/Skill/Child provenance, rather than
an independently writable rank. The host supplies provenance and authentication
observations; no Deserialize surface allows server annotations to manufacture
either. Same-source conflicts fail instead of picking an arrival-order winner;
different-source candidates remain for task 2.2's policy projection.

The configuration hash covers raw declared transport/command/arguments/URL/env/
sandbox inputs with sorted map keys and unambiguous length framing. This keeps
equivalent declarations stable without reading mutable ambient environment.
Authentication revisions and the resolved environment must remain separate cache
key components in 2.3. The hash can encode low-entropy secrets, so even the digest
is omitted from telemetry-oriented Debug alongside command/env and binding IDs.

Uncomfortable limit: the catalog is exported but has no production consumer yet;
adding it alone does not deliver lazy startup, reuse or policy projection. Tasks
2.2, 2.3 and 4.1 remain open. Compile-only T0 passed without warnings in 28.92s;
phase-end integration behavior remains unverified. No dependencies changed.

### 2026-09-03 — Execute: source-pinned MCP step projection

Task 2.2 resolves servers before tool discovery and keeps those choices when
freezing step descriptors. EffectiveResourceSelection.ids is the eligible set
for All/Auto/Inherit too, not just Selected; None always denies. Provenance
eligibility uses active, policy-authorized skills and one host-specified child
identity. Global outranks skill, which outranks child. Highest-rank conflicts
fail; matching settings at that rank keep the stable first source.

Discovery snapshots must match that chosen source/config/auth metadata.
Unavailable higher-authority catalogs never trigger a lower-authority fallback.
Missing/partial catalogs fail explicitly, and differing complete snapshots
cannot be unioned because doing so can resurrect removed tools. Descriptor
fields are not rewritten; source/server associations and provider-name
collisions are checked before exposure. Hidden/denied tools are excluded,
Deferred tools are retained but not initially advertised.

Uncomfortable limit: these are metadata guarantees, not proof of matching live
connections or physical sandbox enforcement. Binding/cache and runtime wiring
remain tasks 2.3 and 4.1. The new module has no manager caller yet. Compile-only
T0 passed without warnings in 45.12s; phase-end behavior remains unverified.

### 2026-09-03 — Execute: owner-isolated single-flight MCP bindings

Task 2.3 keys exact verified user/tenant, server/source/config/required/auth
metadata and OS-string environment/cwd. One caller owns refresh; watch shares
completion, RAII clears cancelled attempts, and generation checks reject stale
publication. Ready bindings and returned single-server registries retain exact
identity. Owner invalidation revokes all revisions. No detached worker or retry.

Synchronous registry cancellation enables drop safety; the cache retains retired
leases until awaited cleanup proves transport closure. Read locks cover ready
lookups and short writes cover state changes; no lock crosses an await.

Uncomfortable limit: snapshot-aware launch/reconnect, lazy catalog completeness
and manager integration remain future tasks. Existing ambient-reading startup
must not be passed as a supposedly compliant snapshot connector. Final T0 passed
with zero warnings in 31.44s after correcting two compiler warnings; no tests or
acceptance review ran. This is cache implementation, not live reuse evidence.

### 2026-09-03 — Execute: captured stdio launch and reconnect inputs

Task3.1's stdio connector consumes captured cwd/env/PATH, discovers all tool pages
within a bounded budget and retains the complete catalog in the shared reconnect
slot. Reconnect rejects changed descriptors before generation-checked install.
Administrative replacement clears the captured inputs with config replacement.
A drop guard clears reconnect accounting on cancellation, needed when the new
total lazy-call deadline expires mid-reconnect.

Uncomfortable limit: HTTP and host-joined partial-launch cleanup remain open.
The dependency-pin skill requires an operator reqwest_mcp="0.13.4" entry before
adding an alias for the SDK's already-locked HTTP client; that pin is pending.
No dependency edits or stdio-only task-completion claim. T0 passed zero warnings
in32.21s, with no behavioral testing or acceptance review.

### 2026-09-03 — Execute: supervise partial stdio startup through shutdown

TL;DR: retain direct-child cleanup independently of the handshake future, using
a host-owned TaskTracker and cancellation-aware transport. Initial captured
launch and reconnect share that supervisor. Runtime shutdown now awaits the
connector as well as the cache and reports OS cleanup failures.

Why: a cancelled lazy-start attempt may never publish a registry, so registry
closure alone cannot prove that its child was reaped. A short admission lock
prevents registration racing the shutdown barrier. No detached cleanup owner,
new dependency, process-tree sandbox or unrelated legacy rewrite was added.

Uncomfortable thing: this is compile-only evidence (T0 exit0, zero warnings,
26.57s); real process and end-to-end proof is deferred to phase end. Task3.1 is
still incomplete, pending the HTTP alias pin and adapter. No completion hook.

### 2026-09-03 — Execute: required/optional MCP preflight without authority fallback

TL;DR: prepare the selected server identities for one verified owner. Required
availability failures abort with an actionable error; optional ones emit named,
secret-free warnings and remove their exact descriptors. The input authority
winners and tool eligibility cannot be replaced by lower-origin declarations.

Why: optional means an unavailable service may be omitted, not that ownership,
generation revocation, shutdown or projection invariants may be ignored. Those
errors abort. Complete cached skill/child catalogs preserve lazy startup.

Uncomfortable thing: task3.2's runtime API has cross-module call sites and clean
T0 evidence (zero warnings,31.79s), but RunManager wiring and behavioral evidence
remain tasks4.1 and phase-end tests. Task3.1 stays open for the HTTP alias pin;
this independent task does not waive that requirement or the remaining scope.

### 2026-09-03 — Execute: generation-bound MCP lifecycle events and metrics

TL;DR: exact cache bindings own ordered normalized lifecycle publishers. Initial
startup, refresh, reconnect, cancellation, invalidation and shutdown use them;
the existing bool status recorder follows the same publication order. AG-UI
adapters accept the new secret-free typed payload.

Why: a late reconnect must not publish Ready after shutdown, nor may an observer
keep a dropped publisher alive. Generation checks, terminal ShuttingDown,
single-flight reconnect admission and Weak subscriptions address those concrete
ownership/order scenarios. Event lag is explicit; resync is not replay.

Uncomfortable thing: task3.3 has compile-only evidence (zero warnings; final T0
11.95s). RunManager forwarding remains task4.1; task1.7 and live integration
tests remain at phase end. The legacy server-name metric is not per-owner
aggregation. No new dependency, credential exposure or execution authority.

### 2026-09-03 — Execute: bounded MCP exposure with next-step discovery

TL;DR: keep immutable descriptors and a separate per-stream visibility map.
At most32 MCP tools are advertised; model-only search selects at most8 deferred
matches for later steps. Hidden remains absent. The real orchestrator freezes
one visible map for both advertisement and execution of the entire batch.

Why: changing descriptor metadata would interfere with exact binding identity;
letting search expand the current batch would make its execution scope differ
from the model request. Chat-local handler registration prevents cross-stream
selection sharing. Search does not connect servers or grant permissions.

Uncomfortable thing: the production path compiles (final T0 zero warnings,
11.30s), but task1.4 and phase-end integration must prove actual200-tool discovery
and same-batch rejection. HTTP3.1 and binding-cache integration4.1 remain open.

### 2026-09-03 — Execute: prepared MCP dispatch and concrete delegation handoff

TL;DR: retain the exact preflight in activation and model-step snapshots; use it
for governed dispatch and descriptor-based outcome attribution. Explicit child
handoff waits for exact bindings and freezes transports, not configuration.

Why: routing a prepared tool through the old registry could select another
connection; deriving usage from the old registry would miss projected tools.
Frozen capture now checks every discovery page and retains leases until final
revocation checks. No additional permission or silent optional omission is added.

Uncomfortable thing: the integration entry points are not the completed root
migration. new_projected and freeze_mcp_bindings still need root callers; graph,
policy-universe, event and shutdown wiring remain. Task4.1 stays open at7/22.
Latest T0 checks passed zero warnings in19.55s,9.55s,7.35s; tests stay phase-end.

### 2026-09-03 — Execute: verified ingress owner retained through MCP assembly

Keep the middleware/actor ActorOwner through actual and shadow turns; reject a
prepared MCP owner mismatch even when tool descriptors match. No principal is
minted from a user ID string. HTTP create/resume/checkpoint/chat and actor
mailboxes are real callers. Anonymous/legacy paths remain explicitly unverified.
Uncomfortable thing: root shared-cache wiring and legacy host isolation are
still unfinished. Task4.1 stays open; four T0 passes, no behavioral tests.

### 2026-09-03 — Execute: captured root MCP bundle consumed by RunManager

Manager now calls projected activation for a supplied host bundle and never
falls back after its preflight fails. Owner/cwd/policy/child checks precede
session mutation. Initial and subsequent activations share run cancellation.
Uncomfortable thing: bootstrap does not construct the bundle; graph support is
explicitly unavailable on that new path. Default migration and graph support
remain required, not deferred from scope. T0 clean34.12s and13.40s; task4.1 open.

### 2026-09-03 — Execute: retain actor joins through server shutdown

Wire server cleanup to shutdown_all; close actor admission and retain each join
handle until completion even when its HTTP stop waiter is cancelled. Remove only
the stopped Arc so a reused actor name cannot be removed by an older request.
Uncomfortable thing: collaboration still starts a root, not a child, and the
process hard deadline can interrupt stuck cleanup. Thread4.1 remains open;
T0 clean17.42s,25.05s,15.73s, with behavior tests deferred to the phase end.

### 2026-09-03 — Execute: captured child kernel and joined thread-service closure

Root-owned resource leases revoke on completion/unwind; a weak run index provides
owner-qualified capture. The child entry reuses frozen MCP and model clients,
root approvals and canonical cwd. Run-specific dialogue prevents following a
newer conversation's messages. ThreadService joins jobs and retains failed
receipts before settling child records, with no blind persistence retries.
Uncomfortable thing: there is no concrete admission host or actor/graph/A2A
capture caller yet. This is not completed delegation. Task4.1 remains open;
six warning-free T0 passes after fixing a text-extraction type mismatch. No tests.

### 2026-09-04 — Execute: typed-default evidence gate satisfied

The three-case corpus report and two-case live shadow receipt are attached under
`openspec/changes/typed-turn-default-flip/`. Both report zero unexpected and
allowlisted differences. Live k3 default-agent runs covered basic input and host
instructions, each with completed text and one comparison dispatching legacy.
The exact command exited 0 and is recorded in `evidence/README.md`; run IDs are
in `evidence/live-shadow-report.json`. The pre-flip gate is met. Full Tier 2
verification must still pass after changing the default. Legacy remains an
explicit rollback for one minor release; shadow becomes opt-in.

Uncomfortable thing: this small smoke set does not cover live memory, MCP,
active skills, multi-step tools, remote peers, or other providers. The separate
real-child cancellation receipt contains no shadow comparison and is not used
to justify this decision.
