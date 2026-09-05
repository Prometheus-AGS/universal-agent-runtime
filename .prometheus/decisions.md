# decisions

Append-only. Dated entries. Mark superseded entries; do not delete them.

## 2026-08-09
- Initialized by prometheus-context-bootstrap.

## 2026-08-09 — Migrated agent context from Base Rules v3 to the bootstrapped structure

**Decision.** Replaced the ~4,720-word `AGENTS.md` (and its duplicate `CLAUDE.md`)
with a managed region of ~1,396 words, plus path-scoped rules, deterministic hooks,
and this learning directory. Applied with
`prometheus-context-bootstrap/scripts/migrate.sh --apply`, profile `mixed`.

**Mechanism.** Resident prose now carries invariants only. Tier discipline,
single-writer, the sycophancy gate, and the compaction re-anchor moved from prose
into `.claude/hooks/`, where they are enforced rather than advised. Per-stack
commands moved to `.claude/rules/`, loaded on file read.

**Stakes.** Two constitutions in resident context degrade adherence to both; that
was the state before, since `AGENTS.md` and `CLAUDE.md` each carried all 45 v3 rule
IDs. `CLAUDE.md` is now a symlink, so nothing double-loads.

Profile is `mixed`, not `lean`, because this repo is worked by more than one model
family. See `.prometheus/model-fleet.md`. Do not switch to `lean` on the strength of
a benchmark — measure against a fixed task set first.

Archives: `.prometheus/knowledge/AGENTS.pre-migration-2026-08-09.md` and
`.prometheus/knowledge/AGENTS.pre-migration-2026-08-09.CLAUDE.md`.
Coverage report: `.prometheus/MIGRATION-REPORT.md`.

---

## 2026-08-09 — RESOLVED: the 2026-07 production-completion lock is retired; the gap it guarded is now tracked debt

**Resolution.** The `uar-final-production-hardening-2026-07` execution lock is
retired. It is **not** restored to resident context.

The lock instructs every session to ask "does this advance changes 20-24; if not,
do not do it." The active waypoint is `uar-uiux-full-migration-2026-08`, status
`running`, 47/72. A resident rule that refuses any action outside a different
phase directly contradicts the phase actually being worked, and would outrank the
waypoint it explicitly defers to. Retiring the lock removes that contradiction.
It does not resolve the substantive gap, which is recorded below.

Its phase-independent rules — never `cargo clean`, batch fixes, CI as asynchronous
evidence, zero warnings, Linux/macOS Stable vs Windows Experimental — are already
carried in `.claude/rules/rust.md` and are deliberately not duplicated into
`AGENTS.md`. Duplicating resident and path-scoped context is the overlap this
migration exists to remove.

**The gap, as tracked debt.** v1.0.0 shipped ahead of at least one of its own
certification gates.

| Evidence | Value |
|---|---|
| `v1.0.0` tagged and published | 2026-07-11, GitHub release "First stable release" |
| Tag is ancestor of `main` | yes; `main` is +367 commits |
| KBD ledger | 20/24 DONE, 4 PENDING, `implementation.status: IN_PROGRESS` |
| Ledger last updated | 2026-08-08 — current, not abandoned |
| `evidence` / `certification` / `publication` | all PENDING, `summary: null` |
| SBOM / supply-chain artifacts at repo root | **none present** |
| `fix-sidecar-loopback-auth` | 5/6 tasks; unchecked 2.2 is `certify-operational-resilience` work under another name |

The four PENDING changes are `certify-operational-resilience`,
`produce-supply-chain-artifacts`, `certify-release-candidate`, `release-1-0-0`.

**The unresolved question, stated precisely.** Is `release-1-0-0` PENDING because
the ledger was never closed after the release shipped, or because the tag was cut
ahead of its gates? These have opposite implications.

For `produce-supply-chain-artifacts` the question is settled: **no SBOM artifacts
exist on disk.** Had the work been done and merely left unrecorded, the artifacts
would be there. Absence of the artifact is positive evidence of absent work, not a
lagging ledger. For the other three, nothing in the repo distinguishes the two
readings.

**Stakes.** This estate's commercial position is evidence discipline — SLSA L2,
signed receipts, provable governance. A 1.0.0 release without supply-chain
artifacts is a gap in exactly the thing being sold.

**What reopens this.** When release or certification work resumes: close the four
ledger items or correct the ledger to match reality, and **do not cut a 1.0.1
until `produce-supply-chain-artifacts` has actually produced artifacts.**

The KBD ledger was not edited here. Correcting `progress.json` is the
orchestrator's record and a separate, deliberate act.

Lock source text preserved at
`.prometheus/knowledge/AGENTS.pre-migration-2026-08-09.md`, lines 4-18.

---

## 2026-08-09 — Retuned the permission surface; moved coupled-dependency checking into a hook

**Decision.** Widened `permissions.allow` from 4 entries to 26, narrowed the
`.kbd-orchestrator` deny from one directory-wide rule to four file-pattern rules,
and taught `tier-guard.sh` to detect coupled-dependency skew.

**Mechanism.** The allow list held only four `git` rules, so every `cargo check`,
`pnpm typecheck`, `jq`, and `openspec` call fell through to an interactive
prompt. Those are the Tier 0-2 commands the rules already mandate; asking about
them spends operator attention without producing a decision. They are now
allowed. `git push`, release builds, `tauri build` and `flutter build` stay in
`ask`, and Tier 3 keeps its independent block in `tier-guard.sh`.

`Edit(.kbd-orchestrator/**)` protected canonical state but also blocked phase
authoring — writing `goals.md` for a new phase was refused. The repo already
draws the line mechanically: `.md` files are authored prose, `.json`/`.jsonl`
are orchestrator-written ledger state. The deny now names the ledger explicitly
(`**/*.json`, `**/*.jsonl`, `current-waypoint.*`, `position*`), which protects it
*better* than a directory rule that had to be worked around.

**Stakes.** Two defects on 2026-08-08/09 shared one shape: half of a coupled pair
merged, the other half not. `wasmtime` 47 beside `wasmtime-wasi` 46 put two
distinct `Linker<T>` types in one build and stopped `server-full` compiling.
`@assistant-ui/react` 0.15.4 beside `react-markdown` 0.14.8 broke a peer
requirement and removed `useMessage`, blocking every frontend commit. Neither was
caught by review: "MERGEABLE" and "minor-patch" both read as safe.

No permission rule would have stopped either — both were allowed operations with
unverified consequences. So the guard belongs in a hook, where it is
deterministic, rather than in prose that asks someone to remember. Widening
`allow` therefore does not increase the risk that actually bit us.

**Proven, not asserted.** The coupled-dependency check was verified in both
directions: silent with pins aligned, and warning correctly after a
`wasmtime-wasi` 47->46 skew was introduced temporarily and reverted (`Cargo.toml`
restored byte-identical). Tier paths retested: Tier 3 blocked (exit 2),
`PROMETHEUS_TIER3=1` opt-in allowed (exit 0), Tier 0 allowed (exit 0).

The dependency check is **advisory** — stderr plus exit 0. A false positive that
blocks a legitimate bump costs more than an occasional unnecessary reminder.

**Known false positive, accepted.** `tier-guard.sh` matches its Tier 3 regex
against the whole command string, so *writing prose that mentions a release
build* is indistinguishable from running one. This entry could not be appended
with a heredoc for that reason and was written with the file editor instead.
Narrowing the regex to anchor on command position would risk missing real
invocations inside compound commands; the current trade favours no false
negatives at the cost of this one benign false positive.

---

## 2026-08-09 — Routine verification is local; GitHub Actions are deployment-only

**Decision.** GitHub Actions are reserved for deployment execution and
deployment validation. Unit, integration, conformance, lint, format, and other
routine development checks run locally before commit and push.

**Rationale.** Remote development-test runs add queue and execution time without
improving the evidence available from the same pinned local command. The
conformance phase keeps its exact recorded/server-full/serial matrix, but its
gate is a local non-zero/zero command result rather than a workflow result.

**Consequence.** The `conformance-baseline-gate` contract, proposal, spec delta,
and tasks were amended from CI red/green proof to local red/green proof. The
non-deployment `spec-conformance.yml` workflow was removed and its active run
cancelled. Verification records cite local command evidence rather than Actions
run URLs.

---

## 2026-08-09 — CI/CD validation is DEFERRED, not declined: gates land after the code, not before

**Decision (operator).** The conformance matrix runs as a **local** gate for now.
No new blocking GitHub Actions job is added while the tests it would run are
still being written. **CI/CD-based validation WILL be supported** — it is
sequenced after a working code base, not abandoned.

This supersedes `conformance-baseline-gate` task 2.1 as originally written
("add a dedicated job with `continue-on-error: false`"). Anyone reading that
task cold will think the executor deviated. It did not; the scope changed here,
deliberately, and Codex's commit `3bf72e15` *"docs(policy): reserve actions for
deployment"* records the same call from the execution side.

**Mechanism — why a gate added now would carry no signal.** Measured on
`origin/main` at `a70996f`, 2026-08-09:

| Workflow | Conclusion |
|---|---|
| Live Integration Tier | success |
| CI | **failure** |
| Build and Deploy to AKS | **failure** |
| Coverage | **failure** |
| Cookbook examples | **failure** |
| BDD Chat Scenario Suite | **failure** |

Five of six red, all predating this phase. A sixth red check is
indistinguishable from the five already there. Worse, it is *actively harmful*:
a pipeline that is always red teaches everyone to stop reading it, which is how
`live-integration.yml` masked a build failure for 25 days behind
`continue-on-error: true` while showing green.

**Second reason — cost against changing code.** The matrix takes ~195s plus
build. Running it on every push while its own cases are still being authored
pays repeatedly for a signal about code that is about to change. That is the
same economics the tier ladder exists to prevent: expensive verification belongs
at phase boundaries, not inside the edit loop.

**Stakes, stated plainly.** Until the gate is wired, the matrix runs when
someone remembers to run it. **That is not enforcement.** Regressions will
reappear silently, which is exactly the failure this phase was opened to close.
Deferring is the right call on today's evidence; forgetting to come back would
undo the phase's whole purpose.

**What is already banked.** The expensive half of the gate work is done and
transfers. Codex ran the deliberate-break probe locally — red naming the
specific case, then green after revert (`13edc142` -> `f873a940`). Proving a
gate can fail is the part people skip; wiring it into a workflow afterwards is
mostly a YAML file.

**What reopens this — the prerequisite is explicit.** Before the matrix is
wired into Actions:

1. Get `main` green. Five failing workflows must be fixed or deliberately
   retired. Adding a gate to a red pipeline gets it ignored.
2. Then add the matrix job with `continue-on-error` absent, and re-prove the
   red/green cycle **in CI** — the local proof does not transfer to the runner.
3. Fixing those five workflows is its own change with its own scope. It does not
   belong inside a conformance-measurement phase, and folding it in would
   invalidate the adversarial review that phase's plan already passed.

`conformance-baseline-gate` tasks 3.3, 4.1 and 4.2 keep their local form: record
the red run and the green run in `verification.md` as local results. When CI is
wired, those rows gain run URLs.

---

## 2026-08-09 — Supersession: deployment validation is the only GitHub Actions test scope

The deployment-only operator decision above is final and supersedes the earlier
idea that routine conformance checks might later move into Actions. Unit,
integration, conformance, lint, and format checks remain local. GitHub Actions
may validate deployments at deployment time; it does not run development tests.

---

## 2026-08-11 — PAGS-SPEC-PID-001 changes the scope of uar-1-0-readiness

**Decision.** `uar-1-0-readiness` scopes to **GAP-02, GAP-03, GAP-05, and
widening `TokenVerifier`**. It does **not** make UAR depend on `frf-did` or
`frf-wallet`.

**Mechanism.** A first draft of this phase proposed making UAR consume those two
crates so that the C-25/C-26/C-27 exclusions would collapse into real tests.
PAGS-SPEC-PID-001 (Prometheus ID, Draft v0.1) supersedes both: §0 lists it as
superseding *"ad-hoc `did:key` issuance in `frf-wallet`"*, and §2.2 marks
`pid-wallet` as *"Supersedes `frf-wallet` issuance."* Adding a dependency on a
layer already scheduled for replacement buys a test result and a migration.

Instead, UAR widens `TokenVerifier` now to the PID FR-5.1 shape — one trait, a
`Presented` enum with `Jwks` / `SdJwtVp` / `DidAuth` variants, returning a single
`Principal`. PID §6.1 makes the argument itself: *"Doing this before there are
consumers costs nothing. Doing it after means every downstream site branches on
auth lane."* That converts C-25/26/27 from "UAR lacks a dependency" into
"awaiting PID P4" — a scheduled dependency rather than an open gap.

