# Reflection — uar-1-0-readiness

Written 2026-08-16 at phase close. Results are requirement-scoped. The phase
contract forbids an aggregate percentage and a runtime-level verdict, so neither
is reported here.

## 1. Delta from the plan

### Evidence work displaced implementation

Execution initially treated evidence obligations as prerequisites instead of
completion gates. Repeated broad builds, scratch controls, and artifact rewrites
ran while A0 was still changing. The operator stopped that pattern. The corrected
order was one cohesive implementation, one Tier 0 sequence, focused Tier 1 and
negative controls after the positive path worked, and artifact assembly last.

### Five planned changes became six

A1 could not start safely on the planned baseline: `jsonwebtoken` 11.0.0 had no
active provider on UAR's path, so valid JWT operations could panic. A0 was
inserted before Track A. Its first spike favored AWS-LC mainly because that
backend was already transitive. Operator challenge caused a contained
re-evaluation, and the binding choice became exactly `jsonwebtoken` 11.0.0 with
RustCrypto for every UAR-owned dependency.

### The first RustCrypto ownership claim was false

The proposed pointer-identity guard could not identify an already installed
provider. `jsonwebtoken` returns the attempted provider when installation loses
the global slot, while its installed-provider getter is private. An adversarial
AWS-LC-first replay demonstrated false acceptance. The enforceable rule is
UAR-first ownership, not identity recovery after another installer wins.

### Restart and compatibility claims were initially weaker than their words

B3 first reused a live SurrealKV provider while calling the result restart
durability. B4 first reconstructed a service over one open database handle, then
preserved pending legacy bindings for GET without applying them when future
skills loaded. B5 first filtered tombstones after a database top-five limit,
returned tombstones from refresh, and had not observed the required empty-source
error log. Independent review rejected each claim until the lifecycle or
behavioral boundary was exercised directly.

### Canonical and agent-seeded progress diverged

Canonical KBD revision 102 recorded all six changes complete. The agent-seeded
`progress.json` still reported all six as TODO, and the generated next command
still pointed at Execute. The runtime refused to overwrite a file it did not own.
Canonical state remained authoritative, but the stale projection could have sent
the next executor backward.

## 2. Root causes

1. The evidence contract was interpreted as work-order guidance rather than as
   the definition of done. Compiler-cache and target-build problems amplified
   the delay but did not cause it.
2. The original plan reasoned about new verifier behavior without first running
   the existing token path. The first backend recommendation then over-weighted
   dependency topology relative to the architecture choice UAR wanted to own.
3. Provider ownership semantics were inferred from `OnceLock` failure data
   instead of executing both provider orderings against the pinned crate.
4. Service reconstruction was mistaken for process restart, response-shape
   compatibility for matching behavior, and primary-list visibility for all
   discovery surfaces.
5. KBD had two projections with different ownership rules and no reconciliation
   path for the agent-seeded file.

## 3. Corrective actions

1. Preserve the critical path now written in `plan.md`: cohesive implementation,
   cheap inspection, one Tier 0 sequence, focused Tier 1, then controls and
   records. Documentation-only edits do not restart Rust builds.
2. Execute one minimal round trip through a dependency boundary before planning
   features on top of it.
3. Separate “already in the dependency graph” from “the standard the project
   chooses to own.”
4. Require separate child processes for cold-restart claims and bind-before-load
   behavioral tests for compatibility claims.
5. Check tombstone visibility across list, refresh, search, vector, match, and
   direct audit retrieval. Capture required logs instead of inspecting logging
   source.
6. Treat canonical KBD state as the only position authority. Do not infer work
   from an unowned projection; repair projection ownership in a dedicated
   control-plane change.

## Requirement results

