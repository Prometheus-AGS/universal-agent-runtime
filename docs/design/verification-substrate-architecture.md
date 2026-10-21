# Verification Substrate Architecture

**Status:** analysis and design only — no code changed.
**Scope:** universal-agent-runtime test estate, certification scripts, and the
production chokepoints those tests guard.
**Method:** read-only audit of the five problem categories (14–18) against the
current tree (HEAD `98b50f6e`, dirty). Every area named below was opened and
read; line references are to the working tree as of 2026-10-20.

---

## 1. The finding in one paragraph

Categories 14–18 look like five unrelated failure lists. They are one failure
with five faces: **the verification substrate is duplicated, and the weakest
copy defines the estate's real guarantee.** The repository already contains a
best-in-class implementation of every required discipline — process isolation
in `tests/support/sidecar_process.rs`, sentinel positive controls in
`tests/sidecar_launch_security/confinement.rs`, bounded child stacks in
`tests/truthful_tool_outcomes.rs`, candidate-bound append-only receipts in
`scripts/dot/acceptance.py`. Each failure in the audit table occurred exactly
where a *second, weaker copy* of the same mechanism existed, or where no copy
existed at all. The architectural answer is therefore convergence, not
invention: one harness, one invariant registry, one fixture preflight
contract, one evidence ledger — each formed by promoting an implementation
that already exists and deleting or absorbing its weaker siblings.

---

## 2. Where each problem type lives

### Category 14 — Security confinement and policy enforcement

**Test side**

| Area | Role |
|---|---|
| `tests/sidecar_launch_security.rs` + `tests/sidecar_launch_security/{token,guard,surface,governance,features,capabilities,confinement}.rs` | ~28 contract tests across the 7 modules. This is the suite the "eight failing sidecar tests" belong to. |
| `tests/support/sidecar_process.rs` (899 lines) | The process harness: private `HOME`/`work`/`data`/`logs`/`no-skills` per launch, `env_clear`, stdin token handoff, `READY:` handshake, `finish_fixture` teardown discipline. |
| `tests/mcp_child_environment.rs` | Child-process environment/stdin rules; re-runs its own test binary as a child with a controlled environment. |
| `src/bin/uar-env-probe.rs` (feature `test-probes`) | The canary binary that records what a tool/MCP child actually sees. |

**Production side (the chokepoints under test)**

| Area | Role |
|---|---|
| `src/bin/uar-sidecar.rs` (482 lines) + `src/bin/uar-sidecar/{inherited_input,control}.rs` | Launch-token read from first stdin line (bounded, zeroized), single `READY:` line, stdin-EOF shutdown contract, forced-off session-mixing features. |
| `src/uar/security/sidecar_guard.rs` (451 lines) | `SidecarLaunchToken`, `SidecarBoundPrincipal` — the only types that may hold launch authority. |
| `src/uar/security/middleware.rs` (788 lines) | Per-request bearer/Host/Origin enforcement. |
| `src/uar/settings/manager.rs` | The only producer of `sidecar_mode_global_mcp_disabled`; governance/settings mutation rejection. |
| `policies/{default,tool-approval,skill-mutation}.cedar` | Governance policy files copied into every fixture workspace by the harness. |

**Notable gap pattern:** the token-absence-from-argv/environment assertion
(`confinement.rs:74`) was left unfinished longest because it is the only test
that must inspect *other* processes (via `sysinfo`); every discipline it needs
(sentinel positive control at lines 99–112, probe-release handshake at
120–141) had to be invented for it alone. When a category of assertion
requires unique machinery, it ships last and fails first.

### Category 15 — Ownership, grants and cross-agent isolation

| Area | Role |
|---|---|
| `tests/dot_memory_owner_contract.rs` | Embedded-memory owner boundary: two `MemoryService` runtimes, distinct issuers/tenants, sentinel markers, native tool owner-binding, MCP session replay rejection, legacy-import quarantine. Gated `#![cfg(all(feature = "server", feature = "surreal-backend"))]`. |
| `tests/dot_owner_adjacent_contract.rs` | Physical-key collision and adjacent-owner partition for sessions, KB, documents, chunks, cost history. |
| `tests/bauar_resource_grants.rs` + `tests/support/bauar_resource_peer.rs` | Grant pinning: owner/run/destination/revision/lease/action; independent receiver with positive and negative credential controls; effect-count assertions after every denied operation. |
| `tests/remote_mcp_run_grants.rs` (Gate R, 720 lines) | Two principals (`boss-session-1/2`) through a real sidecar to a remote MCP peer; the "two-agent/no-leak" evidence. |
| `scripts/dot/owner_isolation_journey.py` + `scripts/dot/journeys.py` (`memory()`) | Installed-host HTTP-layer isolation journeys. |
| `src/uar/security/delegation_grants.rs` (305 lines) | The production grant type the tests pin. |
| `src/uar/security/claims.rs` (`session_owner_key`), `src/uar/runtime/actor/messages.rs` (`ActorOwner::from_verified_context`) | The only legitimate owner-key derivations. |