**What stays UAR's own work.** PID touches neither:

| Gap | Status under PID |
|---|---|
| GAP-02 no JWKS/RS256 verifier | PID §6.1 keeps the RS256 lane **unchanged** — *"San Saba must not need DHT resolution to log a user in."* UAR should close this now |
| GAP-03 A2A store not tenant-partitioned | Untouched by PID |
| GAP-05 builtins not registered on embedded | Untouched by PID |

**Stakes.** PID §8 sequences UAR's GAP-02 as unblocked by its own P4, and P4 sits
behind P0's three blocking decisions — one of which (D-3, entitlement issuance
topology) is a business decision involving an external party. **If UAR waits for
PID to close GAP-02, it waits on someone else's negotiation.** Since PID
explicitly preserves the RS256/JWKS lane, the two are designed to coexist, and
UAR should ship a real JWKS verifier independently.

**What reopens this.** When PID reaches P4 and `pid-verifier` exists, UAR
consumes it as the `SdJwtVp` arm of the already-widened trait. The six
conformance exclusions collapse then, not before.

---

## 2026-08-11 — Cross-harness handoff protocol recorded

**Decision.** The Claude-Code-authors / Codex-executes split used by
`uar-spec-conformance-2026-08` is written down at
`.kbd-orchestrator/HARNESS-HANDOFF.md` and applies to `uar-1-0-readiness`.

**Mechanism.** The boundary is the spec handoff: everything the executor needs
must be on disk and in git, because it does not share the authoring
conversation. `EXECUTION-CONTRACT.md` is the load-bearing artifact — adversarial
review of that phase returned INSUFFICIENT on six findings and **every one was
about autonomous executability rather than correctness** (implicit inheritance
between changes, a dangling cross-change reference, ambiguous requirement scope,
no verification-record format, no precedence rule, an undefined satisfaction
boundary).

**Stakes.** Three failure modes are recorded because each cost real time:
`progress.json` went stale for a day while neither harness owned it; a first
check reported "nothing from Codex is on main" because local `main` was stale
rather than because the work was missing; and the executor made two scope changes
(five extra exclusions, a repo-wide CI prohibition in a measurement-phase spec
delta) that were defensible but unreviewed. **The executor is not obliged to flag
its own scope changes, so diffing the merged spec against the reviewed spec
belongs on the authoring side.**

---

## 2026-08-13 — an installer refuses rather than renames, and proves completeness itself

**Decision.** When a skill cannot be placed at its canonical name, the install
**fails**. The `prometheus-<name>` fallback survives only behind
`--allow-fallback`, and even then the run exits non-zero. A completeness gate
(`scripts/verify-skill-install.js`) asserts every skill at every target and runs
as the install's own last step.

**Mechanism.** Declining to clobber a file the installer does not own is
correct; doing so while reporting success is not. The two are incompatible and
the code chose both — placement diverted silently and verification followed the
divert, so 19 skills were unreachable across 14 targets while every run printed
a checkmark. Making the gate the install's last step collapses "install ran" and
"install is correct" into one statement, so no judgment sits between them.

**Stakes.** The cost is real: an operator with a legitimately foreign skill
directory now gets a failed install instead of a working one with a renamed
skill. That is the intended trade. A silent rename is indistinguishable from
success at the point where it matters, and the operator shipped on that
assumption. A loud failure is recoverable in one command; a silent one persisted
across many install loops and was caught only by a human noticing a missing
skill.

**What reopens this.** If `--allow-fallback` is exercised in practice and the
non-zero exit proves obstructive rather than protective, revisit whether the
flag should exit 0 with a persistent warning. It has never been run.

**Also decided.** Rule promotion for the underlying method failure (completeness
claims require a denominator) goes through §D-6's gates — adversarial review,
sycophancy gate, explicit approval — before any rule text lands. Operator
direction, same date. Record:
`.prometheus/postmortems/2026-08-13-skills-not-installed-at-canonical-names.md`.

---

## 2026-08-22 — Inference certification requires real model inference

**Decision.** Only full integration requests that traverse the packaged UAR
boundary, reach a real loaded model, perform actual inference, and return the
result through UAR count as inference integration, soak, resilience, release,
or production-readiness evidence. Mocked, stubbed, recorded, replayed,
hard-coded, or synthetic provider responses never satisfy those claims.

**Mechanism.** A certifying result identifies the provider and model and retains
evidence that genuine model output was observed through UAR. If credentials,
capacity, model weights, network access, budget, or another real-inference
prerequisite is unavailable, the executor stops and reports the claim as
unverified instead of substituting synthetic success. Fast model-double tests
remain permissible only as explicitly non-certifying unit or component
diagnostics. Multi-hour synthetic inference tests are prohibited.

**Rationale.** A three-hour operational soak sent thousands of deterministic
requests to a local Python provider double. It exercised runtime plumbing but
left the actual provider/model inference boundary untested, consuming the time
needed for the real verification. Duration does not make an unrepresentative
workload production evidence.

**Stakes.** Real inference costs money, depends on external or locally hosted
model capacity, and produces nondeterministic text. Those are the production
boundary being certified, not reasons to remove it. The current mock-only soak
is non-certifying and cannot support an inference-readiness or release claim.
A multi-hour real-inference run also requires a documented failure model,
traffic-volume target, operating-period target, or statistical detection goal;
elapsed time alone is not evidence.
## 2026-08-14 — UAR standardizes `jsonwebtoken` 11 on RustCrypto

**Decision.** Every UAR-owned `jsonwebtoken` dependency resolves through one
workspace entry pinned exactly to `11.0.0`, with default features disabled and
only `rust_crypto` selected. The earlier AWS-LC spike conclusion is historical
and superseded.

**Rationale.** RustCrypto removes the native C/assembly provider from UAR's JWT
choice, works across the separately checked server-full, iOS, and Android
graphs, and is already present in the lockfile. The decision is not based on a
performance claim. A1 requires authenticated public-key verification only; it
does not add RSA/PS private-key signing.

**Uncomfortable constraint.** `jsonwebtoken` 11 stores its process provider
behind a crate-private getter. Its public `install_default()` error returns the
provider the caller attempted to install, not the provider already present.
Consequently UAR cannot distinguish “RustCrypto was installed before UAR” from
“a foreign provider was installed before UAR” by pointer identity. A0 remains
in progress until the operator either requires UAR to own first installation or
expands scope to a patched/forked provider API. No completion claim transfers
from the backend decision to that unresolved initialization contract.

---

## 2026-08-14 — UAR owns first `jsonwebtoken` provider installation

**Decision.** The operator selected the first-owner option. UAR installs
RustCrypto at the shared server-startup funnel and before every UAR-owned JWT
encode/decode operation. Repeated calls reuse only UAR's recorded successful
installation. Any provider initialized before UAR—including RustCrypto—fails
closed with a structured provider-conflict error.

**Rationale.** `jsonwebtoken` 11 exposes neither the installed provider nor an
identity token for it. Treating a failed RustCrypto installation as proof that
the existing provider is RustCrypto would accept AWS-LC or an arbitrary
downstream provider under feature unification. Owning first installation makes
the invariant observable without a fork or a new dependency.

**Supersession.** This resolves the uncomfortable constraint in the preceding
RustCrypto decision. It does not reverse the backend choice; RustCrypto remains
the sole UAR-owned `jsonwebtoken` feature. If another component must own the
process provider, that integration must change architecture explicitly rather
than bypass the guard.

---

## 2026-08-15 — Skill enablement is durable and most-specific-wins

**Decision.** Store skill enablement as durable global, agent, and conversation
records on each skill. Resolve conversation first, then explicit agent state,
then the pre-existing non-empty agent-binding allowlist as a compatibility
fallback, then global state. Keep `Skill::enabled` as the legacy global copy and
synchronize it on new global writes.

**Rationale.** Existing persisted rows and clients already read `enabled`, while
the scoped records must survive built-in re-registration. The compatibility
fallback preserves bindings created before a skill is loaded; explicit scoped
records remain authoritative when present.

**Runtime consequence.** The run policy universe contains every registered
skill. Scoped matching filters that universe using the existing agent and
conversation identifiers, and the returned skill clones remain the run's
start-time binding.

**Uncomfortable constraint.** A conversation enable cannot widen a global
disable if the policy universe discards the skill before scoped resolution.
Changing precedence without changing universe construction produces tests that
pass in the service and behavior that fails in a real run.

---

## 2026-08-15 — `skills/dynamic` is an API-owned persistence namespace

**Decision.** Files beneath the filesystem provider's reserved `dynamic/`
directory are API-managed and reload with `provider_id = "api"`. The filesystem
write boundary rejects every other provider id. Configuration files outside
that directory reload as `fs-skills`, and when an upgrade leaves both sources
for one ID, the real configuration source wins deterministically.

**Rationale.** Reconciliation may tombstone only exact `fs-skills` records.
Before this boundary was explicit, an API skill could reload as configuration,
and a stale dynamic copy of a config skill could win by directory traversal
order. Either failure makes provenance unsuitable as the data-loss guard.

**Uncomfortable constraint.** The reserved path is now part of the provenance
contract. Moving API-created files elsewhere, accepting config records at the
dynamic write boundary, or restoring last-writer-wins cache insertion would
invalidate reconciliation safety and requires a new migration decision.

---

## 2026-08-19 — Persist provider defaults before publishing live state

**Decision.** When a `SettingsManager` is configured, changing the default
provider validates the target, persists the new provider ID, and only then
publishes it to the live registry. A persistence failure leaves the live
default unchanged. The existing registry-only behavior remains available when
the server has no settings persistence configured.

**Rationale.** Publishing first allowed the API to report a live default that
could not survive restart. Persistence-first ordering makes the durable setting
the commit point while retaining compatibility for deployments that explicitly
run without a settings manager.

**Uncomfortable constraint.** Provider deletion and default selection still
span separate stores without a transaction. This change prevents publication
after a failed settings write, but it does not make concurrent deletion and
selection atomic.

---

## 2026-08-21 — Release certification is local; Actions remain deployment-only

**Decision.** Run operational resilience, installed-artifact, supply-chain,
security, load, stress, soak, and release-candidate certification locally from
an immutable checkout. GitHub Actions may execute deployments and validate the
resulting deployment only. Remove every other workflow and enforce the retained
three-file deployment allowlist with a local pre-commit validator.

**Rationale.** A prior plan treated a release gate, container build, and hosted
artifact upload as sufficient to call product testing deployment validation.
That contradicted the 2026-08-09 operator decision and caused a three-hour soak
to be dispatched to Actions. Run `32458212074` was canceled and cannot be used
as evidence. KBD plan revision 8 and decision
`deployment-only-actions-local-release-certification` supersede the conflicting
phase language.

**Uncomfortable constraint.** Keyless supply-chain publication previously
depended on workflow identity and hosted OIDC. The remaining release-tail
changes must replace that mechanism with locally produced and independently
verified evidence before publication; deleting the workflows does not by
itself prove the replacement is complete.

---

## 2026-08-21 — Freeze after all local release-tail tooling lands

**Decision.** Land and locally verify the operational, supply-chain,
candidate-certification, and promotion scripts/contracts before freezing the
immutable candidate. Evidence and status transitions still execute in order;
after the freeze, only evidence/checkpoint commits are allowed.

**Rationale.** Running the three-hour certification before replacing later
workflow-bound release tooling would force a source edit and invalidate the
run. KBD plan revision 9 and decision
`freeze-after-local-release-tail-tooling` preserve one meaningful source-bound
certification instead of knowingly scheduling a throwaway run.

**Uncomfortable constraint.** This permits source preparation for later
changes while the operational change is active. It does not permit their
evidence, tags, signing, publication, or completion transitions to run early.

---

## 2026-08-21 — MCP reconnect state includes configuration generation

**Decision.** Store each configured MCP server's current service, authoritative
reconnect entry, and configuration generation in one private shared slot.
Filtered and merged views share only slots they are already authorized to use.
A reconnect built outside the lock may replace the service only if its captured
generation is still current.

**Rationale.** Sharing only the service pointer fixed dead-handle propagation
but allowed an old view to reconnect configuration A after an A-to-B upsert and
overwrite B. The generation guard closes that rollback without sharing policy
maps, holding a synchronous lock across `.await`, or replaying the failed call.

**Uncomfortable constraint.** This does not serialize concurrent failures or
change snapshot-view behavior after server removal. Those behaviors require a
separate observed problem and plan.

---

## 2026-08-22 — Shutdown timeout is one absolute signal-to-exit window

