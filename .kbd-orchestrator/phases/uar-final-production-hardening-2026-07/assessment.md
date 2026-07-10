# Assessment — uar-final-production-hardening-2026-07

_Assessed 2026-07-10. Operator mandate: **"we ONLY succeed if we are 100%
ready for customer use and consumption."** Method: direct local verification
(API calls, code reads, `gh api`, `cargo audit`/`pnpm audit`, CI run history)
plus two web-research passes (Rust embedding options; 2026 production-readiness
bar for self-hosted OSS servers) with sources cited inline._

## 0. What "100% customer-ready" means for this cycle (proposed acceptance bar)

The mandate is binary, so the bar must be concrete. Proposed definition of
done, in priority order — each maps to a finding below:

1. Every advertised feature functions to task (no zero-vector RAG, no
   "not yet wired" banners a customer can see).
2. Zero open security alerts across GitHub Dependabot, `cargo audit`,
   `pnpm audit` — with ignore-lists containing only genuinely unfixable,
   documented accepted risks.
3. `main` is green on every non-advisory CI workflow (the README now shows
   badges; red badges are customer-visible).
4. A cut, tagged, signed release exists with container image, SBOM,
   changelog — and the version number doesn't contradict the readiness claim.
5. Security/support/licensing policy files exist (SECURITY.md, SUPPORT.md,
   issue templates, licensing-clarity doc).
6. Public docs site is live with the minimum docs set (install, configure —
   including the `UAR_*__*` env convention, upgrade, backup/restore,
   troubleshoot, API reference).
7. Config surface has no silent traps.
8. Test suites credibly back the claims (bdd-chat 6/6 once RAG is fixed;
   weak visibility-only e2e assertions strengthened or superseded).

## 1. BLOCKER — RAG/KB retrieval is broken (zero-vector embeddings)

**Verified root cause** (`src/uar/runtime/matching/vector.rs:150-219`):
tokenization and tensor construction are complete, but `model.forward()` was
never wired ("UNCOMMENT ONCE COMPILED TO VERIFY SIGNATURE") and the function
returns `vec![vec![0.0; 384]; batch_size]` with only a `warn!`.

**Verified blast radius — every embedding consumer in the product funnels
through this one function:**
- KB document ingestion: `src/uar/rag/ingest.rs:70,136`, `chunking.rs:101`
- KB search API: `src/uar/api/knowledge.rs:541` (confirmed live:
  `POST /api/knowledge/{id}/search` → `{"results":[]}` for an exact-phrase
  match against an `indexed` document)
- Agent-scoped chat RAG: `src/uar/runtime/manager.rs:696`
- Skill embedding matching: `src/uar/runtime/skills/registry.rs:54,151`
- LocalEmbedding intent backend: `src/uar/runtime/matching/intent/local_embedding.rs`

**Config lie compounding it:** `KbConfig.embedding_provider` defaults to and
stores `"fastembed"` (`src/uar/domain/knowledge.rs`), but no code consumes the
field and no fastembed dependency exists — the API tells customers their KB
uses fastembed while actually using zeros.

**Validated fix path (web-researched, July 2026):**
- **Recommended: `fastembed` 5.17.2** (actively maintained, ~2-3
  releases/week H1 2026). `BGESmallENV15` is its default model;
  `TextEmbedding::try_new_from_user_defined(...)` loads the repo's existing
  on-disk `bg-small-en-v1.5.onnx` + `tokenizer.json` fully offline; pooling +
  L2 normalization built in; rides `ort =2.0.0-rc.12` prebuilt static
  binaries that build cleanly on ubuntu-latest (~30-50 MB binary impact).
  This also makes the stored `"fastembed"` config value *true*.
- Fallback: direct `ort` rc.12 + existing `tokenizers` crate (hand-rolled
  mean-pool/normalize; `spawn_blocking` for sync inference).
