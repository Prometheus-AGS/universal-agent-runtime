# Independent BDD catalog design review — 2026-10-27

Artifact-only review of `bdd-validation-catalog.md`, read against `behavior-status.md`, `coverage-audit-2026-10-27.md`, the capability/interface source inventories, `scripts/acceptance/fixtures/scenarios.json`, and relevant production policy/memory/route source. No generation history was supplied. Phase: `uar-verification-simplification`, documentation review only. Verification gate: **CLOSED**.

## Findings

### [MEDIUM] Resolve named fixture operations before calling these concrete scenarios

**File:** `.prometheus/verification/bdd-validation-catalog.md:311`

> “capture one private fact, opt out, send a second fact, restart and advance the supported retention fixture.”

E03 requires a short retention fixture and clock/retention control, but names no existing operation. Production `src/uar/memory/controls.rs:17` accepts only 7/30/90/365 days; `controls_store.rs:83–87` explicitly uses the database clock. A process-clock advance therefore does not establish expiry. C52 has the same unresolved-operation problem at catalog line 264: “maintenance operation must be located in production source before script authoring.” The old U21 source (`scripts/acceptance/cases_agents.py:210–229`) used an observer to age a persisted session and observe the real run-retention worker; it does not establish an exposed maintenance endpoint.

**Correction:** name the actual externally prepared aged-record/control fixture and real worker/operation, with its observable result, or mark that portion design-pending/deferred. Preserve consent/startup coverage without inventing a time-control API or rebuilding the retired observer engine. An unavailable external prerequisite may block a bounded case; an unidentified operation still needs design work.

### [MEDIUM] Make skill-policy predicates explicit and account for selected/none

**File:** `.prometheus/verification/bdd-validation-catalog.md:510`

> “the effective eligible set follows inherit semantics”

E25–E27 repeat this pattern for inherit/all/auto without declaring the actual parent set, child request, expected IDs and effective mode. Production `SelectionMode` includes **inherit, auto, all, none, selected** (`src/uar/domain/policy.rs:82–94`). Its resolver intersects selected IDs and prevents lower scopes from widening parent restrictions (`:844–881`). C42 disables one scope but does not explicitly exercise selected-set intersection or a lower-scope widening attempt; no other row names those predicates. Thus the full supported-options description can complete without observing this useful policy boundary.

**Correction:** give the policy rows concrete symbolic sets and expected sets/modes. Include a bounded selected/none narrowing and attempted-widening sequence in an existing governance journey, or explicitly defer those contracts and narrow the full-options claim. A five-mode × scope × activation matrix is unnecessary. Revise the denominator if introducing independently run scenarios.

### [MEDIUM] Require an explicit stopping budget for threshold and asynchronous cases

**File:** `.prometheus/verification/bdd-validation-catalog.md:292`

> “a history long enough for short/mid/long tiers”

E01/E02 and C43/C49/C54 depend on crossing a threshold or observing asynchronous progress. The authoring rule at line 709 asks only for an “intended small inference/output allowance”; it does not require a maximum number of turns/model calls, a deadline, or a terminal outcome when the trigger never arrives. Finite scenario IDs alone do not prevent a script from generating history or polling indefinitely. The existing stop-at-first-result and no-retry rules do not cover waiting inside one scenario.

**Correction:** require each authored scenario config to declare a fixed input/history size, maximum model calls and deadline for each wait before execution approval. State the bounded FAIL/BLOCKED distinction when the trigger is not reached. Reuse thin Node timeouts and explicit fixture data; no new global scheduler or registry is needed.

## Counts and scope assessed by source inspection

- `54 + 28 + 20 = 102` definitions; `102 − 32 = 70` planned; `54 − 32 = 22` unfinished common workflows.
- Historical credit correctly preserves 35 B-labels across 32 outcomes, including the B05–B08 grouping and B03 composite. It does not claim 32 clean feature passes or certify the current checkout.
- The legacy mapping retains all 97 manifest IDs with a disposition; it does not relabel the audited 28 partial/69 uncredited IDs as accepted.
- The stateful parity ranges contain `7 + 39 + 3 + 22 = 71` definitions. Adapter witnesses and platform applicability remain additional execution scope; 102 is correctly not presented as an all-platform execution total.
- The catalog preserves accepted Mac outcomes, external identity with no test credentials, genuine positive inference, operator-selected single scenarios, Mac → Linux → applicable embedded iOS, separate PostgreSQL evidence, and distinct simulator/device claims.
- Source-backed alias deduplication is appropriate; distinct shipped SDKs and wire serializers are not interchangeable. The catalog explicitly preserves residual/deferred scope rather than claiming every designed function is validated.