**Decision.** Under `server-full`, observe SIGTERM/SIGINT once, stop both HTTP
listeners immediately, and measure the configured graceful timeout from that
observation. A standard-library watchdog owns the forced boundary independently
of Tokio. Normal completion waits for ingestion, A2A, MCP, live-query, and
SurrealKV ownership to end; deadline expiry exits 0 after one bounded
non-blocking `deadline_enforced` write and never reports graceful completion.

**Rationale.** The immutable candidate passed the 10,800-second traffic soak
but Docker later sent SIGKILL and observed exit 137. Focused baseline controls
showed the existing implementation spent the full configured timeout before it
began draining, so held work had no remaining internal margin. An absolute
deadline fixes the observed semantic defect without adding a dependency,
protocol, provider, or public API.

**Uncomfortable constraint.** Forced exit intentionally abandons cleanup still
blocked at the deadline. The safety claim is bounded process termination, not
completion of every cleanup branch. Only the normal path may emit
`graceful_complete`; the parent three-hour certification must still restart
from zero on the committed source SHA.

---

## 2026-08-22 — Production image builds consume the repository's dated Rust channel

**Decision.** Keep `nightly-2026-07-18` as the repository and Docker default,
and make the production backend invoke `cargo +"${RUST_TOOLCHAIN}" build
--release`. A local preflight rejects a floating selector or any disagreement
among the Docker default, repository channel, and effective build argument.

**Rationale.** The Docker toolchain stage already installed the intended dated
channel. The defect was the later floating `cargo +nightly` selector, which
bypassed that declaration and resolved to an incompatible compiler. Explicit
selection fixes the causal fault without changing dependencies, features, or
the repository toolchain decision.

**Evidence binding.** The product/build implementation is committed first and
verified from a clean detached checkout. A direct evidence-only child commit
then records those results. Canonical KBD resolves that evidence SHA as the
parent handoff, and the parent rebuilds it and restarts the 10,800-second
certification from zero.

**Uncomfortable constraint.** A passing isolated Cargo probe is not a release
image result, and a passing clean image build is not an operational-resilience
result. Each remains limited to its recorded profile and source SHA.

---

## 2026-08-22 — UAR 1.0 closes on bounded real-inference functionality

**Decision.** Supersede the multi-hour soak, supply-chain, RC, and GA-publication
tail with five real-model functional paths, each observed through both the
packaged API boundary and the shipped UI. Cancel the four old release-tail
changes rather than representing them as passed.

**Rationale.** The operator's final acceptance boundary is working software:
OpenAI proxy inference, skill activation, knowledge grounding, Kimi k3 UI
configuration and inference, and basic-agent creation and inference. Synthetic,
recorded, and duration-only checks do not establish those behaviors.

**Uncomfortable constraint.** This decision closes only the requested local
`server-full` functional scope. It produces no supply-chain, release-candidate,
publication, minimal-profile, or embedded-mobile claim.

---

## 2026-08-23 — Documentation publishes only through protected `main`

**Decision.** Preserve the `github-pages` environment's `main`-only deployment
policy. Feature branches may assemble the complete artifact, but public
deployment occurs only after an approved merge and is followed by an
independent live-route validation.

**Rationale.** The feature-branch run proved artifact assembly but GitHub
rejected deployment before `deploy-pages`. Weakening the environment would have
made the plan appear to pass by changing the trust boundary. PR #263 instead
merged the reviewed artifact, and protected run `32638082981` deployed and
validated the exact merge SHA.

**Uncomfortable constraint.** This makes pre-merge live Pages validation
impossible. Local complete-artifact and feature-branch assembly evidence must be
kept distinct from the live claim until the protected `main` run succeeds.

---

## 2026-08-23 — Windows native service compile evidence uses MSVC

**Decision.** Compile the Windows `server-full` service for
`x86_64-pc-windows-msvc` with a target-scoped cargo-xwin environment. Do not use
the originally planned `x86_64-pc-windows-gnu` target, and do not export
cargo-xwin's global `TARGET_CC` or `TARGET_CXX` into host build dependencies.

**Rationale.** The pinned `ort-sys 2.0.0-rc.13` distribution has Windows
prebuilts only for MSVC. The GNU check failed before UAR type-checking with no
available ONNX Runtime artifact. A normal cargo-xwin invocation then exposed its
Windows compiler as the macOS host compiler for an `aws-lc-sys` build dependency.
Using only target-qualified compiler, linker, SDK, and CMake variables allowed
the complete `server-full` graph and the native SCM adapter to type-check.

**Uncomfortable constraint.** This is cross-compilation evidence from macOS,
not execution under Windows SCM. Runtime behavior remains unverified until a
Windows host runs the packaged service.

---

## 2026-08-23 — Windows native service runs as LocalService

**Decision.** Configure `PrometheusUniversalAgentRuntime` to run as the built-in
Windows LocalService identity. Grant it read-only access to ProgramData config
and credentials, and modify access only to runtime state and `.prometheus` logs.
Do not run UAR as LocalSystem.

**Rationale.** Provider inference needs outbound networking and mutable runtime
state, but it does not require unrestricted machine authority. LocalService is a
native SCM identity with substantially less local privilege than LocalSystem;
SID-based ACLs avoid localized account-name assumptions.

**Uncomfortable constraint.** This ACL and identity contract is parser-checked
from macOS only. A Windows host must still observe service registration, file
access, outbound inference, stop handling, and restart behavior.

---

## 2026-08-23 — Native Alibaba configuration uses released Qwen 3.8-Max

**Decision.** Standardize the native Alibaba/Qwen seed and the observed obsolete
Alibaba default on `alibaba/qwen3.8-max`. Resolve service credentials through
canonical `DASHSCOPE_API_KEY`. Migrate only exact phase-owned or observed stale
values; preserve every non-matching operator model, credential reference,
endpoint, and custom provider block. This is recorded canonically as
`native-qwen-3-8-max` at KBD revision 404.

**Rationale.** The operator selected Qwen 3.8-Max after the installed service
exposed a restart failure caused by `QWEN_TOKENPLAN_API_KEY`, an obsolete
reference that does not name any variable loaded by the native environment.
Alibaba's current model page identifies the released API model as
`qwen3.8-max`, with a 1,000,000-token context and 131,072-token maximum output:
https://www.alibabacloud.com/help/en/model-studio/qwen3-8-max. Alibaba's upgrade
notice says the preview was retired in favor of that release. This machine has
`QWEN_API_KEY`, which wins the already-locked alias precedence, so the existing
Singapore pay-as-you-go endpoint remains the correct endpoint for this install.

**Uncomfortable constraint.** This correction does not prove Alibaba inference;
the phase's bounded real-inference matrix did not require it and its six-request
ceiling is already exhausted. The post-correction claim is limited to exact
migration behavior, provider/model visibility, and successful service restart.

---

## 2026-08-23 — Qwen 3.8 catalog visibility advances the models.dev gitlink

**Decision.** Supersede the proposed UAR `/api/models` overlay with an advance
of the `models.dev` parent gitlink from `03e217866` to upstream `196cecf3a`.
Keep the compile-time catalog as the single Models API/UI source and leave
submodule source unmodified.

**Rationale.** After the corrected native restart, the configured-provider API
showed Qwen 3.8 while `/api/models` still reflected the old pinned catalog. The
operator confirmed the Know-Me-Tools catalog had been updated. Fetched upstream
commit `196cecf3a` contains both `models/alibaba/qwen3.8-max.toml` and
`providers/alibaba/models/qwen3.8-max.toml`, so the existing architecture can
carry the release without a second overlay or local catalog patch.

**Uncomfortable constraint.** The upstream commit contains two Eden AI paths
that differ only by filename case. On macOS's case-insensitive filesystem, Git
reports one of those unrelated paths as dirty after checkout. The parent commit
records only the exact upstream gitlink; no submodule file is staged or authored
by this phase.

---

## 2026-08-23 — Qwen catalog visibility requires the reviewed offline snapshot

**Supersedes.** The preceding assertion that advancing `models.dev` alone would
carry Qwen 3.8 into the release binary is false. UAR's `build.rs` embeds
`catalog/provider_catalog.json`; it does not read either catalog submodule.

**Decision.** Pin `models.dev` at `196cecf3a` and `vendor/git/liter-llm` at
`788877f7a`, refresh the locked path-package graph, and regenerate UAR's
reviewed offline snapshot from `liter-llm`'s synchronized provider and model
schemas. Keep `/api/models` unchanged and add no configured-model overlay. This
is recorded canonically as `native-qwen-catalog-snapshot-sources` at KBD
revision 405.

**Rationale.** The first release built after only the `models.dev` advance still
omitted Qwen 3.8 from `/api/models`. The updated `liter-llm` commit contains the
released model, and the refreshed 316-provider snapshot exposes it through both
the API and shipped Models UI. A second refresh produced the identical
`c4704316...ded1bb6` digest.

**Uncomfortable constraint.** Advancing `liter-llm` from 1.12.0 to 1.18.1 also
changes its locked transitive graph. The exact `server-full` release build
passed, but this phase makes no claim for other profiles or for Windows/Linux
runtime execution.

---

## 2026-08-23 — models.dev uses the newest clean Qwen-containing revision

**Supersedes.** The `models.dev` pin at `196cecf3a` is not acceptable for a
macOS-supported repository. Upstream commit `91aae6c23` introduced two Eden AI
filenames that differ only by case, so every checkout on the default
case-insensitive macOS filesystem reports a dirty submodule.

**Decision.** Pin `models.dev` at `f97df19af`, the newest ancestor before the
case collision. It already contains Qwen 3.8-Max. Keep `liter-llm` at
`788877f7a` as the actual locked input to UAR's reviewed offline catalog.

**Rationale.** Artifact QA reproduced the dirty checkout at upstream HEAD. The
selected ancestor has no case-folded path collision, checks out cleanly on this
host, and contains 23 Qwen 3.8-Max catalog records. UAR's generated catalog
remains unchanged because its source is the pinned `liter-llm` snapshot.

**Uncomfortable constraint.** This intentionally does not use current
`models.dev` HEAD. Advancing past `f97df19af` remains blocked on an upstream
rename or a repository-wide decision to abandon clean case-insensitive macOS
checkouts.

---

## 2026-08-24 — Reconcile legacy KBD phases by evidence, not uniform completion

**Decision.** Terminalize the 51-phase legacy inventory as 45 complete and six
cancelled. A phase is complete only when its own artifacts or a named successor
close its outcomes. Assessment-only, unvalidated, abandoned release, and mixed
unfinished phases are cancelled even when some implementation landed.

**Rationale.** The imported canonical state left 43 top-level phases pending,
but most already had reflections or successor evidence. Treating all 43 alike
would either hide completed work or falsely claim outcomes that were never
delivered. Legal KBD transitions preserve the event history while the
reconciliation ledger preserves why each disposition was chosen.

**Uncomfortable constraint.** Cancelling the old hybrid-architecture phase does
not make mobile irrelevant. Mobile remains Experimental in the support matrix;
future mobile work needs a new bounded phase instead of resuming the mixed
4/12-era plan. Likewise, the old production-hardening phase is cancelled
because certification and publication were not performed, so this cleanup
makes no GA or release claim.

---

## 2026-08-24 — Canonical repository UI/UX skill lives under `.agents`

**Decision.** Track UI/UX Pro Max once at
`.agents/skills/ui-ux-pro-max/`, with a narrow ignore exception, upstream MIT
license, installer lock metadata, and relative tool links. The existing
AGENTS/CLAUDE routing remains authoritative through its durable roster pointer.

**Rationale.** The installer already uses `.agents` as the canonical payload and
tool-specific symlinks as entry points. Preserving that layout gives fresh
checkouts the skill that the instructions mandate without duplicating a large
search corpus across tools.

**Uncomfortable constraint.** The installed payload includes two upstream tests
whose required refresh/evaluation scripts were not installed. They are not
silently deleted or patched: 130 payload-compatible tests pass, while the two
upstream-layout-only import failures remain documented until the installer
ships their dependencies or excludes those tests.

---

## 2026-08-25 — Canonicalize settings routes at the frontend transport boundary

**Decision.** Convert internal settings namespaces with the existing
`namespaceToSlug()` function before every GET. Keep the backend's canonical
plural and hyphenated routes unchanged, and do not add aliases for defective
client paths.

**Rationale.** Saves already use this conversion. Reusing it makes reads and
writes share one route contract and fixes `provider`, `context_management`,
`native_tools`, `llm_failover`, and other underscored namespaces without
changing persistence or payloads.