- **Dead ends, confirmed:** burn ONNX import for BERT-family is still broken
  upstream (tracel-ai/burn#3412, open since 2025-07 — upgrading burn does not
  help); candle would need new safetensors weights and a second ML framework;
  model2vec is a quality downgrade.
- Note: existing zero-vector rows in any persisted KB index must be
  re-embedded on upgrade (migration/re-index step, or document "re-upload").

**Acceptance:** `chat-kb-retrieval.feature` (currently red on purpose) passes
unweakened; direct search API returns ranked matches; skill embedding matching
and LocalEmbedding intent backend verified non-degenerate.

## 2. Security posture — better than believed, but ignore-lists have gone stale

- **GitHub Dependabot: 0 open alerts** (verified via
  `gh api .../dependabot/alerts`, state=open → `[]`; the 131 historical
  alerts are all `fixed`). The "2 vulnerabilities" push-time banner is the
  known-stale GitHub artifact prior phases documented. The prior reflection's
  "2 open alerts" claim is **outdated — already resolved**.
- **`cargo audit`: 11 vulnerabilities, all currently `--ignore`d** in
  `security-audit.yml` — but **four ignore rationales are now false**:
  - `lopdf 0.40` ignored as "no crate fix exists" → **fix exists: ≥0.42.0**
  - `quick-xml 0.37/0.39/0.40` ignored as "no crate fix exists" → **≥0.41.0**
  - `hickory-proto 0.25.2` (RUSTSEC-2026-0119) → **≥0.26.1 now patched**
  - `quinn-proto 0.11.14` "orphaned lock entry" → **0.11.15 exists; trivial
    `cargo update -p quinn-proto`**
  Remaining legitimately unfixable: `rsa` Marvin (RUSTSEC-2023-0071, no
  patch, accepted risk) and hickory RUSTSEC-2026-0118 (no patch yet).
  **Work item:** bump the four fixable families (mostly transitive via
  kreuzberg — may require a kreuzberg bump), shrink the ignore list to
  genuinely unfixable entries, update `docs/DEPENDENCY_MANAGEMENT.md`.
- **`pnpm audit` (root + frontend): clean.**
- **Missing policy files (2026 baseline per research):** no `SECURITY.md` /
  private-vulnerability-reporting policy (OpenSSF baseline; prerequisite for
  EU CRA Art. 14), no `SUPPORT.md`, no issue templates, no `CHANGELOG.md`.
- **CRA clock:** reporting obligations start **2026-09-11** (two months out)
  for products commercially available in the EU; selling the commercial
  license likely makes this a "manufacturer." Posture decision + SECURITY.md
  needed this cycle.
- Positive verified state: JWT auth on by default (`security.jwt_required`
  default `true`), secrets redacted in Debug output, rate limiting on by
  default, guardrails/Cedar mounted.

## 3. BLOCKER — `main` is red on most CI, and the README now advertises it

Last runs on `main`: **failure** on `CI`, `Tests (Quick)`,
`Comprehensive Test Suite`, `live-integration`, `template-cleanup`; success
only on `BDD Chat Scenario Suite` and `Build and Deploy to AKS`. Known causes:
- `ci.yml` (committed version) runs `cargo clippy --all-features -- -D
  warnings` → fails on ~500+ pre-existing pedantic warnings and the broken
  `model-build`/`memory-palace`/`sandbox-microsandbox` features. **A fix
  already exists as an uncommitted working-tree diff to `ci.yml`** (feature
  scoping + warning-policy rationale) that was never committed by whoever
  authored it.
- `quick-tests.yml` has the same `-D warnings` blanket-escalation problem.
- `comprehensive-tests.yml`: three documented follow-ups never triaged —
  inline `cargo audit` missing the ignore list `security-audit.yml` has;
  `bun install --frozen-lockfile` with no `bun.lockb` (repo moved to pnpm);
  docker-compose test health-check timeouts.