**Notable gap pattern:** the shared-store tests
(`shared_surreal_two_runtime_owner_boundary`,
`shared_durable_adjacent_owner_contract`) require an externally provisioned
`UAR_DOT_MEMORY_SURREAL_URL`; when that environment is absent the *embedded*
variant carries the whole category, and any failure in it leaves the
cross-runtime evidence incomplete — exactly the "R2 two-agent/no-leak evidence
was incomplete" symptom. Isolation evidence is only as strong as its weakest
profile, and profile availability is currently implicit (env var present or
not) rather than declared.

### Category 16 — Native-code/test compatibility and stability

| Area | Role |
|---|---|
| `tests/context_history_integrity.rs` (2,253 lines) | The suite whose stack-overflow/SIGABRT followed earlier passes (session-log receipt `20261012T013104Z`: libtest top line read "1 passed/1 failed" while the real child assertion had already aborted — see Category 18 for why that reporting shape is itself a defect). |
| `tests/truthful_tool_outcomes.rs:911` `child_outcome_boundary` | The reproduction of that crash and the fix pattern: a named child thread with an explicit 8 MiB stack (lines 916–928) matching the shipped runtime's bound, parent-controlled cancellation, `tokio::select!` supervision. |
| `Cargo.toml` | `[[bin]]` `required-features` declarations (with the comment explaining that Cargo auto-discovers `src/bin/*.rs` and only explicit declarations gate them), `test-probes` feature gating `uar-env-probe`, `bauar-native-admission-gate` as a separate acceptance-only artifact, `[[test]] required-features` (e.g. `sidecar_launch_security` needs `server,test-probes`). |
| `rust-toolchain.toml`, `versions.toml` | Toolchain and dependency pins. |

**Notable gap pattern:** feature gating is declared per-target in the
manifest, but nothing consumes the union of those declarations as one matrix.
"Some targets lacked required feature flags" and "stale constructors caused
historical compilation failures" are the same defect at different times: the
compile surface of the test estate is not itself a verified artifact.

### Category 17 — Fixture validity and isolation

**The duplicated harnesses (the core finding):**

| File | Isolation rigor |
|---|---|
| `tests/support/sidecar_process.rs` | Full: `env_clear`, private HOME, `no-skills` dir, private policy copy, READY handshake, `files_containing` byte search. |
| `tests/support/api_key_process.rs` | Own `free_port` (line 313), own env handling; unreachable-provider fixture hardcodes `http://127.0.0.1:9` (line 156). |
| `tests/support/credential_process.rs` | Own port allocation (line 109), weak readiness: polls `/health` with `reqwest::get` (line 171) instead of a bound handshake. |
| `tests/support/approval_recovery_process.rs`, `fresh_process_resume_process.rs` (40 KB), `truthful_tool_outcome_process.rs` (**70 KB**) | Each re-implements slices of the same spawn/capture/teardown logic with different rigor. |
| `tests/support/uar_1_1_1_{memory,runtime}_fixture.rs` | A seventh and eighth parallel fixture family. |

**Other fixture-validity areas:**

- `tests/fixtures/*.py`, `*.mjs` — wire peers (MCP, A2A, schedules); route
  shapes and casing are defined independently in each peer.
- `tests/integration/live/stub_llm.rs` — the stub LLM; fingerprint matching
  (`RequestFingerprint`) is the request-shape validator.
- `config.test.yaml` vs `test-config.yaml` — two coexisting test configs with
  different JWT secrets (`test_secret_key_must_be_at_least_32_chars_long` in
  one); the "inconsistent passwords" class of contamination.
- `docker-compose.test.yaml` — shared infrastructure with fixed service
  identity; stale-port risk profile differs from ephemeral-port fixtures.