**Uncomfortable constraint.** The requested full frontend gate is not green on
the merged baseline: 12 provider-store/A2UI tests and three boundary findings
remain. They do not exercise the settings read transport, but this phase makes
no repository-wide certification claim.

---

## 2026-08-25 — Bound provider defaults and treat visible secret masks as round-trip placeholders

**Decision.** Render provider default models through the repository Base UI/shadcn select using the provider row's enabled `models[]` inventory. Mask stored string secrets with one `*` per Unicode character, and restore any submitted all-asterisk legacy or length-preserving placeholder from the existing value at the settings API boundary.

**Rationale.** Bounded selection prevents unsupported model IDs without adding a catalog transport, while schema-guided server masking preserves the requested visual length and keeps plaintext out of responses. Server-side restoration prevents an unrelated provider edit from replacing the real credential with its visible mask.

**Uncomfortable constraint.** A literal all-asterisk replacement secret is indistinguishable from the required display mask, and generic arrays containing sensitive values remain positionally restored. Those cases require a future out-of-band sentinel or stable-identity contract; they are not silently claimed as solved.

---

## 2026-08-25 — UI design review order is repository policy

**Decision.** Run Impeccable first, Anthropic `frontend-design` second, and UI/UX Pro Max third. Initial UI ideation, evaluation, refactoring, and refinement use two isolated Impeccable critique agents followed by a distinct fresh-context adversarial review.

**Rationale.** The ordering preserves one primary design authority while still applying agentic-product and general design guidance. Independent critics exposed concrete empty-state, compatibility, and security-boundary failures that deterministic happy-path checks missed.

**Uncomfortable constraint.** The installed artifact-refiner adapter lacks its canonical runtime files. Until that package is repaired, the repository can record the required dual critique and adversarial evidence but cannot honestly claim formal artifact-refiner QA.

---

## 2026-08-25 — Search provider inventories at eight and keep dirty drafts saveable

**Decision.** Keep the simple Base UI select for one through seven enabled provider models and switch to the installed bounded Base UI Combobox at eight or more. Search display names and raw IDs without free-form values. Derive modified state from the existing settings draft; disable Refresh while dirty or busy, but keep Save available for dirty drafts during background refresh.

**Rationale.** Short inventories remain faster to scan without a search field, while the exact eight-option boundary is explicit and testable. The provider's enabled model list remains the validity boundary. Keeping Save available avoids stranding the only recovery action if a background refresh stalls.

**Uncomfortable constraint.** The provider panel now sits at 599/600 lines under the decomposition gate, and its responsive contract has structural tests but no real-browser narrow/zoom capture. Future panel growth must extract a coherent provider-card component rather than compressing behavior into the remaining line.

---

## 2026-08-25 — Prompt caching is a durable policy, not a frontend toggle

**Decision.** Resolve prompt caching in one ordered policy seam: request override, persisted session override, verified JWT user override, then the system-global default. New installs seed Off. Anthropic receives explicit ephemeral cache controls only when the resolved value is On; OpenAI remains provider-managed and its request body is unchanged.

**Rationale.** A missing settings route and an unconditional Anthropic strategy created two incompatible truths: the UI could not persist the global setting, while some runtime paths cached regardless of operator intent. Central resolution makes chat, tool loops, compatibility, graph execution, and failover consume the same effective decision and source.

**Security boundary.** Remote/admin deployments require an exact constant-time `settings_admin_key`; generic settings reads cannot bypass the protected prompt-caching namespace. The loopback-only macOS package retains its explicit mutation-auth-disabled default, and the installer now adds that default to legacy configs only when the key is absent.

**Uncomfortable constraint.** OpenAI automatic caching cannot be disabled by UAR, the provider-default ephemeral TTL is not configurable, and no Anthropic credential was available for a supplemental live cache creation/read. Stub upstream bodies and provider-usage fixtures are the authoritative evidence for this delivery.

---

## 2026-08-26 — Use native container queries for provider-panel width

**Decision.** Retain the installed Tailwind CSS v4 and Playwright stack. Use a Tailwind native container query for the provider field grid and focused Playwright proof for constrained-panel and desktop states. Add no plugin or resize-observer hook.

**Rationale.** The requirement switches between exactly one and two columns according to available provider-panel width. Native container variants express that state transition directly; Playwright already supplies browser geometry, overflow, and keyboard assertions. Intrinsic `auto-fit/minmax` remains a reference but can exceed two columns without another cap.

**Uncomfortable constraint.** Analyze exceeded its Tier 1 research cap by one request because a bundled metadata command was counted after dispatch. The result uses official documentation and installed-stack evidence, but future research must reserve budget before batching calls.

---

## 2026-08-27 — Certify provider responsiveness from the production bundle without dependency scope expansion

**Decision.** Keep the provider-width repair class-only and certify the already-passing production bundle with Playwright after the normal Vite development server failed before rendering on the optional `loro-crdt` peer. Do not add or change a dependency, optimizer configuration, state authority, or runtime transport to route around that environment defect.

**Rationale.** The Tier 2 production build is the deliverable artifact and exercises the same final responsive classes. Its preview supports deterministic container geometry, keyboard operation, draft preservation, popup positioning, and zero-write assertions without confusing an unrelated development optimizer limitation with a product-source requirement.

**Uncomfortable constraint.** Production-bundle certification does not repair the normal development path. The optional-peer root-resolution defect remains explicit in the phase reflection and needs a separately authorized maintenance change if it is to be fixed.

---

## 2026-08-27 — Local anonymous governance is a boot-proven runtime posture

**Decision.** Default governance Off only when the configured host literal is exactly `localhost` or `127.0.0.1`, installed JWT authentication is disabled, persistence has durably supplied the Off preference, and every registered tool-capable bound ingress is sealed and loopback. Any missing proof, non-loopback listener, required JWT, unreadable preference, or failed normalization keeps the effective gate On. The persisted preference, authoritative runtime status, and unsaved UI draft remain separate state.

**Rationale.** A mutable setting or host string cannot establish on-device-only reachability. The sealed boot inventory makes the bypass decision at the real request-admission boundary, while separate gate/mutation/status handles prevent frontend state or a partially completed write from authorizing tool execution.

**Operational boundary.** The process emits one stable `governance.inactive_local_mode` warning after its first finalized Off transition. Operators can turn enforcement On or Off live only while the boot posture remains eligible; Required and mutation-unavailable states are truthful, reachable UI states rather than disabled-looking booleans.

**Uncomfortable constraint.** The forward release was installed before the planned rollback artifact was built. The staged fail-closed variant and the locally available prior binary both passed isolated compatibility checks, but no committed pre-deployment rollback deliverable exists, so rollback certification remains incomplete.

---

## 2026-08-28 — Treat rollback normalization as a recoverable state migration

**Decision.** The supported fail-closed rollback candidate may normalize a
seed-owned `governance.enabled=false` default to On, while an API-owned row with
a durable `updated_at` marker is preserved. Before downgrade, retain the complete
typed row and checksum with both candidate binaries. After returning forward,
restore the exported Off preference only when the row was seed-owned and the
known rollback normalization is the only change.

**Rationale.** The live shared-database exercise disproved the earlier
row-preservation assumption: rollback correctly enforced On and rejected
mutation, but forward subsequently read On until the operator preference was
restored. Treating this as a named state migration keeps rollback fail-closed
without silently losing operator intent.

**Stop condition.** If the current row differs from both the exported value and
the known normalized On value, preserve it and require operator resolution; do
not overwrite a possible concurrent change.

**Uncomfortable constraint.** Fail-closed rollback is not transparent for a
seed-owned default. API-owned preference remains durable, but a never-edited
local default needs an ownership-aware recovery check.

---

## 2026-08-28 — Publish governance settings only after runtime authority

**Decision.** Suppress implicit database notifications for
`governance.enabled`. After durable write, cache update, and coherent runtime
publication succeed, publish one explicit settings realtime event containing
the accepted boot instance and revision. Notification delivery failure is
observable but non-transactional.

**Rationale.** A database-live event can arrive before the in-process authority
publishes its new revision, causing clients to refetch stale effective state.
The explicit event makes the notification an after-commit observation of the
same authority returned by the mutation response.

**Uncomfortable constraint.** A failed realtime publish can still delay another
client's observation until focus, reconnect, Refresh, or bounded revalidation.
The durable value and runtime authority remain committed; delivery is not
misreported as a rolled-back mutation.

---

## 2026-08-28 — Record dependency releases by accepted source commit

**Decision.** Pin Liter 1.18.2 at `c5c6caac`, Surreal Memory at `432eaa1e`,
the Skill System parent at `ad5c82c6`, SurrealDB crates/server at 3.2.4, and the
SurrealDB container by the exact 3.2.4 tag plus immutable OCI digest. Preserve
UAR's standalone Surreal Memory manifest adaptations while copying reviewed
implementation files byte-for-byte from the accepted leaf.

**Rationale.** Exact source and image identities make the recursive source
archive, local native release, and rendered deployment inputs independently
auditable. Recording the source commits rather than only merge commits keeps
the reviewed trees usable as gitlinks while remote-main reachability proves
publication.

**Uncomfortable constraint.** The canonical offline acceptance profile is
`minimal`; the `server-full` profile links ONNX Runtime and cannot reconstruct
that native dependency without network access. `server-full` remains the local
Tier 3 binary profile and is not claimed as the offline archive profile.

---

## 2026-08-28 — Render chat artifacts with production A2UI v0.9.1

**Decision.** Treat A2UI v0.9.1 as the production chat-artifact protocol. Emit
effective-policy artifacts as `createSurface`, `updateComponents`, and
`updateDataModel` messages using the certified UAR catalog; process them with
`@prometheus-ags/a2ui-core` and render them with `@prometheus-ags/a2ui-uar`.
Keep v1.0 candidate content unsupported until it becomes the production line.

**Rationale.** The official A2UI specification identifies v0.9.1 as Current
Production and v1.0 as Candidate. Rendering through the canonical processor and
catalog preserves protocol validation and component allowlisting instead of
turning serialized messages into an unstructured text surface.

**Uncomfortable constraint.** Invalid and over-budget artifacts cannot retain
unbounded original frames in the render path. They expose a bounded diagnostic
source excerpt and remain visibly invalid rather than attempting a partial or
unsafe render.

---

## 2026-08-29 — Reconcile the standard agent skill library as startup metadata

**Decision.** Resolve the current user's `~/.agents/skills` directory on every
server boot, recursively load physical `SKILL.md` manifests without following
descendant links, and support bounded top-level alias surfaces. Persist new or
semantically changed definitions under the `agent-skills` provider while
preserving operator enabled/scoped state and retaining records whose source is
temporarily absent.

**Rationale.** Startup reconciliation makes the cross-agent standard directory
immediately available to UAR and gives every restart a deterministic new/change
check. Metadata-only persistence keeps readiness independent of embedding-model
latency; changed definitions clear stale vectors and remain available through
the default keyword matcher.

**Boundary.** A literal alias target may contain an ancestor selector such as a
plugin's `current` symlink, but the target entry itself and links below the
resolved target are not followed. Durable reconciliation failures stop startup;
missing or unreadable optional sources are counted and remain non-fatal.

**Uncomfortable constraint.** The standard tree can publish the same physical
manifest through both its physical path and supported alias entrypoints. Those
entrypoints retain distinct path-derived identities; this host currently
publishes 1,038 `agent-skills` records rather than collapsing aliases by inode.

## 2026-09-02 — Adopt Codex reliability patterns, not Codex

**Decision.** The codex-harness comparison treats codex-rs as a source of
reliability patterns to adapt behind UAR's provider-neutral seams, not as an
architecture to port. Explicitly excluded: Responses-API `previous_response_id`
resumption and sticky routing, ChatGPT-auth gating and vendor agent identity,
unsandboxed MCP child processes, the shadow skill selector as a correctness
gate, and vendor-catalog base instructions. A guardian-style model reviewer is
to be evaluated as a tier behind Cedar, never in front of it.

**Rationale.** UAR's mission is a universal runtime; every surveyed harness
with a single provider is free to bind to that provider's transport. UAR's
equivalents must live behind liter-llm and the driver trait so a pattern that
improves one provider does not regress the other 140.

**Evidence correction.** The supplied prior analysis said native and MCP tool
registries are combined late. They are combined early and frozen for the run
(`src/llm/orchestrator.rs:498-512`, `:601`). The correct fix is per-step
re-projection of the tool set, not an earlier merge.

**Uncomfortable constraint.** The 1,510-line `start_run_with_policy_and_history`
has no seams to shadow behind, so the shadow-mode migration the analysis
requires must first cut seams into the monolith, which is itself a risky change.

## 2026-09-02 — Harness change set: seams before structure