- `template-cleanup.yml`: leftover from the repo template; should be deleted.
- `live-integration.yml`: advisory steps but the run still concludes failure — needs a look.
**Acceptance:** every workflow on `main` green (or explicitly advisory and
labeled as such), README badges truthful.

## 4. Runtime Console gated panels — wire-vs-remove, now with real scoping

- **Provider Health (Cockpit):** backend already serves real data —
  `GET /api/uar/providers/health` (registry health snapshot,
  `src/uar/api/providers.rs:69`). Wiring the panel is plain frontend work. No
  excuse to keep the banner.
- **Memory Activity, AG-UI Events, Model Routing, A2UI Surfaces, Artifacts:**
  zero backend emission exists for the frontend entity types
  (`RuntimeMemoryEvent`, `RuntimeAgUiEvent`, `RuntimeProviderHealth`,
  `RuntimeArtifact` — no Rust emitter anywhere). The runs domain has **no
  artifacts concept** at all. Honest options per panel: (a) emit real events
  server-side (the normalized run-event stream already exists to tap for
  AG-UI events), or (b) remove the panel. A customer-facing product cannot
  ship "not yet wired" banners under this mandate — each panel needs an
  explicit wire-or-remove decision at plan time.

## 5. Config surface traps — small and precisely scoped

- Exactly **two** dead CLI/env passthroughs (verified against all 19 `Cli`
  fields): `cli.port` (`PORT`) and `cli.jwt_required` (`JWT_REQUIRED`) are
  parsed but never applied. Fix: apply them (config-builder overrides like
  the other 17) — silently ignoring a *security* flag (`JWT_REQUIRED`) is the
  worst kind of trap.
- `KbConfig.embedding_provider` stored-but-ignored (fixed by Goal 1 if
  fastembed is adopted; otherwise honesty fix).
- The `UAR_*__*` convention needs a complete env-var reference in docs (no
  configuration reference exists today; `.env.example` is partial).

## 6. Test coverage — thin, with known-weak assertions

- Frontend unit tests: **12 of 209 source files (~5.7%)**.
- Documented visibility-only e2e specs (`tests/e2e/rag.spec.ts`,
  `frontend/e2e/chat-agent-selection.spec.ts`) — the exact pattern that hid
  the agent-selector bug. Strengthen or formally supersede/remove.
- bdd-chat suite: 5/6 in CI (advisory), gated on Goal 1 to become 6/6, then
  flip the workflow from advisory to blocking.
- Backend: lib suite healthy (387 passing at last count); cucumber-rs suite
  9/9; live-integration recorded backend exists.
- Realistic bar this cycle: not %-coverage worship — instead (a) all suites
  green + blocking, (b) load-bearing stores/hooks covered, (c) no
  visibility-only assertions on load-bearing paths.

## 7. Release engineering & distribution — never exercised

- `version = 0.1.0` everywhere, **zero git tags, zero releases ever cut**,
  despite a full `release.yml` pipeline. Per research: declaring "100%
  customer-ready" at unsigned, untagged 0.1.0 is self-contradictory — either
  cut **1.0.0** with a stability statement or publish an explicit 0.x
  stability policy. (Operator decision at plan time; recommendation: 1.0.0.)
- Dockerfile + prod compose files exist; GHCR multi-arch publishing,
  cosign/SLSA attestation, and SBOM (CycloneDX via `cargo cyclonedx` + syft)
  are absent — all 2026-baseline expectations per research.
- No CHANGELOG.md; release.yml has never run against a tag.
- Backup/restore + upgrade/migration documentation for embedded SurrealKV:
  **does not exist** — research flags this as a hard adoption blocker for
  stateful self-hosted servers.

## 8. Licensing — mostly in place, needs a clarity statement