- `tests/context_history_integrity.rs` — synthetic budget fixtures
  (`RequestBudgetContract::synthetic_exact("http://fixture.invalid/v1")`),
  the "invalid budget fixture" class.
- Committed root `mcp.json` — pinned empty and asserted empty by
  `bauar_resource_grants.rs:277`; the "inherited environment/HOME skills"
  control that already works and should be generalized.

**Notable gap pattern:** isolation is *per-harness*, not *per-test*. A test
inheriting the weak harness gets weak isolation with no signal; the strongest
harness cannot lend its rigor to suites that don't `#[path]`-include it.

### Category 18 — Result integrity and candidate identity

| Area | Role |
|---|---|
| `scripts/dot/acceptance.py` (871 lines) | **The exemplar.** `source_identity()` binds `git_head` + `dirty_patch_sha256` + untracked-file hashes + recursive submodule identity + `content_sha256`; `require(digest(binary) == spec['binary_sha256'])`; `Evidence.save()` writes receipts with mode `x` (no overwrite), `0o600`, `fsync`; statuses distinguish passed/failed/blocked/interrupted. |
| `scripts/release/uar_1_1_1_acceptance.py` | Exact-count requirement (`"test result: ok. 1 passed; 0 failed;"` must appear, line 235); repair reruns require a hashed parent receipt showing the group failed/blocked/interrupted (line 271); exit codes 0/1/2/130 map to passed/failed/blocked/interrupted. |
| `scripts/certify-operational-resilience.sh` | Correct `PIPESTATUS` capture around `cargo test … | tee`; `results.json` carries `source_sha` and raw `exit_code`. |
| `scripts/certify-release-candidate.sh`, `prepare-release-evidence-local.sh`, `package-candidate-certification-local.sh`, `security-audit-local.sh` | All `set -euo pipefail`; but each writes its own result envelope shape — there is no shared receipt library across shell and Python. |
| `scripts/validate-*.mjs` (23 files), `scripts/ci-grep-gates.sh` | Validator fleet; each emits its own pass/fail text. |
| `.prometheus/session-log.md:4511` | The documented instance of every category-18 failure mode at once: pipeline-masked failure, libtest top-line undercount ("1 passed/1 failed" while the child had aborted), teardown hidden behind a passing no-op top entry, evidence subsequently bound to the *integration executable's* SHA256 — proving the estate already knows how to recover, but only after the fact. |

**Notable gap pattern:** receipt discipline exists in exactly one
(Python, Dot-phase) implementation. Shell gates and the `.mjs` validators
re-implement fragments. Receipts from different binaries become conflatable
precisely at the seams where no shared library enforces identity.

---

## 3. Cross-cutting root causes

1. **N copies of the harness, 1 of them good.** Six to eight process-fixture
   families duplicate `env_clear`/`free_port`/config-render/teardown. Every
   Category 14 and 17 failure traces to a test running on a weaker copy.
2. **Assertions without identity.** Pass/fail is not structurally bound to
   commit + dirty-tree + binary hash + fixture identity, so evidence from
   different candidates can be — and historically was — combined.
3. **Fixtures graded without preflight.** Only the newest tests
   (`confinement.rs`, `bauar_resource_grants.rs`) prove their fixture *could*
   catch the defect before asserting the defect is absent (sentinel and
   receiver positive controls). Everywhere else a vacuous pass is possible.
4. **One exit code for setup, body and teardown.** `finish_fixture` fixes
   this in exactly one harness; everywhere else teardown failure can hide
   behind body success.
5. **The compile surface of the test estate is unverified.** Feature flags
   are declared per target but never consumed as one matrix; stale
   constructors and missing flags are found by failing CI, not by a gate.

---

## 4. The architecture: one substrate, four contracts

No rewrites. Each contract *is* an existing file, promoted to be the only
implementation of its concern. The work is deletion, extraction, and
mandation.