**Decision.** The Codex-informed harness work ships as ten OpenSpec changes ordered so that five correctness changes (history integrity, fail-closed tool arguments, deterministic prompt assembly, progressive skill runtime, model-path resiliency) land before typed turn assembly, MCP projection, thread-native subagents, and project instructions. Flipping the harness default to the typed path is its own change gated on a recorded parity report and live smoke evidence.

**Rationale.** The 1,510-line run entry point has no seams to shadow behind. Each correctness change extracts a pure function that the typed assembler later composes, so shadow parity compares against a legacy path that already has deterministic ordering and validated tools. An intentional-delta allowlist, each entry naming the change that introduces it, keeps "intentional" from becoming a place to hide regressions.

**Uncomfortable constraint.** Two adversarial rounds still ended in BLOCK; the round-2 fixes were applied after the cap and go to the plan stage un-vetted. `versions.toml` entries for jsonschema and the other pins are operator actions that block execution of change 2 until done.

## 2026-09-02 — Typed assembly remains opt-in pending phase-end evidence

**Decision.** Keep `HarnessConfig::default().mode` and the `harness.mode` settings-schema default at `legacy`. `typed-turn-default-flip` still requires both the parity-corpus report and recorded live shadow smoke with zero unexpected differences. Neither evidence record has been produced during this implementation pass.

**Mechanism.** `typed` uses the staged contributors. `shadow` renders typed prompt sections separately, records redacted per-step differences, and dispatches the legacy request. Shadow reuses the prepared legacy history reduction so it does not issue a second paid summarization request. The checked-in allowlist currently has no exemptions: progressive-skill-runtime already refreshes skill tools on the legacy path, so the historical frozen-list assumption cannot justify an exemption now.

**Uncomfortable constraint.** Shared history reduction is not an independent check of the reducer algorithm; its correctness still depends on the phase-end history tests. Production code has not been compiled or tested in this implementation pass. Graph-node tool-loop integration remains work for thread-native-subagents, and no full-change acceptance is claimed for changes 3–6.

## 2026-09-02 — World-state ownership and round-4 independence

**Decision.** Continue change 9 while the change-7 sandbox choice remains unresolved. The plan's gate says "Before Round 4 change 7"; changes 8 and 9 have no dependency on that decision. The earlier checkpoint's blanket stop interpretation is superseded.

**Mechanism.** Project trust comes only from host configuration and defaults to an empty list. Project bodies carry escaped Host markers; enforced policy is unchanged. Session-owned baselines are not deserialized from client/session input. A captured clock bucket and instruction snapshot feed both assembly paths. Context reduction reserves space for full replay, signals all history rewrites, then world state is inserted before active skill bodies are budgeted. Only the selected assembly path commits its update.

**Uncomfortable constraint.** This code is uncompiled and untested pending the operator's phase-end sequence. Shared host capture and legacy reduction in shadow are not independent verification. The existing graph path still needs the planned change-8 adapter work for governed native tool execution and per-call typed snapshots; no change-9 test or acceptance claim is made here.

## 2026-09-02 — Thread admission must not undercount uncertain persistence

**Decision.** Use host-derived owner/root lineage and a single tree admission gate for the thread contracts. Model spawn input contains the artifact, delegated prompt, name, and history choice only; it cannot supply root identity or delegation approval. History forking defaults to none and never copies parent system context or tool traffic.

**Mechanism.** New-child reservations count against concurrency, depth, and lifetime before persistence. Once a database write begins, cancellation is no longer proof that no child exists. Dropping that reservation releases concurrency but retains lifetime capacity until the host reconciles the write. Confirmed failure may explicitly abort; confirmed commit transfers the concurrent slot to the live-turn permit. Root ceilings cannot exceed four concurrent children, depth three, or sixteen total.

**Uncomfortable constraint.** These are implemented contracts, not a working child execution service yet. Provider writes and recovery must honor this protocol in the remaining tasks. Compilation, actual concurrent admission, durable ordering, and cancellation behavior remain unverified until phase-end tests.

## 2026-09-02 — Child policy uses frozen host bindings, not fresh resolution

**Decision.** Intersect each child artifact with a host-held parent snapshot. Credential/service bindings, native/MCP tool identities, sandbox bindings, and the root approval run are host-owned. The versioned `uar.thread_policy` extension expresses restrictions only. Existing `uar.run_policy` and `budgets` are decoded strictly at this boundary, including nested fields.

**Mechanism.** Concrete `Selected`/`None` resource sets avoid the observed manager behavior in which `All`/`Auto` omits a filter. Base and extension policy constraints intersect independently; neither erases the other's deny list. Exact credential bindings intersect, approved filesystem capabilities intersect per binding, isolation requirements only strengthen, and each numeric ceiling takes the lower value. Compiler MCP declarations select inherited identities but cannot replace live connection credentials or endpoints. Root approval identity never changes in a child policy.

**Uncomfortable constraint.** A policy value is not enforcement. The upcoming service/adapters must retain exact bindings, pass the captured narrowed artifact and effective policy, enforce sandbox capabilities and root-shared counters, and route approvals only to the root. The current implementation is uncompiled and untested under the operator's phase-end sequence; it is not yet called from execution.

## 2026-09-02 — Thread storage revisions and atomic spawn lineage

**Decision.** Persist each child and its edge atomically and use a storage revision distinct from history revision. Reads and writes require the verified owner; immutable lineage is never upserted or deleted through the thread API. All providers validate and sort decoded records through shared Rust functions.

**Mechanism.** Memory uses one lock. PostgreSQL uses root-first parent/root shared locks, atomic inserts, foreign keys, and revision-guarded updates. SurrealDB uses a single transaction, parent/root snapshot checks plus physical write fences, unique indexes, and inspection of every statement error. New migrations define the tables; SurrealDB initialization explicitly includes its idempotent schema. Failed or interrupted writes require reconciliation, not an assumption of rollback.

**Uncomfortable constraint.** Source implementation does not prove durable behavior. No migration, database query, compilation, concurrency test, or restart test ran in this turn. Phase-end integration tests must establish the atomicity, owner isolation, stale-write refusal, stable ordering, and cold-recovery claims; the upcoming service must honor the uncertain-write reservation protocol.

## 2026-09-02 — Agent descriptors bind to one live host turn

**Decision.** Construct a fresh native registry for each agent-control context. All five controls are model-only descriptors with strict schemas and declared effects; spawn is omitted without explicit authorization from the original registered artifact or a verified root-user grant. A wildcard expanded during policy intersection is eligibility, not delegation authorization.

**Mechanism.** Private host context captures owner, root, caller run, intersected policy, cancellation, and authorization. Calls check current persisted caller identity and native tool binding. Returned records must remain in that root tree; message identities remain metadata, and list output omits prompt/history/result bodies. Interrupt acknowledges a descendant cancellation request without claiming completion. Read-only waits use current-state watches, retain observed terminal outcomes across a concurrent resume, and recheck the caller after subscription setup and waiting.

**Uncomfortable constraint.** This task delivers descriptors and the host dispatch contract, not a concrete execution service. `registry_for_turn` has no manager caller and `AgentThreadHost` has no implementation yet. The adapters must provide real execution, root authority rechecks at mutation, persistence, mailbox ordering, root approval/budgets, and cancellation. No build or behavior test ran; these claims remain source-level until phase-end verification.

## 2026-09-02 — Lifecycle metadata and live graph boundaries

**Decision.** Project child lifecycle from confirmed persisted transitions using a content-free domain payload. Preserve root-stream identity, actual child/parent run IDs, storage revision, stable lifecycle identity, and persisted timestamp. Emit graph step boundaries around each node execution through the existing host event stream, not from a completed trace.

**Mechanism.** The lifecycle projector reuses persistence transition validation and exact lineage checks, suppresses repeated state writes, and never copies result text or raw failure messages. AG-UI subagent mappings use the documented SUBAGENT_STARTED/FINISHED/ERROR schemas; pending or pre-start failure uses a named CUSTOM event because no child run ID exists. Failed/cancelled categories are static host labels. Graph numbering is engine-owned and resumes from checkpoint iteration; cancelled in-flight nodes do not produce synthetic finished boundaries.

**Uncomfortable constraint.** The graph event path is connected to RunManager, but the forthcoming child host must call the lifecycle projector after confirmed commits and retain the correct captured parent turn. There is no durable outbox/restart-delivery proof in this task. No builds, schema tests, replay tests, or cancellation tests ran; all behavioral claims remain unverified until phase end. Existing trace data and the old graph output prefix remain for task 4.2, not silently removed here.

## 2026-09-02 — Actor mailboxes use the shared run host; task 4.1 remains partial

**Decision.** Direct actor prompts enter RunManager with an exact registered artifact, authenticated owner, private kernel session, host-generated run ID, and actor cancellation token. The actor no longer creates an independent orchestrator or history vector. Commit root/start state before kernel entry and terminal state before the reply. Reject missing/anonymous actor identity before request body extraction.

**Mechanism.** The run emitter captures terminal results through a lossless oneshot channel; retained SSE history holds only a weak reference, so disappearance of the producer closes the completion channel. A live mailbox receiver prevents an SSE viewer disconnect from cancelling its work, without changing explicit cancellation. Actor lookup is owner/tenant-qualified and publication reserves the name atomically. Unknown artifact IDs and storage errors do not select the default artifact.

**Persistence recovery.** Retain the exact expected envelope and revision before every write. A failed write may be reconciled only by an owner-qualified read of that exact envelope, never by retrying the mutation or assuming absence proves rollback. Same-request confirmation permits continuation. Later confirmation of an unstarted turn closes that turn as failed/cancelled before accepting new input. Terminal-write recovery preserves the actual recorded result. Shutdown attempts the same reconciliation. An unavailable or different snapshot leaves the actor unresolved.

**Uncomfortable constraint.** This is not completion of task 4.1. Collaboration still starts the target's independent root instead of a source-root child; concrete AgentThreadHost wiring, frozen execution bindings, root approval/budget integration, and restart recovery remain unfinished. Actor registry keys include tenant, but RunManager/thread owner fields remain user-ID based; full cross-tenant isolation is not claimed. Builds, tests, and acceptance review remain deferred to phase end per the operator. Tokio's Sender::is_closed contract was checked through Context7 at https://docs.rs/tokio/latest/tokio/sync/oneshot/struct.Sender.html. No dependency changes were needed.

## 2026-09-02 — Root-scoped thread host and completion status requirement

**Operator instruction.** Execute the `kbd-status` skill after finishing every task, change, or phase. This is an additional completion-boundary action, not permission to run tests before phase end. Read and follow `/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md`; show canonical implementation counts separately from evidence/certification/publication and disclose stale projections.

**Decision.** Add one ThreadService per live root run as the concrete AgentThreadHost. It owns admission, typed mailbox sequencing, tracked mutations/workers, storage transitions, subscriptions, interruption, and lifecycle publication. The required ThreadExecutionHost bridge owns exact artifact/history resolution, binding/sandbox/budget admission, and shared-kernel execution. It has no default methods or alternate model loop.

**Mechanism.** A spawn intersects the parent's captured policy, validates live authority again after resolution/history I/O, reserves a child, atomically persists the child and edge, then starts its worker. Only confirmed transitions publish. Unknown writes retain their exact expected envelope and child reservation; explicit reconciliation closes unstarted recovered work rather than replaying it. Accepted queued triggers retain the existing active-child slot, preventing another spawn from stranding an acknowledged message between turns. Model execution runs outside the root storage lock; roots retain worker handles and disappearing executor tasks produce failure outcomes.

**Uncomfortable constraint.** The service is implemented and exported but not constructed on the production path yet. ThreadExecutionHost has no concrete RunManager implementation. Root attachment must be single-instance and account for the actor's later root turns; frozen resource bindings, root approvals/per-call budget enforcement, root message consumption, and root completion publication still need manager/actor wiring. No build, formatter, tests, or acceptance review ran; task 4.1 remains open.

## 2026-09-02 — Serialize root approval requests before enabling child execution

**Observed boundary.** RunManager stored one replaceable oneshot sender per run and published the approval event before inserting that sender. Sharing this path with concurrent children would overwrite siblings; an immediate human response could also arrive before the sender existed. The thread specification requires child approvals to use the root channel without granting child-supplied approval authority.

**Decision and implementation.** Replace the production slot map with `thread/approvals.rs`: one host-registered root channel, serialized pending requests, request IDs minted by the host, registration before publication, and synchronous drop cleanup. Five minutes bounds queueing, event publication, and decision together. Root and caller cancellation interrupt the wait; the root emitter and root run identity are captured in a request-only channel. The broker resolver remains on RunManager. Weak channel indexing does not retain completed run emitters. Child channels cannot accept run-only decisions; the exact `approval_id` is required. Ordinary root requests keep the existing run-only API for compatibility. This means uncorrelated legacy root replies still lack replay correlation; no broader replay guarantee is claimed.

