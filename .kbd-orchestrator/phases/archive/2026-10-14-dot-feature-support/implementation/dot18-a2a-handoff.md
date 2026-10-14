# Dot18 A2A implementation handoff

Phase: dot-feature-support / Execute. Source implementation only. No test, check,
build, formatter, validator, probe or independent verification gate has run.

## Plan delta and remaining boundary

The planned 1.0 boundary is implemented as a separate adapter; legacy JSON-RPC
and gRPC keep their wire types. Durable restart correlation, host checkpoint
continuation and live remote cancellation are authored, not verified. Dot18 must
remain unverified; Dot11 exclusion/recovery and Dot19 committed interactive
state/approval continuations are now integrated at source. The uncomfortable thing is that a
valid card and recoverable completed task still do not prove safe restart of an
active producer. The new context and artifact persistence paths also remain
uncompiled and unexecuted until the full-phase boundary.

## Pinned normative contract

- https://github.com/a2aproject/A2A/blob/v1.0.0/specification/a2a.proto
- https://a2a-protocol.org/v1.0.0/specification/ sections 3.6, 5.3–5.7, 9.
- Context7 discovery: /websites/a2a-protocol, then official versioned sources.

JSONRPC `SendMessage`, `GetTask`, `CancelTask`, `ListTasks`; request
`A2A-Version: 1.0`; `SendMessage` response is `{task: ...}`. JSON fields are
camelCase; role/state strings are the `ROLE_*` / `TASK_STATE_*` ProtoJSON enum
names. Content is a oneof `text`, `raw`, `url`, `data`, without legacy type/kind.
This text-only compiler rejects other input content with -32005. Task history
limits, blocking default, returnImmediately, tenant assertion consistency,
unsupported streaming/push/extended-card, and standard 1.0 error numbers are
explicit. Auth failures use HTTP401/403 and custom -32010/-32011; inaccessible
private tasks yield -32001 without identifying another owner.

Card: `/.well-known/agent-card.json`, exact GET anonymous exemption. Its
`supportedInterfaces` advertises only JSONRPC1.0 at `/a2a/v1/compiler`.
`securitySchemes` is a map of 1.0 security unions and `securityRequirements`
uses `schemes`/`list`. Version derives from `env!("CARGO_PKG_VERSION")`, currently
the manifest source; installed artifact/tag equality remains unverified.
`/.well-known/agent.json` remains an anonymous legacy card alias. Existing
`/a2a/compiler`, `/a2a/agents/{agent_id}` and gRPC are compatibility surfaces and
are not advertised as 1.0.

## Files authored by A2A teammate

- `src/uar/api/a2a/v1.rs`: independent wire/card/dispatch projection.
- `src/uar/api/a2a/durable.rs`: full-principal transport binding, validation/CAS
  identity contract, exact reserved run, non-expiring process identity, digest
  and cancellation intent. This is correlation, never execution authority.
- `src/uar/api/a2a/thread_service.rs`: existing actor dispatch reserves binding
  before mailbox publication; retries correlate by message ID; completion
  observer retains final task/artifact projection after caller disconnect.
  Missing local handles read exact durable roots after full-principal binding
  lookup. Durable cancellation intent is consumed by the owning actor observer,
  and success waits for its confirmed cancelled root/cleanup.
- `src/uar/api/a2a/mod.rs`: exports + new well-known route.
- `src/uar/api/a2a/agent_card.rs`: legacy version derives from same package.
- `src/uar/api/a2a/types.rs`: corrected misleading legacy version comments.
- `src/uar/runtime/actor/system.rs`: narrow host persistence accessor and
  crate-visible reserved captured dispatch; stable full-principal context
  session restoration; no kernel mutation.
- `src/uar/runtime/thread/actor_host.rs`: host history restoration and session/
  artifact receipt persistence before terminal root commit.
- `src/uar/runtime/thread/graph_host.rs`: optional artifact collector forwarded
  through the existing owned graph worker; ordinary start remains unchanged.
- `src/uar/runtime/manager.rs`: narrow dedicated A2A continuation helper mints
  the exact owner/run collector and invokes that existing host; seals request
  policy/budget/remote provenance into the continuation permit. Root-requested
  `runtime_epoch()` getter and Required-before-governance-Off condition authored.
- `src/uar/runtime/turn/request.rs`: host-only fresh A2A contract provenance.
- `src/uar/persistence/checkpoint_resume.rs`: explicit A2aToolLoop execution kind;
  generic HTTP restoration remains ToolLoop-only, while A2A requires protected
  equality to a freshly authenticated contract. Storage claimability is separate.
- `src/uar/persistence/accounting.rs`, `runtime/cost_budget.rs` and new
  `persistence/providers/accounting_remote_grants/{surreal,postgres}.rs`:
  cumulative destination grant model/tool attempt counts and immutable absolute
  first-admission deadline; PG transaction/Surreal CAS precedes execution.
  Existing accounting SQL/Surreal migrations include the grant table.
  Provider and trait registration was added in a fencing-granted writer window.
- `src/uar/security/middleware.rs`: exact GET card exemptions only.
- `src/uar/api/routes.rs`: extracted `resume_checkpoint_admitted` typed receipt
  using the existing complete protected owner/resource/catalog/service checks.
  Original HTTP wrapper delegates with no reserved ID and retains response.
- `src/uar/persistence/providers/a2a_bindings/{surreal,postgres}.rs`: native
  create/load/CAS/list adapters; Surreal requires durable provider capability.
- `migrations/20261012020000_a2a_bindings.sql` and
  `migrations/surrealdb/a2a_bindings.surql`: owner/agent/task partitioned records.
