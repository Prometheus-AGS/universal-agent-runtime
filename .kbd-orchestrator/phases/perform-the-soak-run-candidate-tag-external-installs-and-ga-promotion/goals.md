# Goals: perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion

Seeded from: prior phase `uar-final-production-hardening-2026-07` (no `reflection.md` was
generated for that phase — /kbd-reflect was not run before this transition. Goals below are
hand-seeded from that phase's `current-waypoint.json` implementation audit and the four
active OpenSpec changes' unchecked evidence tasks, which is the same source a reflection
would have drawn from).
Created: 2026-07-13T17:52:40Z

## Context

`uar-final-production-hardening-2026-07` closed at 24/24 **implementation** completion
(PR #108, merged into `main` at `ca655a6`). All code/integration requirements for the four
OpenSpec changes below are implemented and locally validated. What remains is exclusively
**release-evidence generation and publication** — running already-implemented automation,
not writing code.

## Seeded Goals

Work through these in order; each later step depends on the previous one's evidence:

1. **Operational resilience evidence** (`openspec/changes/certify-operational-resilience`)
   - Run the full resilience/soak certification (`scripts/certify-operational-resilience.sh`
     driving `tests/operational_resilience.rs` + `scripts/certify-release-candidate.sh`),
     including the multi-hour streaming/reconnect soak (`UAR_SOAK_DURATION_SECONDS`), and
     upload machine-readable results as CI artifacts via `.github/workflows/operational-resilience.yml`.

2. **Signed supply-chain artifacts** (`openspec/changes/produce-supply-chain-artifacts`)
   - Generate real SBOMs, checksums, SLSA provenance, and signed multi-arch GHCR images
     against an actual candidate build via `.github/workflows/supply-chain.yml`, then
     independently re-verify them in the workflow's separate `verify` job.

3. **Release candidate certification** (`openspec/changes/certify-release-candidate`) —
   **requires operator authorization to cut a tag**
   - Freeze the final `1.0.0` source commit under the next unused candidate tag `v1.0.0-rc.3`.
   - Run the complete CI/security/offline/platform/UI/resilience matrix against it.
   - Publish signed candidate artifacts and the evidence manifest.
   - Install the binary/archive/container on every supported platform with no dev checkout;
     exercise install/config/backup/restore/upgrade/troubleshoot docs paths.
   - Record at least three external installations and one week of operation without
     maintainer intervention (time-bound; cannot be completed in one session).
   - Open focused follow-up changes for any failure and rerun certification on any source change.

4. **GA promotion** (`openspec/changes/release-1-0-0`) — **requires operator authorization
   to publish and mutate public state**
   - Confirm source still equals the certified candidate.
   - Create the signed `v1.0.0` tag via the guarded no-rebuild promotion script
     (`scripts/promote-release-candidate.sh`) — retags the certified OCI digest, performs
     no rebuild, requires an exact `UAR_CONFIRM_GA_PROMOTION` confirmation string.
   - Publish the GitHub release, signed images, SBOM/provenance/checksums, and evidence manifest.
   - Download and verify all artifacts from public endpoints; run production smoke/health
     and documentation link checks.
   - Archive the four OpenSpec changes and close this KBD phase, but only once every goal
     above is genuinely met — not on elapsed time or partial evidence.

## Expanded scope (operator, 2026-07-13, via /kbd-assess arguments)

The operator expanded this phase from release-evidence-only to **full customer-release
readiness**. Before (or alongside) the certification goals above, the phase must also close:

5. **Documentation & packaging readiness**
   - Rewrite `README.md` to address the entire package, with mermaid architecture, flow,
     and scenario diagrams, including UAR's relationship to the wider fabric
     (flint-realtime-fabric, flint-gate, flint-forge, etc.).
   - Assess/repair SDKs (rust, typescript) and the Docusaurus site generation for customers.

6. **Competitive analysis** — research the Mastra framework and its Playground; compare
   feature-by-feature against UAR's built-in React 19 application.

7. **Screen-by-screen functional validation** of the web tools and admin functions:
   validate each screen's purpose and function individually, including local-first behavior,
   and confirm conversations can be started with ANY defined agent as the focus.

8. **Live browser validation with video proof** — real live integration tests in the browser
   proving the orchestrator and default agents return expected answers.

9. **Skills lifecycle** — add/enable/remove skills; a supported method to download and
   install the prometheus skill system from its repository (including Rust toolchains,
   build, install) onto a UAR installation; admin UI shows ALL skill-pack skills; skill-pack
   skills cannot be deleted, only disabled per-conversation, per-agent, or globally.

10. **Runtime behaviors** — intent classification and intent-based skill activation work;
    context management works; AG-UI endpoint emits all event/chunk types; RAG knowledge
    bases work, can be assigned to agents, and KB hit event chunks render in the UI; JWTs
    minted with a JWT secret work; memory works at global, agent, and user levels; user
    isolation holds with no cross-user bleed.

## Non-goals / explicit boundaries

- No further source/integration code changes are expected in this phase — if a defect is
  found during certification, open a narrowly-scoped follow-up change rather than folding
  ad hoc fixes into the release pipeline.
- Tag creation, GHCR/GitHub publication, and GA promotion are irreversible, publicly visible
  actions. Each MUST get explicit operator confirmation at the point of execution — creating
  this phase or running `/kbd-assess`/`/kbd-plan` does not itself authorize them.
- The 3-external-install-plus-one-week gate in change 3 is genuinely time-bound; do not
  manufacture or infer this evidence.

---

## Instructions

Review and refine the goals above before running `/kbd-assess`.
Add, remove, or clarify as needed. When ready:

```
/kbd-assess perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion
```