**Wiring.** RunManager now uses this broker for real root runs, registers it before assembly, and accepts an internal inherited-channel parameter for the forthcoming child entry. Both HTTP approval routes accept optional `approval_id` after their existing owner lookup. The normalized event and official/legacy/runtime event mappings expose the ID. Runtime approval entity IDs use the request identity to avoid sibling tool-call-ID collisions. A malformed non-string ID on the raw JSON route is rejected, not treated as absent. Cancellation is checked before governance, and the future inherited-child path cannot use the root's local governance toggle to erase narrowed Ask/Deny policy.

**Uncomfortable constraint.** Every existing kernel caller still supplies no inherited channel. Child execution, resource freezing, actor collaboration, and root-shared usage enforcement are not integrated. Current browser clients send only `{ approved }`; they continue working for ordinary roots, but must carry the event's request ID before child approvals are exposed there. No frontend edits were made. Old standalone approval helper tests are retained under `cfg(test)` and do not test the new broker; phase-end integration coverage must replace or supplement them. No build or tests ran. Task 4.1 remains open, not accepted.

## 2026-09-02 — Pin executable connections, not mutable service names

**Observed boundary.** MCP filtering shares mutable service slots; configuration replacement and reconnection can change their credentials/endpoints. Skill activation starts dependencies from each skill's source configuration. The vendored LLM client's constructor resolves environment keys. All three are incompatible with a child's inherited immutable execution bindings.

**Decision.** Add frozen MCP views that retain exact transport Arcs, recheck selected tool descriptors on those same transports, and refuse replacement, revocation, closure, reconnect, resource merge, and tool registration. Filtered frozen views retain exact connections and have child cancellation tokens. Closing a borrowed view disables that view, not shared parent transports; removing an owned server revokes retained slots. Frozen skill activation validates inherited dependencies instead of executing the artifact's command/URL/auth/env recipe. Ordinary root activation remains unchanged.

**Model binding.** Add an object-safe, refusal-by-default `LlmDriver::with_bound_model`. LiterLlmDriver reuses its existing DefaultClient for a qualified model in the same provider. AnthropicDriver retains its HTTP client, key, endpoint, and defaults while changing only the model. The host must still check policy and binding identity; this method is not authorization. No vendor code or dependency pin changed.

**Uncomfortable constraint.** No production root capture calls freeze_bindings or with_bound_model yet. This supplies the missing binding primitives, not the concrete ThreadExecutionHost/actor integration. McpRegistry::merge now returns McpMergeError so immutable-binding rejection is distinguishable from a descriptor collision; downstream callers that explicitly name its former error type need migration. No compilation or behavior tests ran. Task 4.1 remains open. Asked the operator asynchronously for change 7's planned sandbox choice while continuing independent work; no decision has been inferred.

## 2026-09-02 — Consume captured model and skill resources in real root turns

**Decision.** `turn/bindings.rs` captures a primary model client, healthy
fallback clients, configuration, and health binding once after root model and
credential resolution. The shared manager uses these exact clients for initial
summarization, ordinary tool-loop execution, and its graph driver. Host-supplied
model drivers now also handle summarization; that path must not construct an
unrelated credential-bearing client. Existing fallback ordering and unavailable
fallback skip behavior remain.

**Skill capture.** A read-only matching view captures skill definitions,
enablement, legacy agent bindings, and matching config. Manager matching,
catalog generation, and activation share its registry. Vector retrieval may
contribute scores for captured IDs but cannot replace a captured skill body.
The global service still owns mutations/evolution for later runs. The snapshot
retains host embedding/search resources; it does not claim frozen vector
ranking, child budget accounting, or a complete capability-free registry.

**Verification policy.** Tests remain at phase end. The repository's required
compile-only Tier 0 check is distinct from tests and was run. It exposed and
led to fixes for the existing BackON feature/lock mismatch, three compile
errors, and 23 warnings. The final locked server-full check passed, zero
warnings, in 30.92s. No package upgrades, formatter, test runs, or acceptance
critic. Redacted debug output excludes prompts, message content, credentials,
and executable handles. Missing actor persistence now refuses admission rather
than referencing a backend absent from the build.

**Uncomfortable constraint.** These are connected root preparation paths, not
completed child execution. The optional kernel argument still carries only
approvals; all callers still pass None. No concrete ThreadExecutionHost or
production root MCP freeze exists. Task 4.1 and the 2/10, 83/182 counters remain
unchanged. The status skill was executed at this checkpoint and remains
mandatory after every genuine task/change/phase completion.

## 2026-09-02 — Put root cost admission on captured model drivers

**Requirement and mechanism.** Thread execution requires root-shared model-call
budgets. The existing tracker reported Exceeded only after a run. ModelCallBudget
now captures the payer scopes and cancellation token; primary and fallback
clients receive BudgetedModelDriver wrappers, reused by summarization and graph
execution too. Each call checks the ledger before dispatch. Received priced
Usage updates atomically replace that request's cumulative cost estimate across
all scopes. Same-client model rebinding retains this payer and guard. Agent cost
ceilings are installed before driver capture, and run completion no longer
double-charges. Scope gauges and threshold traces remain on the call path.

**Boundary.** Ledger poisoning refuses new paid work instead of authorizing from
possibly partial accounting. Root cancellation interrupts local startup/stream
waits; it does not claim to stop billing already accepted by a remote provider.
Accounting is independent of optional cost-display configuration.

**Uncomfortable limits.** This is not complete task 5.1 or task 4.1: concrete
child hosting, narrowed token/rate/tool/time budgets, missing/unpriced usage,
in-flight reservation, and child admission remain unfinished. Existing durable
cost roll-ups still run at normal-run completion and omit graph/summarization
calls; no durable accounting completeness is claimed. The existing agent-scope
identity policy is unchanged. Final locked server-full compile: exit 0, no
warnings, 7.85s. No tests or acceptance check ran.

## 2026-09-02 — Child assembly uses inherited resources, not root resolution

The optional execute_request_inner input is now InheritedRunBindings rather
than an approval-only channel. It carries the thread/policy/control identity,
model and skill snapshots, frozen MCP view, native handlers, captured harness
settings, cwd, and root approval capability. The child branch validates owner,
run, private session, canonical-history presence, control-policy identity and
root approval identity before touching session state. It uses the narrowed
artifact and policy without global policy backfill. It bypasses provider,
credential, skill-index, and fresh payer construction. Parent activation/agent
handlers are excluded before child-local handlers are registered.

Captured model clients now have opaque credential grants. Child route selection
requires the exact grant, reuses or rebinds an existing driver, and retains its
root-budget wrapper. Keep the still-authorized binding catalog separate from
this turn's selected primary/fallbacks: choosing one provider does not revoke
other provider grants that the child's policy still permits its descendants to
use. Connection recipes are stripped from the child's copied LlmConfig.

The uncomfortable thing: every current caller still supplies None. This is a
compiled inherited assembly branch, not functioning end-to-end child execution.
Root capture/attachment, concrete ThreadExecutionHost, actor lifetime and actual
collaboration, enforceable sandbox restrictions, full budgets, cancellation
cleanup and phase-end acceptance remain required before task 4.1 is done.

## 2026-09-02 — Actor conversations span distinct root runs

ActorThreadSession now creates a fresh persisted root for each direct turn,
while retaining its owner-qualified session_id and canonical conversation
history. The old root stays in persistence. Updating run_id on the original
root retained a stale root_run_id and contradicted ThreadService::attach's
fresh-root authority check. An unresolved/live predecessor cannot be replaced.
The existing exact-write reconciliation path also handles the next root's
creation without replaying the prompt or retrying an uncertain write.

The actor's terminal reply is now delayed until its executing kernel future
unwinds. RunCompletionCapture freezes a terminal event; RunCompletionGuard
releases the reply at producer exit. Short synchronous capture locking permits
this non-async drop boundary. A producer panic reports failure; absence of a
terminal event closes the channel instead of inventing successful output.
Actor stop/shutdown_all now cancel and join mailbox tasks outside the registry
lock, rather than detaching them by dropping their JoinHandles.

Uncomfortable limits: ActorInfo.thread_id is the latest root, not a stable
conversation ID; clients use session_id for the latter. Joining the mailbox and
main kernel future does not join separately spawned cost/evolution maintenance,
remote billing, or the still-unwired descendant service. This does not complete
task 4.1, real actor collaboration, or tree-wide cancellation. The final
compile-only T0 passed without warnings in 18.13s; behavior tests remain at
phase end.

## 2026-09-02 — Reject unsupported MCP stdio sandbox requests

projected-mcp-runtime task 0.1 chooses its explicit rejection alternative.
Codex commit 986ff1cc7ced0081ec5014b700a376333d87f869 requires permission-profile,
network-proxy and platform-helper integration; UAR SandboxRunner does not expose
a long-lived stdio launcher. Source paths and the rejected port alternative are
recorded in the phase decision log. No new dependency or sandbox claim.

Shared config validation now rejects sandboxed:true stdio during McpConfig
deserialization and before registry startup/reconnect/provisioning. HTTP and
embedded saves and hydration validate before persistence or connection removal,
including disabled entries. The guard addresses the observed inert sandbox flag
at the trusted host process-launch/configuration boundary. Boot's existing
log-and-empty-registry fallback is unchanged. This does not solve child-thread
physical filesystem/network enforcement.

Four compile-only T0 checks passed with zero warnings (30.97s, 8.23s, 8.01s,
10.18s); behavior testing remains at phase end. Task 1.8 stays open.

## 2026-09-02 — MCP catalog separates declarations from execution

Task 2.1 captures immutable source-qualified server definitions. Global/Skill/
Child provenance determines authority; a second rank cannot contradict it.
Required/optional status and host-observed authentication are explicit, and
sandbox policy derives from the validated config. Same-source collisions fail;
different sources await authority-aware projection, not insertion-order override.

The versioned SHA-256 identity includes every declared launch input, preserving
argument order and sorting env keys with fixed-width length framing. It does
not pretend to capture resolved process environment or credential revisions;
task 2.3 must include those separately. Configuration and hash Debug surfaces
are redacted because configured values can contain secrets. The catalog has no
live handles, I/O or mutation APIs, and is not a grant of executable authority.
No run-path caller yet: production wiring is task 4.1, not claimed by 2.1.

## 2026-09-03 — Source-pinned MCP projection

Task 2.2 uses the resolved eligible IDs for every policy mode and never treats
All as permission to broaden the resolved universe. Server authority is chosen
before tool discovery, then retained exactly. Global > active eligible skill >
current child; equally authoritative conflicting definitions fail. A missing
winner's catalog is an error, not an invitation to use lower-authority tools.

Tool snapshots retain their declaration and completeness. Conflicting full
snapshots cannot be unioned (that would revive deleted tools); stale config/auth
metadata fails, as do wrong-server/non-MCP descriptors and name collisions.
Hidden tools are omitted, deferred tools remain eligible without initial
advertisement, and descriptors retain all governance metadata. The host still
owns concrete binding identity, environment, sandbox and Cedar checks. The
projection is not yet wired into the manager; no live behavior claim.

## 2026-09-03 — MCP binding identity and caller-owned refresh

Task 2.3 uses verified ActorOwner (user and tenant), source-qualified declared
config, required/auth metadata and an exact OS-string environment/cwd snapshot
as the cache identity. No anonymous owner, lossy environment conversion, secret
Debug or ambient fallback is introduced. The host connector must actually use
the supplied snapshot; this task does not claim that later integration exists.

One caller owns a refresh future; watch shares its result with waiters. The
drop guard clears cancellation/failure and generation checks reject stale
publication. Invalidation cancels rather than overlapping a replacement attempt.
Read-heavy RwLock state follows async-patterns without adding a lock dependency;
guards never span awaits. Tokio watch semantics were checked via Context7:
https://docs.rs/tokio/latest/tokio/sync/watch/struct.Sender.html and
https://docs.rs/tokio/latest/tokio/sync/watch/struct.Receiver.html.

Synchronous registry cancellation is separate from awaited transport closure.
Retired handles remain owned through cancellable cleanup and shutdown. The
uncomfortable limit is the still-unimplemented snapshot-aware connector and
manager wiring: production connection reuse and partial-launch cleanup are not
yet demonstrated. Final T0 passed with zero warnings in 31.44s; tests stay at
phase end. No replacement of the existing reconnect-generation mechanism.