```
                 ┌────────────────────────────────────────────┐
                 │            INVARIANT REGISTRY              │
                 │  generated from test names; receipts hash  │
                 │  it; no assertion exists off-registry      │
                 └───────┬──────────────────────┬─────────────┘
                         │                      │
        ┌────────────────▼───────┐   ┌──────────▼─────────────┐
        │   HARNESS CONTRACT     │   │   EVIDENCE CONTRACT    │
        │  one process fixture   │   │  one receipt library;  │
        │  (sidecar_process.rs,  │   │  candidate identity +  │
        │  generalized)          │   │  phase-separated exit  │
        └────────────────┬───────┘   └──────────▲─────────────┘
                         │                      │
                 ┌───────▼──────────────────────┴────────────┐
                 │          FIXTURE PREFLIGHT CONTRACT       │
                 │  declared ports/routes/credentials;       │
                 │  positive control before grading          │
                 └───────────────────────────────────────────┘
```

### Contract 1 — One harness (fixes 14, 16, 17)

Promote `tests/support/sidecar_process.rs` to the single process-fixture
implementation for the estate. It already is the superset: private
HOME/work/data/logs, `env_clear` with an explicit platform allowlist, private
policy copies, stdin token handoff (including the Windows named-pipe branch),
READY-line handshake, panic-preserving `finish_fixture`, byte-search
utilities, raw-HTTP client, route inventory from the served OpenAPI document.

- Absorb the six sibling harnesses (`api_key_process`, `credential_process`,
  `approval_recovery_process`, `fresh_process_resume_process`,
  `truthful_tool_outcome_process`, `uar_1_1_1_*_fixture`) into it: each keeps
  only what is genuinely unique (its peer servers, its config knobs) as data,
  never as spawn/isolation/teardown code.
- Keep the existing `#[path = "support/…"]` inclusion mechanism — it is how
  integration targets share fixtures without a crate boundary — but there
  must be exactly one file behind the path for any given concern.
- Fold the bounded-stack child pattern from
  `truthful_tool_outcomes.rs:916–928` into the harness as
  `Harness::spawn_bounded_child`, so every future child-process fixture gets
  the SIGABRT fix by construction, and a child crash is a first-class,
  diagnostics-preserving result rather than an abort that corrupts the
  libtest summary.

**Why it is elegant:** the strongest isolation stops being opt-in. A test
cannot silently run on a weak harness because there is only one harness.

### Contract 2 — Invariant registry (fixes 14, 15)

Each security/ownership assertion gets a stable name, and the name is the
contract. The registry is *generated from the test suite* (test name → module
→ named invariant → the frozen requirement text in the OpenSpec change), never
maintained as a parallel document that can drift.

- Every Category 14 assertion maps to a named invariant
  (`token.argv_absent`, `token.env_absent`, `logs.no_prompt_canary`,
  `mcp.global_mutation_rejected`, `governance.normalized_required`,
  `capabilities.auth_required`, `denied_tool.no_side_effect`, …). The eight
  failing sidecar tests already have this shape; the registry makes the
  mapping explicit so none can be waived, renamed, or weakened to make
  another suite green.
- Every denied-action assertion must pair with a *no-side-effect* probe
  (the pattern already in `bauar_resource_grants.rs`: `peer.effects().len()`
  is re-asserted after every rejection) and every protected-route assertion
  with an unauthorized-caller rejection. The registry requires the pair; an
  unpaired invariant fails the registry check itself.
