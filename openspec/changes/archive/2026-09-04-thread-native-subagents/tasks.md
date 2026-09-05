# Tasks — thread-native-subagents

scope: src/uar/runtime/thread/**, src/uar/runtime/actor/**, src/uar/runtime/graph/**, src/uar/api/a2a/**, src/uar/api/actors.rs, src/uar/persistence/**, src/uar/domain/events.rs, src/uar/api/adapters.rs, src/uar/api/sse.rs, src/uar/runtime/cost_budget.rs, src/uar/runtime/manager.rs (graph branch, budget), src/uar/runtime/native_skills/agents/**, tests/agent_threads.rs, tests/agent_policy_intersection.rs, tests/a2a_thread_service.rs

## 1. Failing tests first

- [x] 1.1 `tests/agent_threads.rs`: a spawned child persists an `AgentThread` and `AgentEdge` in memory, SurrealDB, and PostgreSQL providers with identical ordering on list
- [x] 1.2 A child spawned with `HistoryForkMode::None` sees only its delegated prompt; `LastTurns(2)` sees two turns and no tool outputs
- [x] 1.3 `tests/agent_policy_intersection.rs`: a child artifact granting a tool the parent policy denies yields a child policy without it; an unsupported policy shape fails closed
- [x] 1.4 A child cannot cause an approval to be granted; approval requests bubble to the root run's approval channel
- [x] 1.5 The fifth concurrent child, the fourth nesting level, and the seventeenth total child are refused with typed limit errors
- [x] 1.6 Root cancellation cancels every child; a remote A2A child receives `tasks/cancel`
- [x] 1.7 Child model calls are recorded against the root run's budget; an exceeded budget refuses new spawns and new model calls
- [x] 1.8 `send_agent_message` delivers a typed message whose sender identity is metadata; the child's user turn contains no prepended identity text
- [x] 1.9 `tests/a2a_thread_service.rs`: inbound `message/send` starts a run on the named agent artifact; `tasks/get` reflects thread status; `tasks/cancel` cancels the thread; wire shapes unchanged
- [x] 1.10 Lifecycle events carry parent id, child id, canonical path, status, and terminal outcome and no prompt text; AG-UI subagent events are emitted from them
- [x] 1.11 Actor endpoints without user context return 401

## 2. Kernel

- [x] 2.1 Add `thread/mod.rs` (`AgentThread`, `AgentEdge`, `AgentHandle`), `thread/spawn.rs` (`AgentSpawnRequest`, `HistoryForkMode`, fork filtering that keeps only final assistant messages and user turns), `thread/messages.rs` (`InterAgentMessage`), `thread/limits.rs`
- [x] 2.2 Add `thread/policy_intersection.rs`: intersect parent and child artifact policies for skills, MCP servers, tools, credentials, sandbox, budgets; fail closed
- [x] 2.3 Persistence trait methods and three provider implementations with stable ordering

## 3. Tools and events

- [x] 3.1 Descriptors for `spawn_agent`, `send_agent_message`, `wait_agents`, `list_agents`, `interrupt_agent`; spawn description states that spawning requires explicit user or artifact authorization
- [x] 3.2 Lifecycle event variants; AG-UI subagent mapping; live `RuntimeStep` for graph runs

## 4. Adapters

- [x] 4.1 `AgentActor` as a mailbox over the kernel; actor API requires user context
- [x] 4.2 `AgentNode` spawns and waits on a real child thread; remove the synthesized two-message call and the `[route]` prefix
- [x] 4.3 A2A handler maps onto the thread service; client propagates cancellation

## 5. Budget and cancellation

- [x] 5.1 Record child usage against the root run; enforce `Exceeded` on spawn and model call
- [x] 5.2 Child cancellation tokens from the run token; A2A cancel on parent cancel

## 6. Verification

- [x] 6.1 Tier 1: the three new test targets
- [x] 6.2 Tier 2: fmt check and full test run
- [x] 6.3 Local multi-agent cancellation smoke with a real model (record command and output)
- [x] 6.4 `openspec validate thread-native-subagents --strict`

## 7. Independent phase-end audit corrections

- [x] 7.1 Repair never-dispatched remote reservation ownership and add host-path regressions

The independent audit found that tree admission failure and cancellation before
first remote dispatch could retain a budget reservation without a remote task.
Perform fallible local admission before leasing remote capacity; release a lease
on contract rejection and when the host proves no execution was admitted. Keep
uncertain dispatched work reserved. Exercise both paths through ThreadService,
including joined shutdown. Build the full audit correction batch before running
its integration tests. Prior passing receipts remain historical evidence, not
coverage of this defect.

## Execution note — 2026-09-03, governed UAR peer task adapter (4.3)

The inbound A2A service now maps authenticated, owner-qualified v2 delegation
contracts onto the persisted actor/thread host. The signed peer instance claim
must match the contract source, the configured target instance and named agent
must match exactly, and every governed receipt acknowledges the contract digest.
The source persists the peer endpoint plus task/context identity after the first
receipt and continues that exact remote task for follow-up turns.

The client retains send, poll, cancel and reconciliation futures in one owned
execution object. A lost known-task send can still be cancelled; an uncertain
cancel is never replayed and uses read-only reconciliation. Execution terminality
is distinct from actor closure. Governed cancellation requires the peer's
persisted `uar.cleanup_closed` marker before source capacity is released, and
concurrent cleanup callers serialize on the same retained execution.

Tier 0 `cargo check --locked --no-default-features --features server-full`
passed with zero warnings after the final source state (13.89s). Targeted
`git diff --check` produced no diagnostics. A fresh artifact-only critic found
no remaining P0-P2 issue in the reviewed A2A settlement and budget paths. Per
the phase policy, no tests, test authoring, formatter, full build, or strict
OpenSpec validation ran; those remain phase-end work.

Uncomfortable limit: contract acknowledgement and cleanup markers prove the
trusted peer's protocol response, not remote attestation. A peer that closes
without usable usage metadata is treated as having consumed every finite
reserved dimension. This is intentionally conservative and can exhaust a
source budget after a malformed receipt. Runtime network and cancellation races
remain unverified until the phase-end test tasks.

## Execution note — 2026-09-03, shared local and remote budget enforcement (5.1)

Model and tool attempts now enter one root-owned `CostBudgetTracker` before
execution. Local descendants retain the root payer, request/token windows,
tool-call count, cost scopes and deadline. Governed remote children reserve the
remaining finite root capacity before dispatch, so another local or remote
child cannot spend the same allowance while the peer is active.

The v2 peer contract keeps target-local `ThreadBudgets` separate from a
source-owned cumulative `UarUsageGrant`. Per-turn token/tool counters use the
actual target run, rolling request/token windows use the stable contract
accounting identity across follow-up turns, and cumulative target usage is
returned in authenticated terminal receipts. The source charges monotonic
actual usage, including over-grant usage, and holds the reservation until actor
closure. Missing/malformed usage charges every finite reserved dimension before
release. Settled remote traffic is conservatively entered into the source
rolling window at confirmed close because peer-side timestamps are not trusted.

Tier 0 `cargo check --locked --no-default-features --features server-full`
passed with zero warnings after the final accounting state (13.89s and 1.52s
incremental passes). The artifact-only critic found no remaining P0-P2 issue in
the reviewed budget/settlement paths. Tests and test authoring remain deferred
to section 6 by operator instruction.

Uncomfortable limit: one model response can physically consume more than a
small remaining token/cost grant before the provider reports usage. That
overage is recorded, fails the run, and is charged to the source, but cannot be
prevented after inference has occurred. Missing peer usage intentionally burns
the full finite reservation. Runtime enforcement remains unverified until the
phase-end integration tests.

## Execution note — 2026-09-03, root-scoped child cancellation (5.2)

Every child receives a descendant token from its parent thread, whose root is
the captured run token. Root shutdown cancels the tree, closes new admission,
joins accepted host jobs, and then settles persisted child state. Local child
execution races cancellation through the shared kernel and retains cleanup
ownership until completion.

Remote cancellation drives `tasks/cancel` on the exact persisted peer task.
Pending send/cancel futures and uncertain reconciliation remain owned by the
thread service if a waiter disappears. Idle cancellation creates and stores one
cleanup execution before its first await; interrupt and shutdown callers share
that operation. Confirmed peer close, cumulative accounting and reservation
release serialize under the same execution mutex, and later shutdown skips an
already released reservation.

Tier 0 `cargo check --locked --no-default-features --features server-full`
passed with zero warnings in the final source state (13.89s). The final
artifact-only adversarial review reported no remaining P0-P2 findings in these
paths. No tests or test authoring ran; section 6 remains the phase-end gate.

Uncomfortable limit: if a remote cancellation receipt is lost and read-only
reconciliation never observes `uar.cleanup_closed`, the source intentionally
retains the operation and capacity instead of replaying cancellation or
claiming cleanup. Process hard-stop can still end that wait. Phase-end tests
must exercise dropped waiters, simultaneous interrupt/shutdown, and live peer
cancellation before runtime behavior is certified.

## Execution note — 2026-09-03, actor shutdown integration (4.1 partial)

The server's async cleanup now calls ActorCollaboration::shutdown_all before
closing shared MCP resources. ActorCollaboration owns a child cancellation token
that permanently closes actor admission without cancelling unrelated manager
runs. A spawn already holding the registry lock finishes publication into the
shutdown snapshot; later spawns fail before launching a mailbox.

Actor handles remain registry-owned through joining. stop_actor and shutdown_all
await a borrowed JoinHandle under its own mutex and remove the registry entry
only after completion. Cancelling an HTTP stop/cleanup future therefore leaves
the handle available to subsequent cleanup. A completed stop removes only its
exact Arc, not a newer actor that reused the same public name. Registry locks
are not held while joining actors.

Compile-only Tier 0: cargo check --locked --no-default-features --features
server-full passed with zero warnings after each source group: 17.42s, 25.05s,
15.73s. Targeted git diff --check exited 0 without output. No tests, test
authoring, formatting, strict validation or acceptance critic ran; these remain
phase-end work. No dependency, pin, workflow, commit or deletion operations.

Uncomfortable limit: task 4.1 is still incomplete. Actor collaboration still
starts an independent root; the concrete ThreadExecutionHost and root resource
attachment are not wired. This shutdown change does not establish remote A2A
cancellation, child budgets, or graph delegation. The existing process shutdown
deadline/hard-stop remains in effect and can interrupt stuck cleanup. Runtime
race and shutdown behavior are not yet tested. No checkbox or end-task hook.

## Execution note — 2026-09-03, captured child kernel and joined service shutdown

manager.rs now retains a weak reference to each verified root's actual model,
skill, native, activation, harness, cwd and approval resources. A root-owned
RunDelegationLifetime cancels delegated execution on completion/unwind even if
a borrower retains the resources. Anonymous and inherited child runs do not
publish root captures. The canonical cwd comes from WorldStateRuntime's new
read-only directory accessor, not another environment lookup.

New thread/kernel.rs provides CapturedThreadKernel through the manager's
owner-qualified capture_thread_kernel API. It verifies the live committed root
before and after MCP freezing, snapshots native bindings, resolves exact named
artifacts, reads owner/root-qualified history, and executes a HostedThreadTurn
through execute_request_inner with Some(InheritedRunBindings). It retains the
verified tenant owner, narrows resources, uses captured model clients and root
approvals, splits the final text input once, and awaits producer completion on
explicit cancellation. A dropped execution future cancels its own token.

RunDialogue records this kernel's dialogue/tool outputs separately from the
mutable conversation session. The normal conversation writes remain intact;
thread history cannot switch to another run's messages when that conversation
advances. History lookup admits the exact committed thread or the immediately
preceding terminal turn while ThreadService prepares its next turn. Missing
records/history do not become an empty-history fallback. Dialogue snapshots
are in-process, not a new durable history recovery mechanism.

ThreadService::shutdown closes job admission, cancels the tree, joins all
accepted jobs, then settles pending/live child records. Join slots remain owned
if cleanup is cancelled; failure receipts remain visible to later callers.
Completed jobs are joined before pruning, and a panicked host operation closes
admission. Uncertain writes are read/reconciled, never retried blindly.

T0 cargo check --locked --no-default-features --features server-full: initial
E0308 identified an Option<&str>/String mismatch in delegated text extraction;
fixed with explicit MessageContent::Text matching (no multimodal data loss).
Subsequent passes were warning-free: 31.13s,24.17s,31.00s,12.81s,17.08s,31.77s.
No tests, test authoring, fmt, strict validation, acceptance critic, dependency,
pin, workflow, commit or deletion operations. Tracked diff check clean.

Uncomfortable limit: no actor/graph/A2A caller yet invokes capture_thread_kernel,
and no concrete ThreadExecutionHost implements sandbox/budget admission. The
kernel entry is not an admission bypass: its caller must first perform those
host checks. Root ThreadService attachment and its shutdown caller, policy and
permission capture, control-tool installation, actor collaboration, graph/A2A,
and the remaining budget/cancellation work are still required. Task4.1 stays
unchecked,5/25; no task-end hook. Runtime behavior remains phase-end verification.

## Execution note — 2026-09-03, shared root budget admission (5.1 partial)

CostBudgetTracker now records known cumulative token usage as well as cost in
the captured root run/session/agent/global scopes. ModelCallBudget carries
strictly decoded ThreadBudgets and the root entry clock; child limits intersect
without resetting the payer, counters, rate windows or deadline. Repeated usage
events do not double-charge tokens; token corrections retain the observed high
water mark, while cost corrections still replace the cumulative estimate.
Model attempts reserve the root request-rate slot atomically. Known token rate,
turn/session tokens, session cost, configured scope costs and the root deadline
gate subsequent requests. Stream start/consumption also race the deadline.
Finite cost limits reject models without catalog pricing before calling them.

RunModelBindings retains raw captured clients and applies exactly one budget
wrapper at use, including fallback and child model rebinding. manager.rs parses
root budgets before capture and gates approved or governance-bypassed tool
attempts against the shared root allowance. Governance-rejected tools consume
no allowance. The legacy assignment of max_cost_per_session_usd to Agent scope
was removed: unrelated sessions must not share an artifact's session ceiling.
The old permissive parser remains test-only for its existing tests.

ThreadService::attach now requires the actual CapturedThreadKernel, checks the
exact committed root, and independently rechecks its budget before reserving a
spawn, admitting triggered turns, and entering each child turn, including after
history/preflight awaits. Adapters cannot replace this with a no-op admission
callback. The separate sandbox/binding callback remains mandatory.

T0 cargo check --locked --no-default-features --features server-full passed five
times with zero warnings: 35.14s,20.72s,15.31s,13.61s,12.11s. No tests or test
authoring, fmt, strict validation, acceptance critic, dependency/pin/workflow
changes, commits or deletions. Runtime behavior is not verified by compilation.

Uncomfortable limit: task5.1 remains unchecked. No actor/graph/A2A caller yet
attaches ThreadService or invokes capture_thread_kernel; completing helpers is
not a delivered child path. Concrete sandbox enforcement and adapter attachment
remain necessary. Budgets admit against observed usage, not prepaid reservations:
already in-flight calls can overshoot, missing provider usage cannot be billed
accurately, and dropping a local stream does not prove remote billing stopped.
The deadline applies to model streams and subsequent tool/thread admission, not
yet to forcibly stopping an already-running native/remote tool. Phase-end tests
must cover cumulative usage, fallback/child wrappers, root scope isolation,
concurrent admission, missing usage and deadlines. No task-end hook was emitted.

## Execution note — 2026-09-03, sandbox execution boundary (4.1 partial)

Observed prerequisite defects: manager did not wire artifact execution_mode;
the orchestrator treated Sandboxed like Auto, ignored descriptor isolation in
Direct mode, and fell through to direct native/MCP execution when no runner or
code extraction existed. WasmtimeRunner currently launches normal host processes
and does not enforce the isolation its name suggests. Do not use it as proof of
child sandbox enforcement.

Orchestrator now applies artifact mode to actual dispatch: Sandboxed requires
isolation for every call, Auto for code execution, and descriptor sandbox_required
always applies. The parallel fast path obeys the same predicate. Missing isolation
or an unsupported adapter yields a bounded failed ToolResult and paired history,
never direct fallback. SandboxRunner has an explicit default-false isolation
contract. RemoteRunner declares the configured service as its trusted boundary;
that declaration is not remote attestation. The process-based runner remains
unmodified and cannot satisfy governed sandbox-required execution.

NativeSkill::sandbox_request replaces tool-name/argument-field code guessing.
TerminalExecTool explicitly adapts sh,/bin/sh,bash,/bin/bash with shell-quoted
command input, supplied env/cwd and an ephemeral request. Unsupported shells or
tools do not receive guessed adapters. Both direct and sandbox terminal requests
cap caller timeouts at the configured ceiling. Other sandbox adapters remain to
be implemented; rejecting them is not equivalent to delivering their behavior.

Server startup now resolves the existing remote sandbox env configuration once
through configured_isolated_runner, validating the backend/base URL without
printing credentials. No endpoint means no isolation backend; explicit unsupported
configuration fails startup. RunManager retains the client and root captures pass
its exact Arc to children. No child environment credential lookup is added.

T0 cargo check --locked --no-default-features --features server-full passed with
zero warnings after each cohesive group:24.77s,25.46s,23.50s. Targeted tracked diff
check passed with no diagnostics. No tests/test authoring/fmt/strict validation/
critic/dependencies/pins/workflows/commits/deletes. No remote server was contacted.

Uncomfortable limit: task4.1 is NOT complete. Concrete physical permission
binding capture/enforcement and actor/graph/A2A ThreadService attachment are still
missing. The newly connected remote route also needs owned sandbox-operation
cleanup: the existing inline create/execute/destroy sequence loses its handle
when the outer model stream is cancelled, ignores destroy errors, and cannot
reconcile a lost create response. Do not call this cancellation-safe or ready
for acceptance. Complete that lifecycle before accepting the host implementation.
Actual remote isolation, command semantics and race behavior await phase-end
integration tests. Counter remains5/25; no task-end hook or completion claim.

## Execution note — 2026-09-03, owned sandbox lifecycle and immutable configuration (4.1 partial)

Supersedes the preceding note's inline-lifecycle gap, not its task-completion
limits. sandbox/execution.rs retains every ephemeral create/execute/destroy job
in a manager-owned supervisor. Dropping the tool future requests cancellation;
it does not drop the job that owns the remote handle. Cancellation/deadline after
creation skips or interrupts execution and still awaits destruction. Creation
and destruction are attempted once: lost responses retain unconfirmed receipts,
not retries or fabricated cleanup success. Backend panics are caught, known
handles are retained, and execution panics still enter cleanup. The root budget
deadline is passed from cost_budget.rs into the owned operation.

RunManager drains the scope before terminal completion and in its caught-unwind
finalizer; RunCompletionGuard can override a prior terminal result with failure.
Server shutdown drains outstanding scopes after actor shutdown. Unconfirmed
cleanup makes the run fail, including when its caller requested cancellation.
Content-free diagnostic snapshots retain operation/run/backend/sandbox identity.
Source inspection found one join-cancellation gap: worker failure inspection
awaited a receipt lock before publishing handle consumption. Consumption and a
conservative outcome are now saved first, so a cancelled waiter cannot cause a
completed JoinHandle to be polled again.

sandbox/bindings.rs captures the exact backend/configuration before a root run.
RunDelegationBindings and InheritedRunBindings retain that object; child kernel
entry narrows its network and opaque environment grants. The execution scope
supplies configuration, not the tool caller. Backend substitution and overriding
or reintroducing protected host environment entries fail before remote creation.
The current production profile is the existing default: network disabled, no
host mounts/environment. The untyped volumes map has no represented access-mode
contract, so nonempty mount configurations/filesystem grants are rejected rather
than claiming read-only enforcement. This does not constrain direct native tools.

Observed current T0 command: cargo check --locked --no-default-features --features
server-full. Baseline pass 1.96s; join fix pass 24.60s; standalone binding compile
36.16s reported two unused-code warnings; integration removed those warnings and
passed in 20.31s, exit0, zero warnings. The old session73429 output was unavailable,
so the baseline was rerun rather than claiming its result. Tracked diff check
exit0/no output; new-file no-index checks exit1 (file differences)/no whitespace
diagnostics. No tests, test authoring, formatter, strict validation, acceptance
critic, dependency/pin/workflow changes, commits, deletion or external calls.

Uncomfortable limit: remote isolation is a configured service contract, not
attestation; a lost create/destroy response remains unresolved, and an existing
process hard shutdown can interrupt waiting. Runtime race/remote behavior is
unverified until phase-end tests. Concrete ThreadExecutionHost admission, direct
tool permission enforcement, one root ThreadService attachment, actor child
collaboration, graph/A2A adapters and other sandbox adapters remain unfinished.
Task4.1/5.1 remain unchecked; thread5/25, total90/182, implementation2/10, round4/5.

## Execution note — 2026-09-03, concrete thread host and native admission (4.1 partial)

Observed session_search defect: it always loaded the anonymous owner's session,
including when called by a verified actor. NativeExecutionContext now retains
the ResolvedTurn's verified owner. SessionSearchTool requires that context, uses
the owner-qualified persistence lookup, and verifies the returned identity.
Uncontextualized/anonymous search is rejected; caller arguments cannot supply a
replacement principal. Delegated memory=false also rejects session search.

NativeSkill now has explicit check_thread_policy and supports_sandbox_execution
contracts. Direct delegation defaults to rejection, not inference from ReadOnly.
The trusted execute_native wrapper checks the inherited owner/policy before the
overridable tool method, in both sequential and parallel orchestrator dispatch.
Echo and fixed platform facts consume no filesystem/environment grants; session
search checks memory policy and verified identity. Terminal advertises only its
implemented sh/bash sandbox adapter, not direct delegated process enforcement.
Legacy in-process NativeTool implementations inside McpRegistry lack contextual
permission support and are rejected for delegated calls until ported.

Agent controls check exact policy identity and operation authorization. Activation
and discovery retain their constructed turn's policy. Child registry filtering
now excludes search_tools as well as activation/agent handlers; otherwise an
equivalent descriptor could retain its parent's discovery state. These handlers
are recreated from child-local state rather than shared across turns.

CapturedThreadKernel now implements ThreadExecutionHost. Admission validates live
owner/root, resource subsets and limits, captured model routes/budgets, exact
frozen MCP server/tool/opaque credential identities, and sandbox availability.
Direct native policy checks still run at each call before I/O; unsupported tools
are not claimed to have full delegated implementations. root_policy derives the
authority from the retained original artifact and actual captured bindings.
RunDelegationBindings now retains that original artifact and a shared atomic
attachment claim. ThreadService::attach takes only kernel/events/user-grant;
policy/artifact/persistence/cancellation and its concrete executor come from the
kernel capture. A second attachment, including through a fresh kernel capture,
cannot reset tree counters. A failed attachment attempt is not retried in-place.

T0 cargo check --locked --no-default-features --features server-full passed with
zero warnings after each cohesive group:27.49s,22.50s,17.15s,24.94s,21.51s. Targeted
tracked diff check exited0 with no diagnostics. All build handles terminal.
No tests/test authoring/fmt/strict validation/critic/dependency/pin/workflow/
commit/delete/external-service work. No unrelated feature added.

Uncomfortable limit: the prior statement that a concrete ThreadExecutionHost is
absent is now superseded, but ACTOR ATTACHMENT IS STILL ABSENT. No production
caller invokes capture_thread_kernel or ThreadService::attach. Actor collaboration
still starts an independent target root. Next wire actor startup before model
launch, root control eligibility/registration, retained service lifetime and joined
root closure, then true actor child collaboration. File/patch/web/compiler/A2UI
and legacy MCP-native permission ports remain; rejecting unsupported calls is not
equivalent to implementing them. Runtime owner isolation, native policy checks,
handler rebinding and attachment races are unverified until phase-end integration
tests. Task4.1/5.1 remain open; counters unchanged; no task-end hook.

## Execution note — 2026-09-03, actor root attachment and real collaboration (4.1 partial)

ActorThreadSession now hands its exact committed root and retained persistence
provider to RunManager. Before session mutation, the manager checks the record,
verified owner, artifact and run identity, and confirms the exact stored revision.
Actor roots include the five built-in control names during ordinary policy
resolution; wildcard eligibility still does not explicitly authorize spawning.

The manager publishes the executable capture before context/model assembly,
attaches ThreadService once, and installs its fresh root handlers before collecting
authorized descriptors and the manifest. The native capture is copied before
installing those handlers, avoiding service/kernel/registry ownership cycles.
Known control factories have host-owned identities, not dummy global handlers.
Root capture validates actual bindings without imposing child adapter admission
on existing root tools; every child still passes the stricter execution checks.

ActorCollaboration now uses the source actor's live, prepared root service, checks
Cedar, and spawns a real child on the target artifact with HistoryForkMode::None.
The authenticated endpoint records explicit root-user delegation authorization;
it does not approve child tools or widen selected resources. The shared root tool
allowance applies. Replies carry persisted lineage/result metadata; mailbox-only
Collaborate messages cannot create a replacement independent root. Idle/terminal
source actors fail explicitly rather than synthesizing another root.

Normal/cancelled root completion drains the child service before terminal events;
the unwind finalizer drains it before releasing completion. ActorRootBinding owns
the producer JoinHandle, published with no await gap after launch. Actors join it
before replying; stop/shutdown retain unresolved root/child receipts rather than
removing a finished mailbox's handle. Abandoned HTTP waits do not own child jobs.

T0 cargo check --locked --no-default-features --features server-full:
an initial unused-persistence warning was fixed by the exact commit read; a
governance field-name E0609 was corrected. Subsequent passes were zero-warning:
13.29s,20.77s,23.22s,23.18s. All checks terminated. Tracked diff check exit0/no
output; no-index checks for new actor_host/kernel/service files returned diff
exit1 with no whitespace diagnostics. No tests/test authoring/fmt/strict
validation/acceptance critic/dependency/pin/workflow/commit/delete operations.

Uncomfortable limit: task4.1 remains open. File/patch/web/compiler/A2UI and legacy
MCP-native delegated permission ports are still incomplete; rejecting their
execution is not the finished feature. Actor attachment, child collaboration,
ownership races and cleanup behavior have compile evidence only. Graph4.2,
A2A4.3 and remaining budget/cancellation work remain in scope. Do not repeat the
superseded statement that actor attachment/collaboration have no source callers.
Counters remain round4/5,2/10changes,90/182tasks,thread5/25. No task-end hook.

## Execution note — 2026-09-03, compiler/memory/web permission ports (4.1 partial)

Supersedes the preceding remaining-port list: four compiler tools now enforce
host-conversation/verified-owner session scope and captured local signer
capability; six legacy MCP-native memory tools now use contextual owner checks
and child memory policy. Web fetch retains the configured public-web capability
with parsed-host allowlisting, checked-address connection binding, no automatic
proxies and an exact streamed response byte cap. No sandbox rights are inferred
for these direct native operations.

Tier 0 cargo check --locked --no-default-features --features server-full passed
three times, zero warnings:47.45s,32.56s,15.11s. Targeted git diff --check exit0,
no output. No tests/test authoring/fmt/strict validation/acceptance critic; no
dependency/pin/workflow/commit/delete operations. Full file-by-file evidence is
in .prometheus/session-log.md under the matching checkpoint title.

Uncomfortable limit: task4.1 remains open for file/patch, direct-terminal and
A2UI ports. Proxy-only web deployments may fail now; deleted memory history
cannot be authorized without a live owner record. Timeout is not rollback, and
the blocking DNS lookup still has no joined cancellation contract. Runtime
behavior remains phase-end verification. Counts unchanged; no end-task hook.

## Execution note — 2026-09-03, bounded file I/O (4.1 still partial)

file_tools.rs reads through one metadata-checked handle with an exact byte cap;
file_write checks exact payload/append size and flushes before success.
file_patch.rs computes replacement size before allocation, reads with that cap,
and seeks/writes/truncates its original handle rather than reopening the path.
Tier0 cargo check --locked --no-default-features --features server-full passed
zero warnings12.46s; targeted git diff --check exit0/no output. Tests remain
phase-end work. No dependencies, pins, workflows, commits or deletions changed.

Uncomfortable limit: initial pathname traversal still needs confinement; these
tools remain denied for delegated direct execution. Same-handle operations are
not transactions or exclusion locks against external writers. The proposed
directory-capability implementation uses cap-std4.0.2 (already transitive in
Cargo.lock). Its direct dependency awaits the operator adding cap_std = "4.0.2"
to versions.toml; dependency-pin-discipline forbids bypassing this authority.
Asked for the entry, did not edit versions.toml. Task4.1 remains unchecked.

## Execution note — 2026-09-03, managed terminal process ownership (4.1 partial)

RunManager now owns direct terminal jobs through TerminalSupervisor, with actual
NativeExecutionContext/Orchestrator/TerminalExec callers. Jobs retain exact Child
and worker handles across dropped callers, cancel at run deadlines, join before
completion and preserve failed reaping receipts. Both pipes are concurrently
drained with bounded head/tail capture. Normal/cancel/graph/unwind/server cleanup
paths drain the host scope; verified contextual calls without a scope fail.

Tier0 cargo check --locked --no-default-features --features server-full passed
twice, zero warnings39.43s,19.92s. Tracked diff check exit0/no output; new module
no-index check exit1/no whitespace diagnostics. No tests/test authoring/fmt/
strict verification/acceptance critic/dependency/pin/workflow/commit/deletion.

Uncomfortable limit: direct delegated shell remains denied because reaping a
shell does not confine its permissions or own detached descendants. Standalone
raw callers keep legacy output capture with best-effort kill_on_drop, not joined
host ownership. File confinement awaits the cap_std manual pin; A2UI and remaining
adapter work are still open. No checkbox/end-task hook; counts unchanged.

## Execution note — 2026-09-03, ordinary graph root integration (4.2)

Local AgentNode uses real persisted children and retained first-turn receipts;
the synthetic two-message model call and route-name assistant prefix are gone.
Ordinary verified graph requests now obtain a persisted root through the owned
GraphRootSupervisor and shared ActorThreadSession. Full request fields survive
the handoff; policy discovery includes controls before resolving restrictions.
Worker/join/session ownership outlives a dropped HTTP preparation waiter, while
the internal completion observer does not defeat last-SSE-viewer cancellation.

Graph terminal state now matches its events, and cleanup failure propagates
through server shutdown. Source critic found these two defects; both were fixed,
compiled and re-reviewed without remaining findings in the reviewed paths.
Tier0 cargo check --locked --no-default-features --features server-full passed
zero warnings1m57s,21.23s,17.71s. Targeted tracked git diff --check exit0/no output.
No tests/test authoring/fmt/strict acceptance or dependency/pin/workflow changes.

Uncomfortable limit: this is local graph-adapter implementation, not complete
multi-agent acceptance. Remote AgentNode/A2A remains4.3/5.2; captured-MCP graph
requests still fail closed pending that integration. Direct child file/shell
permissions remain4.1. GraphContext test constructors and runtime verification
remain phase-end work. No broader completion claim is made.

## Execution note — 2026-09-03, A2A task4.3 in progress

HTTP and gRPC now share A2AThreadService, which binds owner/artifact-qualified
tasks to real actor-root turns. Exact invocation receipts, actor capabilities,
host-retained sessions and joined shutdown replace the canned compiler task
path. Named-artifact HTTP routing and gRPC metadata selection are wired through
server startup. The compiler default is an actual registered artifact using the
existing compiler tools, not an alternate ungoverned compile invocation.

The outbound client now wraps message/send correctly, validates RPC/task receipt
identity and owns resumable send/cancel futures in A2ATaskExecution. Parent-token
cancellation learns the send receipt before cancelling; uncertain send mutations
are never replayed. Failed execution and confirmed cleanup are separate. Both
client APIs honor the shared cleanup_unconfirmed flag. Inbound cancel publishes
the flag before awaiting stop, serializes attempts and clears it on confirmed
settlement. Independent source review found and cleared the receipt/lifecycle
defects documented in .prometheus/session-log.md, not runtime acceptance.

Latest Tier0: cargo check --locked --no-default-features --features server-full,
session96545 exit0/zero warnings45.30s. Earlier source groups also compiled; a
redundant import warning was fixed before final passes. All compiler handles
terminal. No tests/test authoring/fmt/strict validation; tests remain phase-end.
No dependencies/pins/workflows/commits/deletions changed.

Uncomfortable limit: task_execution() still has no graph/thread-host caller;
remote AgentNode remains the one-shot adapter. Persisted remote-child identity,
policy/budget binding and cancellation integration are not finished. The plan
requires inherited enforcement but excludes external-agent identity and has no
remote enforcement handshake. Asked the operator to choose trusted UAR peers
with explicit enforcement versus arbitrary A2A with local tracking/cancellation;
no answer yet. Inbound compiled-descriptor artifact projection and stale test
fixture migration also remain. Task4.3 began once at waypoint1870, semantic ID4.3
index19/25; do not repeat begin. No end-task hook. Counts remain2/10 changes,
91/182 actual tasks, thread6/25, Round4/5. Goal remains active, not complete.

## Execution note — 2026-09-03, compiler artifact receipts (4.3 partial)

Supersedes the preceding missing-descriptor-projection limit. The native host
captures typed compiler output before model-history truncation. Exact owner/run
collectors follow UserRun/ActorTurn, close after execution before reply, and
project compiled-descriptor.json through existing HTTP/gRPC artifact fields.
Artifacts survive later model failure/cancellation without changing terminal
status. Ordinary native tools declare no artifact; assistant text is not parsed
as one. The production call path is wired, not just a standalone helper.

Source critic found two other cleanup error classes were being mistaken for
settlement. ActorRootBinding now retains exact sandbox and terminal scopes;
shutdown attempts all families and retains failed receipts. A2A recognizes those
classes and gRPC keeps unresolved cleanup nonterminal despite lacking a metadata
field. Final source review found no remaining concrete defect in reviewed paths.

Tier0 cargo check --locked --no-default-features --features server-full passed
three times, zero warnings:1m01s,11.88s,37.20s. All handles terminal. Source diff
checks had no whitespace diagnostics. No tests/test authoring/fmt/strict
validation; no dependency/pin/workflow/commit/delete changes. Tests stay at phase
end. Artifact correlation remains in-memory, and runtime behavior is unverified.

Remote graph/thread client ownership and inherited policy/budget enforcement
remain incomplete pending the unanswered remote-peer contract question. Do not
mark4.3/5.2 complete or repeat begin-task. Waypoint1870; counts unchanged.

## Execution note — 2026-09-03, cache-inclusive budget receipts (5.1 partial)

Task5.1 already in_progress; no duplicate task-start or premature task-end.
Independent source review found no local actor/graph root-ledger admission bypass.
Fixed cache input undercounting in native Anthropic normalization and included
catalog cache-write pricing in BudgetedModelDriver cumulative charge corrections.
Normalized prompt tokens include both cache portions; root token totals therefore
count them once. Existing public catalog helper signatures retain their behavior.

Tier0 cargo check --locked --no-default-features --features server-full passed
19.55s/18.28s, zero warnings; targeted git diff --check exit0/no output. No tests,
test authoring, formatting or strict validation. Phase-end regression coverage
must include inclusive cache counts and repeated/corrected cumulative usage.
Catalog prices remain estimates without duration-specific cache billing; unreported
provider usage and already-in-flight requests retain their documented limitations.

Remote A2A usage/admission is still not connected to the root budget. Task5.1
stays unchecked pending that production integration and the operator's remote
trust contract. Waypoint1870, actual91/182, thread6/25, implementation2/10.

Independent four-file artifact review accepted source correctness with no remaining
concrete defect. It ran no tests/builds and does not close the remote-budget gap.

## Execution note — 2026-09-03, graph A2UI projection parity (task 4.1 partial)

Ordinary and graph model turns now use one host-owned A2UI tool-output projector.
Successful `a2ui_render` results are validated, published to the exact run replay
backbone, emitted as ordered StatePatch and ArtifactDisplay events, then followed
by ToolEnd. Malformed envelopes/messages emit `a2ui_protocol_error`. Blank surface
IDs are rejected and opaque IDs are JSON-Pointer encoded before state paths are
built. GraphToolHost retains the replay capability with its in-flight model turn,
so cancellation/shutdown drains the same projection operation rather than replaying
or dropping it.

Independent source review found the initially silent malformed-envelope path and
unsafe raw JSON-Pointer interpolation; both were fixed and the corrected artifact
was accepted with no remaining finding in scope. Tier0
`cargo check --locked --no-default-features --features server-full` passed twice,
28.52s and 17.16s, exit0 with zero warnings. Targeted tracked `git diff --check`
exited0 with no diagnostics. No tests, test authoring, formatting, strict acceptance,
dependency/pin/workflow/commit/delete operations were performed.

This does not complete task4.1. Root MCP catalog/cache/shutdown work and delegated
file confinement remain; phase-end runtime coverage has not run. Counts and
waypoint remain unchanged, and no task-end/status hook is due yet.

## Execution note — 2026-09-04, actor adapter and delegated file confinement (4.1 complete)

The previously delivered AgentActor mailbox now runs over the exact attached
thread kernel, actor routes derive ownership from verified user context, and
local actor collaboration uses the source actor's prepared root service. The
remaining direct file-tool gap is closed: `cap-std` 4.0.2 is pinned as a direct
dependency, trusted host configuration is converted to immutable directory
handles during native-tool registration, and read/write/patch share those
preopened handles.

Delegated calls reject empty and wildcard path authority, filesystem-root
capabilities, relative targets, absent preopened roots, and any target outside a
captured root. Capture opens the directory before observing its canonical path,
matches the path identity back to that handle, independently rejects a handle
identical to the filesystem root, and thereafter performs directory creation and
open relative to the capability. `..`, symlink escape and pathname replacement
cannot redirect the retained file handle. Ordinary nondelegated calls retain
their existing allowlist branch and the earlier exact bounded same-handle I/O.
Direct child terminal execution remains denied: owned process lifetime is not a
filesystem, environment, credential, or descendant-process isolation boundary.

Lock refresh was offline because `cap-std` was already resolved transitively.
Final Tier 0 `cargo check --locked --no-default-features --features server-full`
passed with exit 0, zero warnings, in 12.12s; the preceding final-source pass was
13.16s. Targeted `git diff --check` produced no diagnostics. An isolated
artifact-only critic found and drove fixes for the root-alias and capture-order
races, then reported no remaining P0-P2 finding. No tests, test authoring,
formatter, full build, strict validation, workflow, commit, deletion, or external
service action ran; tests remain tasks 6.1-6.4 at the phase boundary.

No feature beyond task 4.1 was added. Every new guard traces to the delegated
filesystem trust boundary or the critic-observed alias/swap failures. On targets
without supported stable directory-identity fields, delegated roots fail closed.
Runtime path, ownership and cancellation behavior remains unverified until the
phase-end test group.

## Phase-end live evidence — 2026-09-04 (task 6.3)

Command: `UAR_SMOKE_MODEL=k3 UAR_SMOKE_LOG=info node openspec/changes/thread-native-subagents/evidence/live-cancellation.mjs target/debug/uar-sidecar --before-first-response`.
Observed exit: 0. The output reports `result: passed` and
`cancellation_point: awaiting-first-provider-response`; the complete JSON is
retained at `evidence/live-cancellation-report.json`.

The real router returned HTTP 200 and two text chunks. One persisted child
entered running state and sent its request to the same real provider. Root
cancellation aborted that pending fetch before teardown, closed all attempts,
started no additional attempt, and produced the same child's cancelled
lifecycle. The sidecar log reported `UAR_SHUTDOWN outcome=graceful_complete`.

The artifact-only reviewer checked this scenario against the written 6.3
criterion and accepted its scope. Cancellation after emitted child text remains
unverified: real provider timeouts and 500/502 responses prevented that separate
scenario from reaching its cancellation point. Provider-side computation and
billing termination, live remote-peer cancellation, and typed shadow parity
are not established by this receipt. `shadows` was empty; it cannot satisfy the
typed-default evidence gate.