## 2026-09-03 — Captured stdio bindings and HTTP dependency prerequisite

New projected stdio launch resolves command paths and environment from the
binding request, without fresh global reads or implicit provisioning. Complete
paginated discovery compiles descriptors before publication. The authoritative
reconnect slot captures both request and complete catalog, so an old view cannot
re-read different launch inputs or install a changed schema after reconnect.
Administrative replacement clears that snapshot with its config update.

The total lazy-call deadline can cancel reconnect while its counter is nonzero.
A ReconnectAttempt drop guard now releases that count on every unwind; ordinary
and snapshot reconnect retain the existing generation check. This fixes a named
cancellation/shutdown sequence, not speculative hardening. Uncomfortable limit:
counter cleanup and SDK cancellation do not prove that UAR shutdown has joined
every partial-launch child; that work and phase-end behavior evidence remain.

HTTP requires an explicit SDK-compatible client to honor captured proxy inputs.
Direct UAR reqwest0.12 is not rmcp3.1.2's reqwest0.13.4. Dependency-pin-discipline
requires the new direct alias pin in operator-owned versions.toml; requested
reqwest_mcp="0.13.4", leaving all manifests unchanged. Cached official index
checksum matches the existing Cargo.lock package; fresh registry access was403.
No version bump or alternative handwritten MCP HTTP protocol was introduced.

## 2026-09-03 — Own partial stdio launches through a host join barrier

Cancellation during handshake/discovery can precede cache publication. Keep
the direct Child in a tracked reaper, not solely inside the handshake transport.
Dropping the transport cancels its reaper; explicit transport close waits for
the reaper result. The runtime connector owns the tracker and awaits it during
shutdown, including attempts that never returned a RunningService. Admission
and tracker registration share a short lock so shutdown cannot miss a spawn.
The async-patterns skill drove this structured ownership; TaskTracker close
alone is not an admission gate (official docs loaded via Context7):
https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html.

The uncomfortable limit: direct-child kill/reap is not a descendant-process
tree sandbox; no such guarantee is claimed. Compile-only evidence passed in
26.57s with zero warnings, not live process proof. Task 3.1 remains unfinished
pending the HTTP pin and adapter; RunManager integration is still task 4.1.

## 2026-09-03 — Optional availability cannot suppress binding failures

Task 3.2 proceeds independently while task 3.1 awaits the HTTP alias pin. MCP
preflight accepts only authority-selected definitions, one verified owner and
captured parent environment. Optional environment/auth/connection/discovery/
timeout failures produce a named warning and omit the selected server's tools.
Required availability failures abort with an actionable, secret-free error.
Invalid binding, revocation, cancellation, shutdown and projection errors remain
fatal regardless of optional status. Otherwise a connector ownership bug could
silently be reclassified as acceptable degraded availability.

The narrowing helper can only remove selected optional servers; it neither
reselects origins nor changes tool eligibility. Cached complete catalogs retain
lazy startup. No credential lookup or tool authorization moved into the kernel.
Uncomfortable limit: runtime API call sites compile, but RunManager integration
and live behavior evidence remain outstanding. T0 passed zero warnings in31.79s;
tests and acceptance review remain at phase end per operator instruction.

## 2026-09-03 — MCP lifecycle publication shares the binding generation

Each exact owner/config/auth/environment cache entry owns one ordered lifecycle
publisher. Opaque random IDs and typed reasons preserve correlation without
serializing secrets or secret-derived configuration hashes. Initial snapshots
and bounded future-event subscriptions are atomic; lag is explicit and recovery
is a snapshot, not fabricated replay. Observers retain only a Weak publisher.

Reconnect callbacks attach only after binding validation and carry the cache
generation as well as the existing registry configuration generation. Shutdown
is terminal within that lifecycle generation. A concrete shutdown/late-install
interleaving could otherwise emit Ready after ShuttingDown. Reconnect admission
is single-flight per slot; cancellation changes only its own Connecting state.
The bool compatibility metric is written under the same publication lock, not
by a late independent reconnect write. Its server-only labels remain unchanged.

The async-patterns, agent-runtime-security and agui-event-contract skills drove
weak observer ownership, generation-bound publication and secret-free envelopes.
Uncomfortable limit: there is no RunManager subscriber yet (task4.1), no idle
transport health-monitor claim, and no behavioral evidence until phase-end tests.
Task3.3 T0 passed with zero warnings in32.23s,23.66s,11.95s. HTTP3.1 still awaits
the operator-owned reqwest_mcp pin; no dependency or versions.toml edits.

## 2026-09-03 — MCP discovery changes visibility, not descriptor authority

Keep original ToolDescriptor metadata intact and project a separate bounded
visible map. Mutating Deferred into Eager on the original descriptor would
break exact catalog/binding comparisons or confuse advertisement with grants.
The host caps each step at32 MCP tools, search selects at most8, and retained
selections are bounded/revalidated against the next authorized snapshot.
These are implementation defaults for the required bounded tool surface, not
new operator config or policy grants. Hidden tools never enter the search index.

Register search_tools only in a fresh chat-local native registry and only when
deferred tools exist. Run every model batch against its frozen visible map,
even if search modifies next-step selections earlier in the batch. This follows
agent-runtime-security's host boundary and async-patterns' explicit shared
state ownership; no lock spans await, no new task is spawned, no server starts.

Uncomfortable limit: the live orchestrator call site is implemented, but its
200-tool and concurrency behavior awaits phase-end tests. Final T0 was clean
in11.30s after removing one observed unused import. HTTP adapter and manager
binding-cache integration remain separate unfinished tasks; no scope reduction.

## 2026-09-03 — Prepared MCP execution and delegation preserve binding identity

Carry McpPreflight alongside the activation/step snapshot rather than route
projected tool names back through a mutable global registry. Host-governed calls
resolve exact projected descriptors; native tools retain their separate path.
Publish activation state only after preparation and collision checks finish.
Use that same descriptor snapshot for run outcome attribution.

At an explicit child handoff, wait for selected lazy bindings and freeze the
actual transport Arcs. Keep leases until capture and revocation checks finish;
retain only projected MCP tools and companion in-process tools. A child cannot
silently omit a previously prepared dependency or receive reconnect recipes.
Complete catalog discovery must also apply during freezing: first-page-only
validation would reject legitimate tools discovered by the paginated connector.

No new guard is speculative: these checks enforce the delegation trust boundary,
captured identity, descriptor collisions and observed discovery-page mismatch.
Uncomfortable limit: task4.1 is not complete. The new root constructor and child
handoff APIs have no root caller yet; graph/bootstrap/policy-universe/lifecycle/
shutdown integration remains. Latest T0 checks passed zero warnings in19.55s,
9.55s and7.35s; no behavioral acceptance is claimed before phase-end testing.

## 2026-09-03 — Preserve verified principal through MCP assembly

Root ingress must retain ActorOwner, not just user_id: the latter loses tenant
identity used by McpBindingCache. HTTP adapters and actor mailboxes now carry
the existing verified owner through RunExecutionRequest and ResolvedTurn;
prepared step assembly compares it with McpPreflight.owner. Anonymous and
legacy caller strings are not promoted into verified cache owners. This is the
runtime-security skill's existing tenant-binding trust boundary, not JWT parsing.

Uncomfortable limit: shared caching is still not enabled at the root, and older
embedded/interaction APIs still lack the stamp. These require explicit host
identity/isolation before migration; compilation is not tenant-isolation proof.
Four T0 passes were warning-free:18.47s,9.87s,8.14s,12.90s. Tests remain phase-end.

## 2026-09-03 — Root MCP resources enter RunManager without ambient fallback

Accept the host's immutable owner/catalog/environment plus its shared runtime
through RunExecutionRequest. When supplied, build the projected activation host
and preserve it across skill activations; required failures stop the run. Every
preflight observes the same run token. Do not shut down a shared runtime when
one run ends, and do not replace a failed captured binding with legacy config.

Uncomfortable limit: bootstrap still supplies no bundle; default execution is
not migrated. Captured graph requests fail explicitly until their governed
adapter is implemented. This temporary unsupported branch must be removed by
task4.1 completion, not accepted as a smaller scope. T0 clean34.12s and13.40s;
no behavior tests. The HTTP operator pin remains absent.

## 2026-09-03 — Actor shutdown retains join ownership until completion

Observed two gaps in the actual actor path: the server resource cleanup did not
call shutdown_all, and stop_actor removed its JoinHandle before awaiting it, so
an HTTP timeout could detach the actor from all later cleanup. Retain handles
in registry-owned Arcs, await by mutable reference under a per-handle mutex,
then remove only the matching actor. A cancelled waiter releases the mutex but
not the handle. Close admission with an actor-system child token before taking
the shutdown snapshot, and await mailboxes before shared MCP shutdown.

Uncomfortable limit: compile checks do not prove shutdown races or persistence
recovery. The server's existing hard deadline still applies. Actor collaboration
is not yet a child-thread adapter, so task 4.1 remains open. These changes follow
the actor-model, async-patterns and agent-runtime-security skills; no unrelated
feature or new timeout was added. T0 passed zero warnings: 17.42s, 25.05s, 15.73s.

## 2026-09-03 — Capture child kernel resources without recapturing authority

Keep root executable resources under a root-owned cancellation lease, with only
a weak reference in the run index. CapturedThreadKernel validates the committed
owner/root before freezing MCP, then enters the same manager kernel with inherited
clients, skills, native bindings, approvals and cwd. Model/client constructors and
ambient resource lookup are not substitutes for inherited bindings. Root completion
revokes the lease even while a borrower holds an Arc. The async/security skills
drive cancellation ownership and owner/resource validation at this boundary.

History is recorded per run as well as into the existing conversation: the latter
is mutable across runs and cannot identify an earlier child's canonical dialogue.
This costs an additional in-process dialogue snapshot; it does not add durable
recovery. ThreadService shutdown joins owned jobs before closing live child
records, retains failed join receipts, and reconciles uncertain writes only by
exact readback. It is not equivalent to dropping task handles.

Uncomfortable limit: this supplies the captured execution entry, not the missing
admission host or adapter attachment. No actor/graph/A2A caller uses it yet. Full
sandbox/budget enforcement and root scheduler ownership remain prerequisite to
enabling delegation. Initial compile mismatch fixed; six subsequent T0 checks
passed zero warnings. No tests or acceptance review before the phase boundary.

## 2026-09-03 — Root budget envelopes narrow without new accounting identities

Reuse CostBudgetTracker's captured root scope keys for child token/cost/tool/rate
accounting. Root entry time anchors narrowed deadlines; raw captured model
clients receive one budget wrapper only when used. This prevents fresh child
balances and duplicate billing from nested wrappers. ThreadService requires the
actual captured kernel and checks its budget itself, independently of adapter
sandbox admission. Approved and governance-bypassed tools share one allowance.
Finite cost limits reject unpriced models. Session cost is not Agent-wide spend:
the prior assignment could reject unrelated sessions of the same artifact.

Uncomfortable limit: known-usage admission is not a prepaid billing guarantee.
In-flight work can overshoot, missing provider usage remains unaccounted, and
local deadline expiry does not prove remote work stopped. No adapter attachment
exists yet; task5.1 stays open. Five warning-free T0 checks, no behavior tests.

## 2026-09-03 — Sandbox ownership and permission capture belong to the host

Keep remote create/execute/destroy in supervisor-owned jobs, not the model stream
future. Await one creation/destruction response and retain uncertainty instead of
replaying a mutation. Complete actor replies only after finalization; cancellation
does not prove remote cleanup. Save consumed JoinHandles before another await.
Capture execution configuration once and pass the exact binding to descendants;
opaque environment grants narrow captured values without host lookups. Reject
mount restrictions while the current string-map protocol lacks explicit access
semantics. No new remote wire format or read-only claim is introduced.

Uncomfortable limit: compile success is not cleanup/isolation evidence, and this
binding does not enforce permissions for direct native tools. Missing concrete
host/adapter attachment keeps task4.1 open. Final T0 zero warnings20.31s; behavioral
tests remain at the phase boundary as instructed by the operator.

## 2026-09-03 — Actor adapters reuse the admitted root service

Pass the host's committed root and persistence instance directly into assembly.
Include control-factory names in normal policy resolution, snapshot executable
resources before installing root-local handlers, then attach once before the
manifest/model launch. This avoids both policy widening and self-retaining
service/kernel/handler cycles. Capture validation preserves existing root tools;
the stricter adapter checks continue to gate delegated execution, not root capture.