- Ownership invariants require distinct owners *and* distinct sentinel data
  (`dot_memory_owner_contract.rs`'s marker pattern) — a shared database
  connection is explicitly not accepted as isolation evidence.

**Why it is effective:** "never waive these expectations" stops being a prose
rule in a requirements table and becomes a mechanical property: the receipt
(Contract 4) hashes the registry, so a waived or weakened invariant changes
the hash and invalidates the evidence produced under it.

### Contract 3 — Fixture preflight (fixes 17, and the vacuous-pass half of 14/15)

A fixture declares its needs; the harness validates them before any grading
assertion runs:

- **Ports:** ephemeral only (`:0`); a declared port that arrives as a literal
  is a preflight failure, not a test failure.
- **Readiness:** the bound-handshake pattern (`READY:` line, reported-port
  equality check) replaces `/health` polling everywhere; polling an endpoint
  that would also pass on the wrong server is rejected.
- **Positive control:** every absence assertion must first demonstrate the
  measurement channel works — the sentinel-environ check in
  `confinement.rs:99–112` and the receiver-credential check in
  `bauar_resource_grants.rs:44–61` are the two existing instances; preflight
  makes the pattern mandatory.
- **Isolation:** HOME, skills directories, stores and `mcp.json` are
  harness-owned per launch (already true in `sidecar_process.rs`); the
  committed-empty root `mcp.json` assertion generalizes to a rule that no
  fixture may read operator-global state.
- **Frozen requirement:** the requirement text (OpenSpec scenario) is hashed
  into the fixture declaration; a fixture repair that weakens what is tested
  changes the hash and fails preflight review — this directly answers
  "verify repaired fixtures still test the frozen requirement."
- **Config singularity:** `config.test.yaml` and `test-config.yaml` converge
  to one generated artifact from `render_config`, ending the two-secret
  ambiguity by construction.

### Contract 4 — Evidence ledger (fixes 18, and the reporting half of 16)

Extract `scripts/dot/acceptance.py`'s identity and receipt machinery into one
shared library (importable from Python, callable as a subprocess from shell
and Node), and mandate it for every gate in `scripts/`:

- **Candidate identity:** every receipt binds `git_head`,
  `dirty_patch_sha256`, untracked-file hashes, submodule recursion,
  `binary_sha256` of the artifact under test, operating mode, configuration
  digest, and fixture-tree hash — all already implemented in
  `source_identity()`; the change is that *no gate may emit results outside
  this envelope*.
- **Phase-separated exit codes:** setup, body, and teardown each preserve
  their own raw log and true exit status (the shell gates already do
  `PIPESTATUS` correctly; `finish_fixture` does it in Rust). A teardown
  failure can no longer hide behind a passing body because the receipt has
  three status fields, not one.
- **Independent count reconciliation:** the runner counts `test result:`
  lines *and* parses them, requiring exact expected counts
  (`uar_1_1_1_acceptance.py:235` already does); the libtest
  top-line-undercount failure mode (session-log 4511) becomes impossible
  because a child-process crash is a Harness-level event (Contract 1) with
  its own receipt field, not a line in a stream a counter can miss.
- **No cross-candidate combination:** a repair rerun must name the hashed
  parent receipt showing the group previously failed (already enforced at
  `uar_1_1_1_acceptance.py:271`); the ledger refuses to merge passes whose
  candidate identities differ.

### Compile-surface gate (fixes 16, supporting all)

One `test-matrix` manifest — generated from `Cargo.toml`'s
`[[test]]`/`[[bin]]` `required-features` declarations, consumed by one runner
— makes the union of (target × declared features) a verified artifact:
"compile applicable targets with their declared features" becomes a single
declared command instead of tribal knowledge. Stale constructors then fail at
the matrix gate, in one place, against the chosen candidate, before any
behavioral run.

---

## 5. What this deliberately does not do

- **No new frameworks.** Every mechanism above is a promotion of code that
  already exists and is already trusted in its home suite.
- **No production-code rewrite.** The production chokepoints
  (`sidecar_guard.rs`, `middleware.rs`, `delegation_grants.rs`,
  `settings/manager.rs`) stay exactly as they are; the eight failing tests
  are unclassified causes, and this architecture's job is to make each
  failure reproducible, diagnosable, and impossible to mask — not to presume
  the fix.
- **No weakening of any frozen expectation to buy green.** Contract 2 exists
  precisely to make that detectable.

**The uncomfortable limit:** a registry can itself be edited. The mitigation
is structural — registry entries are generated from test names and hashed
into receipts — but a determined editor can change test, registry, and
receipt in one commit. This substrate makes integrity violations *visible in
one diff*, it cannot make them impossible; the human review gate that this
repository already mandates at phase boundaries remains the final check, and
this design's real contribution is that it gives that reviewer exactly one
place to look.

---

## 6. Convergence map (existing → promoted)

| Existing implementation | Becomes |
|---|---|
| `tests/support/sidecar_process.rs` | Contract 1: the single process harness |
| `tests/truthful_tool_outcomes.rs:911–928` | Contract 1: `spawn_bounded_child` |
| `confinement.rs` sentinel control, `bauar_resource_grants.rs` receiver control | Contract 3: mandatory positive controls |
| `sidecar_launch_security/*` test names + OpenSpec scenarios | Contract 2: the invariant registry |
| `scripts/dot/acceptance.py` (`source_identity`, `Evidence`, `digest`) | Contract 4: the shared evidence ledger |
| `uar_1_1_1_acceptance.py` exact-count + repair-receipt rules | Contract 4: count reconciliation and rerun binding |
| `Cargo.toml` `required-features` declarations | Compile-surface gate: the test matrix |