| Requirement | Result | Limit |
|---|---|---|
| RustCrypto standardization | **MET.** Every UAR-owned `jsonwebtoken` dependency inherits exactly 11.0.0 with defaults disabled and `rust_crypto` enabled; UAR and the proxy initialize it explicitly. | A prior process installer is rejected even when it installed RustCrypto. |
| JWKS/RS256 `TokenVerifier` | **MET.** Shared-secret and JWKS verification use one abstraction; rotation, unknown `kid`, issuer, audience, and effective `jwt_required` behavior were observed. | `server-full` only. |
| Verified tenant partitioning | **MET.** Verified claims create typed tenant identity; A2A task and context lookups are tenant-keyed; HTTP and gRPC cross-tenant reads are denied. | A2A storage only, not sessions, memory, knowledge bases, or other stores. |
| C-21 replacement | **MET.** A real two-tenant denial replaced the exclusion. The pinned command observed 29 passing and 0 failed; its tenant-key inversion made the exact case exit 101 before exact restoration. | Recorded backend, `server-full`, one test thread. |
| Embedded built-ins | **MET.** Fresh seeding, seeding-disabled reload, and enabled re-registration were observed across three processes without duplicates. | `server-full` construction of the embedded path. |
| Scoped skill governance | **MET.** Scope precedence, cold restart, next-match live effect, in-flight stability, origin deletion rules, filesystem round trip, and legacy binding behavior were observed. | `server-full` only. |
| Config reconciliation | **MET.** Config skills add, change, tombstone, and restore without hard delete; API-created and built-in skills survive; scoped state survives; empty input refuses tombstoning. | `provider_id = "fs-skills"` remains the safety boundary. |
| OpenSpec and evidence contract | **MET.** Six strict-valid changes carried row-form verification and were archived in dependency order into five strict-valid capability specs. | Unchecked boxes are non-fired stop conditions. |

## Artifact-refiner outcome

All six artifacts record converged state. Their iteration counts were A0 4, A1
2, A2 1, B3 2, B4 3, and B5 3. This count is not an aggregate quality score.
A0 and A1 retain converged state and passing receipts but no `finalized_at`
timestamp or history snapshot; the other four retain finalized snapshots.

Independent review materially changed A0, B3, B4, and B5. Several early
`terminate` decisions preceded later reachable findings, so convergence metadata
cannot substitute for the final critic and judge evidence.

## Architecture delivered

- JWT selection is centralized in workspace dependencies. UAR-owned operations
  pass through one crate-private wrapper, and startup acquires provider ownership
  before routes or readiness.
- Only verified claims can construct the tenant proof consumed by A2A storage.
  Task and context keys encode tenant identity rather than relying on callers.
- Embedded registration uses the existing registry and persistence paths.
- Skill governance is durable data with conversation over explicit durable
  agent, legacy binding fallback, then global precedence.
- Reconciliation is provenance-gated and reversible. Removal is a tombstone,
  restoration preserves scoped state, and hard deletion was not introduced.

The uncomfortable architecture limit is process composition: if another
component initializes `jsonwebtoken` first, UAR fails closed even when that
component selected RustCrypto. That is deliberate under the pinned API, but it
is not transparent interoperability.

## Debt carried forward

1. `fix-skills-scope-semantics` still needs the operator's supersession decision;
   `skill-scoped-governance` absorbed its intended behavior.
2. `add-skill-kind-and-origin` remains 8/11 although Track B consumes the
   existing origin representation.
3. The recommended `multi-tenant-isolation` to `user-data-isolation` rename and
   unrelated `SPECIFICATION.md` citation drift remain deliberately untouched.
4. `harden-jwt-defaults` remains pending and overlaps middleware behavior now
   owned by the merged `jwt-hardening` spec. Its next plan must reconcile that
   spec instead of replaying old assumptions.
5. The agent-seeded progress projection remains stale by design until its
   ownership problem is fixed.
6. These behavioral results do not transfer beyond their stated `server-full`
   rows. A0's iOS and Android `embedded-mobile` compile checks apply only to
   those targets.

## Coordination lessons and next action

Track order held: A2 consumed authenticated tenant identity, B4 consumed built-in
registration, and B5 preserved B4 state. The tracks were file-disjoint, but
shared build volume, canonical transitions, and evidence review made sequential
execution easier to reason about; parallelism would not have removed the main
delays.

The reviews that changed the result executed the uncomfortable boundary:
AWS-LC-first installation, real process exit, bind-before-load, vector-limit
filtering, and empty configuration with a stored catalog. The lesson is not to
collect fewer proofs. It is to build the smallest working behavior first and
then aim proof at the boundary most likely to falsify the claim.

The six OpenSpec changes are archived and the five merged specs validate
strictly. Complete Reflect and the parent phase in canonical KBD state. The next
lifecycle action is `/kbd-new-phase`; no next implementation phase is selected
here.