Authenticated actor collaboration targets a child in the source's live root,
not a mailbox command that creates a new independent run. The explicit endpoint
request authorizes root delegation after Cedar; child resources still intersect
and child tool approvals still bubble to the root. Do not infer authorization
from message text or silently start a new root for an idle source actor.

Retain the actual actor producer JoinHandle as well as child jobs. Join by
reference and keep unknown cleanup receipts in the actor registry. A completion
message or finished mailbox is insufficient evidence that its tree is drained.

Uncomfortable limit: these are source/compile results, not runtime acceptance.
Native permission ports remain incomplete, task4.1 is open and phase-end tests
are deferred under the operator's implementation-first rule.

## 2026-09-03 — Native child tools retain their actual authority

Compiler sessions use (verified owner, host conversation, compiler session)
keys instead of trusting a model-supplied ID as a global namespace. Compiler
signing only inherits a captured in-memory provider with an explicit local
delegation contract; unknown providers default to denial. Legacy memory tools
now receive the same host context as NativeSkill implementations and bind every
verified call to the existing memory owner field. No new tenant storage grammar
or credential material is synthesized.

Web-fetch admission retains the configured public-web capability. Requests bind
to the addresses checked by the SSRF guard and disable ambient HTTP proxies so
an intermediary cannot silently resolve a different destination. Response size
is enforced while reading, using exact bytes. This is not a claim that sandbox
mount/environment policy controls host-native tools.

Uncomfortable limit: proxy-only deployments can stop working; deleted memory
history lacks a live ownership record; source/compile checks do not establish
runtime isolation or rollback. Other native permission ports remain open.

## 2026-09-03 — File byte bounds are separate from directory confinement

Implement exact file-size enforcement and a single patch read/write handle now,
but do not use these as evidence that path-prefix checks enforce confinement.
The remaining path traversal boundary needs captured directory capabilities.
cap-std4.0.2 is already locked transitively and its Dir API fits this boundary;
direct adoption awaits the operator's cap_std pin in versions.toml, required by
dependency-pin-discipline. No dependency manifest changed. Do not copy a large
OS-specific resolver or reach through another dependency to evade that gate.

Uncomfortable limit: the opened-handle patch prevents redirecting its final
write, not an escape during the initial open. External file writes and partial
I/O on cancellation are not made transactional. Task4.1 is not finished.

## 2026-09-03 — A direct terminal process needs host ownership, not just timeout

RunManager now retains a dedicated terminal process supervisor. Tool futures
request work; run-owned workers keep exact Child and JoinHandle ownership through
timeout/cancellation and report reaping failures without discarding receipts.
Leases cancel on unwind; normal, cancelled, and server shutdown paths join jobs.
Output capture drains both pipes while retaining bounded head/tail data.

Uncomfortable limit: this owns the shell process, not escaped descendants, and
does not confine host credentials/cwd/environment. Keep delegated direct-shell
execution denied until its physical permission contract exists. Standalone raw
tool compatibility is explicitly weaker and is not evidence of managed cleanup.
Task4.1 remains incomplete; runtime acceptance is deferred to phase end.

## 2026-09-03 — A2UI validation does not consume a host resource grant

The native A2UI permission port permits its existing declarative validator,
not surface mutation by an agent kernel. RunManager owns publication with the
actual execution run ID. The host still applies selected-tool policy, approval
and execution-mode checks. This backend-only change does not redesign UI or
authorize actions described in a surface. No visual-design pass applies.

Uncomfortable limit: the existing result truncation/publication path remains
unchanged and runtime delivery is unverified until phase-end tests. Do not use
this pure-data tool's contract to justify direct file, shell or network grants.

## 2026-09-03 — A graph invocation consumes its first child result

Graph AgentNode now uses the host's child controls and actual approval/budget
gate rather than pretending a two-message LLM request is a specialist agent.
The built-in graph target names now resolve real artifacts. Route identity
stays in graph/thread metadata, not assistant text. Child history is explicitly
selected and defaults to none.

An ordinary latest-state watch is insufficient: a queued follow-up can replace
the first result before the graph waiter is scheduled. ThreadService therefore
retains one first-terminal receipt per entry in addition to its existing latest
watch. The independent critic identified a later-pending-write check that still
hid the receipt; fixed it while preserving caller/parent authorization.

Uncomfortable limit: ordinary graph root attachment is missing; actor/inherited
bindings are the only supplied delegates. This is partial4.2, not acceptance.
The runtime behavior and integration-test migration remain phase-end work.

## 2026-09-03 — Ordinary graphs share persisted root ownership, not mailbox lifetime

Supersedes the missing ordinary graph root caller above. Reuse ActorThreadSession
for the complete verified request and retain it in GraphRootSupervisor. The HTTP
preparation future observes readiness; it does not own the worker or its writes.
Its cancellation requests cleanup. The internal completion observer must not
defeat last-viewer cancellation, unlike a real independently waiting mailbox.

Graph terminal status now agrees with terminal events; a cleanup failure overrides
successful execution. Server cleanup retains failures and propagates them instead
of emitting graceful success. These changes address the source critic's concrete
findings. Exact uncertain writes remain retained, never blindly replayed.

Uncomfortable limit: compilation and isolated source review are not behavioral
acceptance. Captured-MCP graph dispatch still fails closed, remote A2A is pending,
and tests remain deferred to the phase boundary.

## 2026-09-03 — Governed outbound A2A children require trusted UAR peers

Operator decision: the prevalent deployment is UAR instances delegating to other
UAR instances. Count a remote A2A task as a governed child only when its endpoint
is operator-configured and authenticated and the peer explicitly acknowledges a
compatible UAR enforcement contract for exact owner/root/parent/child identity,
the intersected policy and root budget, usage receipts, and cancellation. Missing
or mismatched acknowledgement fails closed. Keep standard A2A JSON bodies stable;
carry the UAR contract through authenticated transport metadata and exact receipts.

Arbitrary A2A endpoints are not governed children in this change. Supporting them
later requires a separate external/unmanaged capability and explicit policy because
local tracking cannot prove remote tool, credential, sandbox, budget, usage, or
cancellation enforcement. A trusted-peer acknowledgement is still a contractual
boundary, not cryptographic remote attestation; do not describe it as attestation.

## 2026-09-04 — Delegated direct files use captured directory capabilities

Trusted UAR peers may delegate `file_read`, `file_write`, and `file_patch` only
when the exact native tool is inherited and the host configured at least one
bounded directory root. Convert those roots to open directory handles before a
child request exists and perform all child path operations relative to them.
Reject wildcard, empty, filesystem-root, relative, unavailable, or unsupported
root authority. Keep ordinary root-run compatibility on its existing allowlist
path.

This matches the prevalent UAR-to-UAR use case without treating pathname prefix
checks as physical confinement. Direct child shell execution stays denied: a
supervisor that owns a process lifetime does not confine its filesystem,
environment, credentials, or escaped descendants. The uncomfortable cost is
that permissive `*`/filesystem-root configurations work for root runs but cannot
be delegated, and unsupported directory-identity platforms fail closed.

## 2026-09-04 — Peer-first projected MCP uses verified host binding identity

The prevalent topology is UAR-to-UAR. A verified root therefore captures the
target host's immutable MCP definitions and enters one shared host-owned cache
keyed by full subject/tenant owner, config hash, opaque per-boot binding revision,
and resolved environment. Wire `user_id` is metadata, never authority. Anonymous
or identity-stripped adapters cannot acquire the projected cache.

Treat Unknown and Required authentication as non-executable. Treat host-captured
definitions as Authenticated with a redacted revision identity; actual secrets
remain in host-only snapshots and never enter errors or events. Administrative
definition changes revoke every cached owner revision before the new catalog can
be used.

All/Auto stays open to tools discovered from already-selected frozen server
definitions only when no higher scope narrowed or denied the resource family.
Once closed, the effective selection becomes finite Selected/None so lower scopes
cannot reopen it. Stdio skill processes receive explicit configured environment
keys plus minimal launch variables, not the complete UAR process environment.

## 2026-09-04 — Delegated MCP authority is frozen locally and peer-local remotely

Projected MCP catalogs contain only host-global and skill-contributed server
definitions. Local agent children inherit narrowed frozen bindings and cannot
reconstruct or reconnect MCP servers. Authenticated remote UAR children execute
as roots on the peer and resolve the peer's own definitions and credential
bindings. The unused child-definition provenance was removed because it implied
a connection-recipe transfer path that the trusted thread kernel deliberately
does not provide. This decision favors the prevalent UAR-to-UAR deployment while
keeping credentials and launch configuration within the host that owns them.

## 2026-09-04 — Name the live cancellation point instead of overstating evidence

Task 6.3 requires a local multi-agent cancellation smoke with a real model; it
does not require child text before cancellation. An independent artifact-only
review confirmed that cancelling a persisted child's real outbound request
while it awaits its first provider response meets that criterion. The passing
k3 sidecar run is recorded in thread-native-subagents/evidence/live-cancellation-report.json.
The original after-text scenario remains available and unverified, not silently
replaced or reported as passing. Real provider timeouts and 500/502 responses
prevented it from reaching emitted child text. A closed local request does not
prove the provider stopped computation or billing. Empty shadow reports cannot
serve as evidence for the typed-default flip.

## 2026-09-04 — Typed default evidence gate met with explicitly narrow coverage

The copied typed-turn-assembly parity report covers three cases: basic user
turn, host instructions, and memory contribution. It has zero unexpected or
allowlisted differences, verified by the completed phase-end corpus test.
The live command `UAR_SMOKE_MODEL=k3 UAR_SMOKE_LOG=info node
openspec/changes/typed-turn-default-flip/evidence/live-shadow.mjs
target/debug/uar-sidecar` exited 0. Two independent default-agent runs (basic
input and host instructions) each emitted real text, completed, dispatched legacy,
and emitted one shadow comparison with zero differences. The command, output,
and run IDs are in the change's evidence directory. This satisfies the recorded
pre-flip gate; full Tier 2 verification with the new default is still required.

The uncomfortable thing: two live cases on k3 and a three-case corpus are weak
coverage for the complete runtime. Live memory, MCP, active skills, multi-step
tools, remote peers, and other providers are not established by this receipt.
Keep `harness.mode: legacy` as an explicit rollback for one minor release;
shadow remains opt-in and performs duplicate assembly, not duplicate inference.

## 2026-09-04 — Deferred provider observation is not an implementation gap

Model-path-resiliency's approved proposal explicitly defers the real-provider
429 smoke. Its spec requires retry behavior, covered by completed local tests,
but adds no live-provider observation gate. Independent artifact review confirmed
that implementation may be complete while task5.4 stays unchecked and evidence
stays outstanding. Correct the canonical change status without inventing a21/21
task pass. The exact live reproduction command and provider observation remain
unverified; no request flooding or synthetic429 is authorized as a substitute.

## 2026-09-04 — Original-scope phase audit corrections before closure

Plan revision 6 records five observed source defects across four original changes.
Preserve the earlier passing suite and live receipts, but do not treat them as
coverage of the missed paths. Build the full correction batch before authoring
and executing its integration regressions; keep archive/phase reflection pending.

Remote capacity is released only on host-proven non-dispatch or confirmed peer
cleanup. Governed parallel reads retain exactly-once host budget admission and
per-call approval/execution ordering. Catalog minimum lines retain titles and
suggestions. Provider metadata does not end the pre-semantic retry boundary;
partial semantic output still interrupts rather than replaying inference.

Primary chat reconnect reuses the existing x-uar-run-id as an explicit request
identity paired with Last-Event-ID, verifies exact subject/tenant ownership, and
uses format-tagged event/frame cursors. It never guesses a session's latest run.
Projection-prefix eviction returns 410 because no retained projection checkpoint
exists. That bounded limitation is explicit; a fresh run is not recovery.

## 2026-09-04 — Presentation domain confirmed as reusable UI templates

The operator chose reusable UI templates, separate from the development-only
A2UI tester. The active parent phase now has goals, three OpenSpec deltas and
a reviewed implementation plan. Trusted-host persistence owns identity,
tenant/subject partition and revisions; template content remains declarative.
Admitted runs will capture complete validated content and revision so a later
edit, disable or delete cannot relabel new content with old provenance.
Text-only negotiation must govern every surface-publication ingress, including
host policy artifacts and direct surface submission, not just native tools.

Impeccable was updated locally from 4.1.1 to 4.2.0 at the operator's request,
without installing hooks. The existing admin identity remains authoritative.
UI/UX Pro Max's landing-page/glass recommendations were rejected as off-target;
its accessibility and error-recovery guidance applies. No no-code builder,
marketplace, arbitrary executable components or restored release gates enter
this phase. Runtime tests remain at the end of the implemented phase.
