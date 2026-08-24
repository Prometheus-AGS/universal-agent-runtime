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