`LICENSE` (AGPL-3.0) + `LICENSE-COMMERCIAL.md` dual licensing already exist.
Per research, AGPL is auto-rejected by many enterprise legal departments;
what's missing is only a **plain-language licensing page** (what AGPL
does/doesn't require of a self-hosting customer; when the commercial license
applies; contact path) linked from README and the docs site.

## 9. Docs site — decision needed, default recommended

Research verdict: **Docusaurus on GitHub Pages via GitHub Actions** is the
lowest-friction default for a GitHub-org repo (free, no new accounts, official
deploy path; PR previews can come later via Vercel/Netlify if wanted).
Minimum docs set to ship: install (compose + binary), full configuration
reference (env conventions incl. `UAR_*__*`), upgrade, backup/restore,
troubleshooting, API reference. Existing `docs/*.md` (ARCHITECTURE,
DEPLOYMENT, etc.) provide substantial raw material.

## 10. Operational surface — mostly good, verified

- `/health`, `/healthz` (liveness) + `/readyz` (readiness): **exist**.
- Prometheus `/metrics`, JSON structured logs, OTLP tracing: **exist** (prior
  phases, verified in waypoint history).
- Missing: resource-sizing guidance, backup/restore runbook (see §7).

## Priority ordering for /kbd-plan

| P | Item | Anchor |
|---|------|--------|
| P0 | Real embeddings via fastembed (repairs RAG, KB search, skill matching, intent) + re-index/migration note + bdd-chat 6/6 | §1 |
| P0 | Green `main`: commit the ci.yml fix, align quick-tests, fix comprehensive-tests' 3 knowns, delete template-cleanup, diagnose live-integration | §3 |
| P0 | Re-remediate the 4 stale-ignored RUSTSEC families; shrink ignore list; SECURITY.md + private reporting + CRA posture | §2 |
| P1 | Runtime Console: wire Provider Health (backend exists); wire-or-remove decision + execution for the other 5 panels | §4 |
| P1 | Release: version decision (rec: 1.0.0), tag + run release.yml, GHCR image, SBOM, cosign, CHANGELOG | §7 |
| P1 | Docs site on GitHub Pages + minimum docs set (config reference, backup/restore, upgrade) | §9 |
| P1 | Config traps: apply `PORT`/`JWT_REQUIRED`, env-var reference | §5 |
| P2 | Test hardening: strengthen/remove visibility-only specs, cover load-bearing stores/hooks, flip bdd-chat to blocking | §6 |
| P2 | SUPPORT.md, issue templates, licensing-clarity page, sizing guidance | §2/§7/§8 |

## Open questions for the operator (surface at plan time)

1. **Version signal:** cut 1.0.0 (recommended — "100% ready" and 0.1.0
   contradict) or stay 0.x with a written stability policy?
2. **Docs hosting:** accept the GitHub Pages default? (Blocks Goal 6 —
   previously deferred on exactly this decision.)
3. **Runtime Console panels without backend concepts** (Artifacts, A2UI
   Surfaces, Model Routing, Memory Activity): wire (real backend emission
   work) or remove per panel?
4. **CRA posture:** does the commercial license make this an EU-market
   "manufacturer"? (Affects SECURITY.md content and the Sept 2026 clock.)

## Honesty note on "100%"

"100% ready" is achievable this cycle for everything enumerated above; the
two items with real schedule risk are (a) the fastembed integration if the
`ort` static-binary build misbehaves in CI (fallback documented), and (b)
kreuzberg transitive bumps for lopdf/quick-xml if upstream hasn't adopted the
patched versions (fallback: direct dependency patching via `[patch]` or a
kreuzberg PR/fork — must not be silently re-ignored). Nothing else found is
research-hard; it is execution volume.

_Sycophancy self-check: this assessment overturns one prior claim in the
operator's favor (Dependabot alerts already 0, not 2) and adds several the
operator didn't ask about (red CI on main, stale ignore rationales, no
release ever cut, missing backup/restore docs) — it optimizes for the actual
bar, not for agreeing with the brief._