These are document/source conclusions, not product acceptance. No tests, checks, builds, dry-runs, imports, services, provider/API calls or publication occurred. Commands were source inspection only (`cat`, `sed`, `nl`, `rg`, scoped `git diff`); the sole write is this requested review artifact. No production code or guards were added. Runtime behavior, fixture availability and cross-platform execution remain unverified.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 3 | info |
| LOW | 0 | pass |

Verdict: **APPROVE as a proposed draft**. Resolve the three design gaps before describing every planned row as concretely runnable or adopting full supported-options coverage. This review does not open the verification gate.

## Final disposition after catalog revision — 2026-10-27

Re-read the revised C33/C42/C52, E03/E09/E25–E28, backend limitation and finite-ceiling sections against current policy, memory-control and retention-owner source. This is a re-review of the findings above, not a new product verification or expanded review scope. The original findings remain above as history; their current dispositions follow.

1. **Fixture-operation finding: CLOSED for the design artifact.** E03 now states at catalog line 311: “PUT the real 7-day policy so the old record's expiry becomes first_capture+7d.” Its prerequisite names a consistently provisioned eight-day-old automatic memory/control pair under an initial 30-day policy, with a fresh control. This follows real shortening at `src/uar/memory/controls_store.rs:56–77`, database-clock visibility at `:80–109`, purge at `:248–294`, and startup purge at `:21–32`. C52 now names only an owned timestamp-aging fixture followed by production activity and the real run-retention worker, rather than an unidentified maintenance endpoint (catalog `:261–265`; worker `src/uar/runtime/manager.rs:1390–1452`). Both explicitly return BLOCKED when the external fixture is unavailable. Fixture availability remains unverified; the catalog no longer treats inventing a clock/control API as part of authoring a scenario.

2. **Skill-policy finding: CLOSED.** C42 at catalog lines 171–175 explicitly requests `selected[A,C]`, `none` and allowed `selected[A]`, with a narrowed parent and disabled context; it names expected sets and modes. E25 states “effective mode is selected and exact eligible set is {A}” (`:510`), E26 states “effective mode remains all, eligible/catalog IDs equal {A,B}” (`:519`), and E27 states “effective mode is selected with exact IDs {A,B}” after a lower auto request (`:528`). Those oracles follow intersection/non-widening in `src/uar/domain/policy.rs:815–899`. E28 remains a separate negative implicit-matching case rather than a substitute for policy none/selected. The correction adds predicates within existing journeys without silently increasing scenario count.

3. **Stopping-budget finding: CLOSED.** The new section at catalog lines 709–723 supplies fixed input/history/chunk limits, total dispatch/operation caps, elapsed limits, restart/interruption limits and per-stream/startup deadlines. It states: “Overlapping classes take the stricter applicable bound per dimension” and “A required outcome not reached within these limits is a recorded TIMEOUT/FAIL; missing admitted preconditions are BLOCKED.” This prevents threshold seeding, regeneration polling or recovery waits from becoming unbounded inside one ID. The ceilings are planning constraints; they authorize no execution and prove no scenario will pass within them.

C33 now separates effective default B from persisted session override A and requires rejected writes to preserve valid state. E09 now lists finite owner-scoped run read/events/cancel and same-public-session history operations; it no longer assumes global agent definitions are owner-private. These revisions do not reintroduce credentials into scripts.

The new catalog source limitation at line 601 accurately states that a PostgreSQL primary runtime using Surreal memory is not PostgreSQL memory-control parity. `src/uar/memory/controls.rs:53–80` selects the Surreal backend or returns an explicit unsupported error; `controls_store.rs:9–10` obtains the memory service's Surreal database. C39/E03 therefore remain unverified for an independently PostgreSQL-backed memory claim. That is a disclosed support limitation, not runtime failure evidence or authority for an unsolicited implementation change.

**Still required corrections from this review: none.** Scenario definitions remain proposed and not executed; actual fixtures, receipts and platform/backend results remain unverified. The sole changed file in this re-review is this report. Only source reads and the requested documentation append occurred; no tests, checks, builds, imports, services, provider/API calls or publication ran. No production code, new guards or unrelated changes were added. Verification gate remains **CLOSED**.

## Final Review Summary

| Severity | Open count | Status |
|----------|------------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 0 | pass |
| LOW | 0 | pass |

Verdict: **APPROVE the revised proposed catalog.** All three prior design findings are resolved by source-consistent documentation. Adoption and execution remain separate operator decisions.
