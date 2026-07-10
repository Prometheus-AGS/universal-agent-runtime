# Plan — uar-final-production-hardening-2026-07

_Planned 2026-07-10 from assessment.md. All 4 operator questions resolved via
AskUserQuestion: (1) cut **v1.0.0**; (2) docs on **GitHub Pages**; (3) console
panels: **wire Provider Health + AG-UI Events, remove Memory Activity / Model
Routing / A2UI Surfaces / Artifacts**; (4) **CRA manufacturer posture**._

Success bar (binary, from goals.md): 100% ready for customer use — every
advertised feature works, zero open security alerts with honest ignore-lists,
green `main`, a tagged signed v1.0.0 release with image/SBOM/changelog, policy
files, live docs site, no config traps, suites credible and blocking.

## Changes (9, in 4 risk-ordered rounds)

### Round 1 — P0 correctness & hygiene (independent of each other)

1. **fix-embeddings-fastembed** — Replace the zero-vector placeholder in
   `VectorMatcher::embed_batch` with real inference via `fastembed` 5.17.2
   using `try_new_from_user_defined` against the repo's existing
   `src/uar/runtime/matching/models/bg-small-en-v1.5.onnx` + `tokenizer.json`
   (offline; no runtime download; `spawn_blocking` around sync inference).
   Makes `KbConfig.embedding_provider: "fastembed"` true instead of a lie.
   Document/implement re-index for previously-zero-embedded KB rows.
   - Verify: `chat-kb-retrieval.feature` passes unweakened (bdd-chat 6/6);
     direct `POST /api/knowledge/{id}/search` returns ranked matches; skill
     embedding matching + LocalEmbedding intent backend non-degenerate;
     full lib suite green; CI build time/binary impact recorded.
   - Risk: ort static-binary download in CI (cacheable); fallback = direct
     `ort` rc.12 + hand-rolled pooling. Est: L. Value: CRITICAL.

2. **green-main-ci** — Make every workflow on `main` green or explicitly
   advisory: adopt/commit the existing uncommitted `ci.yml` fix (review it —
   it predates this phase and scopes features + drops the `-D warnings`
   blanket); align `quick-tests.yml` clippy policy the same way; fix
   `comprehensive-tests.yml`'s 3 knowns (inline cargo-audit ignore parity
   with security-audit.yml, bun→pnpm lockfile step, compose health-check
   timeouts); delete `template-cleanup.yml`; diagnose+fix
   `live-integration.yml`'s failing conclusion. Real-dispatch verification
   for each (this repo's standing rule).
   - Est: M-L. Value: HIGH (customer-visible badges).

3. **re-remediate-stale-rustsec** — The 4 stale-ignored RUSTSEC families now
   have upstream patches: `cargo update -p quinn-proto` (≥0.11.15);
   hickory-proto ≥0.26.1 (RUSTSEC-2026-0119); lopdf ≥0.42 and quick-xml
   ≥0.41 (transitive via kreuzberg — bump kreuzberg if it has adopted them,
   else `[patch]`/PR upstream; do NOT silently re-ignore). Shrink
   `security-audit.yml` ignore list to genuinely unfixable only (rsa Marvin,
   hickory RUSTSEC-2026-0118), update `docs/DEPENDENCY_MANAGEMENT.md`.
   - Verify: `cargo audit` clean minus the 2 documented accepted risks;
     full suite green. Est: M. Value: HIGH.

### Round 2 — P0/P1 policy & console honesty

4. **security-policy-and-community-files** — `SECURITY.md` (manufacturer
   posture: private vulnerability reporting via GitHub, 24h/72h escalation
   targets, CVE triage SLA, CRA-alignment note), `SUPPORT.md`, issue
   templates, `docs/LICENSING.md` plain-language dual-license clarity page
   (AGPL obligations for self-hosters; commercial path), linked from README.
   - Est: S-M. Value: HIGH (research: 2026 baseline; CRA clock 2026-09-11).

5. **runtime-console-wire-or-remove** — Per operator decision: wire
   **Provider Health** (existing `GET /api/uar/providers/health`) and
   **AG-UI Events** (tap the existing normalized run-event stream); REMOVE
   the Memory Activity, Model Routing, A2UI Surfaces, and Runs-page
   Artifacts panels entirely (and `NotWiredRuntimeState` if left unused).
   Update the `runtime-console-ux` spec deltas accordingly.
   - Verify: no "not yet wired" banner reachable in the UI; wired panels
     show real data live; build+typecheck+e2e green. Est: M-L. Value: HIGH.

6. **fix-config-passthroughs** — Apply `cli.port` → `server.port` and
   `cli.jwt_required` → `security.jwt_required` in the config builder
   (matching the 17 working passthroughs); regression tests; extend
   `.env.example` and the new docs config reference with the `UAR_*__*`
   convention. Est: S. Value: MEDIUM (a silently-ignored security flag).

### Round 3 — P1 distribution & docs

7. **docs-site-github-pages** — Docusaurus site (`website/`), GitHub Pages
   deploy workflow, ingest existing `docs/*.md`, plus NEW required content:
   full configuration reference (every env var + `UAR_*__*` convention),
   install (compose + binary), **backup/restore runbook (embedded
   SurrealKV)**, **upgrade/migration guide**, troubleshooting, API
   reference. Branding per the previously-approved Prometheus guide.
   Mandatory UI/UX routing (CLAUDE.md) applies to site theme work.
   - Est: L. Value: HIGH. (Unblocked today by the hosting decision.)

8. **release-1-0-0** — LAST change, gates on all others green: bump versions
   0.1.0 → 1.0.0 (Cargo.toml, package.jsons), `CHANGELOG.md`
   (Keep-a-Changelog), stability statement, extend `release.yml` with SBOM
   (cargo-cyclonedx + syft for the image), cosign keyless signing + SLSA
   provenance attestation, GHCR multi-arch (amd64/arm64) image publish; tag
   `v1.0.0` and run the release pipeline **for real** (first release ever —
   expect pipeline fixes; that is part of the change).
   - Est: L. Value: CRITICAL (the deliverable).

### Round 4 — P2 test credibility (may overlap Round 3)

9. **test-hardening** — Convert `tests/e2e/rag.spec.ts` into a real
   upload→search→assert flow (possible once change 1 lands); strengthen or
   supersede `chat-agent-selection.spec.ts`-style visibility-only asserts;
   add targeted vitest coverage for load-bearing stores/hooks
   (chat-message-store, use-agents, auth-keys-store already covered? verify);
   flip `bdd-chat.yml` advisory→blocking at 6/6; flip `live-integration.yml`
   advisory→blocking if stable. Est: M. Value: MEDIUM-HIGH.

## Dependencies

- 8 (release) depends on 1-7 + 9 being green — cut the tag last.
- 9's rag.spec.ts work depends on 1. Everything else independent.
- 2 and 3 both touch CI security steps — sequence 3 after 2 or coordinate.

## Commands

/opsx:new fix-embeddings-fastembed → /kbd-apply …
/opsx:new green-main-ci → /kbd-apply …
/opsx:new re-remediate-stale-rustsec → /kbd-apply …
/opsx:new security-policy-and-community-files → /kbd-apply …
/opsx:new runtime-console-wire-or-remove → /kbd-apply …
/opsx:new fix-config-passthroughs → /kbd-apply …
/opsx:new docs-site-github-pages → /kbd-apply …
/opsx:new test-hardening → /kbd-apply …
/opsx:new release-1-0-0 → /kbd-apply …   (LAST)

PLAN COMPLETE