- `tests/fixtures/dot18_a2a_wire.json`: pinned literal conformance fixture.
- `tests/fixtures/dot18_a2a_real_path.py`: deferred installed-runtime fixture,
  submit/read-after-restart/cancel modes; uses real authenticated endpoints,
  checks anonymous discovery, version negotiation and cross-owner denial.
- `docs/A2A_V1.md`: public contract, compatibility and recovery limits.

## Integration contracts and other writers

Root registers four A2A persistence trait methods and provider delegations, durable
capability, migrations, and the two routes `v1::handle_rpc` and
`v1::handle_agent_rpc`. Root supplies
`A2AThreadService::with_run_api(Arc<routes::RunApiState>)`.

Accounting teammate implements
`RunManager::execute_checkpoint_a2a_with_run_id(request, envelope, bound,
approval, run_id)` by sharing existing protected claim checks, consuming the
pre-reserved ID and using existing `graph_roots.start` / `ActorThreadSession`
for native root persistence. A2A never bypasses that host to execute a task.
The fresh-contract parameter is now the final argument of
`routes::resume_checkpoint_admitted`; ordinary HTTP passes None. The fencing
worker added protected `a2a_delegation` context capture and changed provider claim
admission to `require_claimable_execution`. Only the current host renews authority.
Root must use `kernel.remote_budget().admit_tool_durable().await` at existing
thread/service.rs host-tool admission sites; the manager tool gate already does.
No kernel mutation was introduced.

Dot19 may set trusted task metadata `uar_auth_required: true` from a committed
host interaction, projected to TASK_STATE_AUTH_REQUIRED. Waiting durable roots
project INPUT_REQUIRED. Arbitrary client message metadata does not set task
metadata. Human approval still requires the existing approval-specific host
path; free-text follow-ups never silently approve a tool.

## Outstanding production integration / uncertainty

1. Dot11 integration now captures protected runtime_epoch in each new reservation
   and continuation. Native Surreal reconcile_a2a_cancellation reads the exact
   binding and closed host epoch, reuses require_resolved_effects for tool/canonical
   outcomes and descendant accounting, and rejects unsettled remote children.
   One transaction compares binding/epoch/native thread snapshots, refuses an
   existing continuation claim, cancels nonterminal native threads, marks source
   checkpoint envelopes cancelled, and commits the canceled task projection.
   Checkpoint claim now refuses that cancellation marker. Terminal native results
   remain immutable; an excluded yielded turn may close its transport task without
   rewriting its kernel result. No effects or money are settled by exclusion.
   Legacy no-epoch records and unsupported PG supervisor recovery stay held.
   Nonlocal CancelTask returns conflict while unconfirmed intent remains durable;
   later reads can reconcile. Process absence is never successful cancellation.
2. Dot19 approval-specific continuation is now connected: task metadata exposes
   exact broker records; SendMessage metadata uar.approval resolves warm CAS or
   passes the protected record into cold host admission. Generic text cannot
   approve. See dot19-interactive-handoff.md; this remains unverified.
3. Fresh governed resume now requires incoming contract equality, current peer
   and target identity, exact owner/task/checkpoint, renewed local policy and
   durable accounting. Model/tool counts and absolute expiry survive restart.
   A canceled/ambiguous admission remains reserved rather than replaying.
4. Host artifact receipts now commit before terminal root persistence, including
   the dedicated checkpoint continuation collector. A failed receipt commit
   retains an unconfirmed context and cannot release the context slot.
5. Atomic provider context slots and shared host session restoration are authored.
   Cold reads reconcile committed terminal roots into transport projections;
   supplied contexts cannot overlap tasks or silently discard persisted history.
   Items 4 and 5 have source implementation, not qualification evidence.
6. Both native provider implementations and all code remain uncompiled and
   unexecuted under the full-phase boundary. The real-path fixture is authored,
   not acceptance evidence; fresh-process exclusion, races and installed version
   equality require the final phase gate and customer artifacts.
7. Dot19 committed approval projection/decision integration is now authored and
   handed to root, including all-open interrupt correlation and protected cold
   continuation. The manager descriptor-Required precedence fix is authored;
   root owns the governance warning/spec projection and other Off risk rules remain.

## Guard rationale and scope

Full-principal owner checks protect the actual external task boundary. Oneof,
role/version/media validation implements the explicit protocol requirement.
Durability capability prevents an ephemeral store advertising restart support.
CAS/non-replay protect competing submissions and uncertain writes named in the
plan. Cleanup uncertainty never proves cancellation. No unrelated dependency,
workflow, pin, UI, kernel or package change was added by this teammate.

## Final source integration additions

`api/a2a/durable.rs` adds optional protected runtime_epoch (legacy absence does
not imply proof), disallows changing the epoch for one native run, and forbids
new continuations after cancellation intent. `thread_service.rs` captures epochs
before dispatch, calls native reconciliation on canceled cold reads, preserves
the confirmed cancellation projection, and blocks approval/resume after intent.
`persistence/mod.rs` exposes a default-unsupported host reconciliation seam;
`providers/surreal.rs` delegates; `providers/a2a_bindings/surreal.rs` owns its
atomic implementation. `providers/checkpoint_records/surreal.rs` exposes the
existing resolved-effects preflight to the sibling host adapter and refuses
canceled envelope claims. No new ledger, executor, timer lease or supervisor
proof constructor was added. SQL/data remain unexecuted.

Latest source handoff: root accepted Dot19/1 source completion for canonical
recording only. MCP interaction lifetime finalization and AG-UI settled terminal
projection are integrated at source (see Dot19 handoff). No pending production
seam in the assigned A2A implementation is being represented as verified. Build,
restart, race, wire conformance and installed-version claims remain unqualified.
