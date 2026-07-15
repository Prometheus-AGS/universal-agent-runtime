---
title: "UAR Post Grade-A Re-Assessment (validated)"
date: 2026-07-15
phase: uar-grade-a-upgrade-2026-07 (25/25 implementation-complete; 25/25 merged)
supersedes: the 2026-07-15 draft re-assessment (which scored against a stale
  tree and contained factual errors in both directions — see §2)
prior-assessments:
  - uar_release_readiness_assessment_2026-07-13.md (the baseline this report re-scores)
  - uar_assessment_2026-02-21.md (the original baseline)
validation:
  - every checkable repo claim re-verified against main @ ed44924 (2026-07-15)
  - external competitive claims re-verified by web search (see §9 sources)
  - sycophancy-correction detector run on the verdict text (score 0.0 on
    S-01..S-08 linguistic patterns); the substantive correction work is the
    fact-check in §2, which changed conclusions in both directions
---

# Universal Agent Runtime — Post Grade-A Re-Assessment (2026-07-15, validated)

> **Headline.** All **25 of 25** grade-A changes are implementation-complete
> and **all 25 are merged into `main`** (PRs #112–#131; Change 21 merged as
> PR #131 / `ed44924`, Change 22 as PR #130). The draft version of this
> report — written a few hours earlier against a stale tree — claimed
> Change 21 was never started and under-credited several shipped surfaces
> (ADRs, property tests, `security.txt`, reproducible builds, cosign
> verification docs). Those claims are corrected below with evidence.
>
> **Customer verdict (the question the operator asked).** The
> *implementation* layer is done. The *release evidence* layer — the thing
> that separates "code-complete" from "customers can rely on it" — has not
> run: no release-candidate tag, no external installs, no soak period, no
> published signed artifacts verified from a customer's vantage point. The
> honest recommendation is: **start design-partner / early-access customers
> on a tagged release candidate now; do not call it GA until the existing
> certification scripts have actually executed** (3 external installs,
> 1-week soak, signed-artifact publication + public verification, no-rebuild
> promotion). Every script and workflow for that track already exists; none
> has been run. There is one accuracy fix that should land **before**
> customer-facing security collateral goes out (§6.1, the SLSA level claim).

---

## 1. State of the plan (verified against `main` @ `ed44924`)

| Metric | 2026-07-13 baseline | 2026-07-15 (verified) | Draft claimed |
| --- | --- | --- | --- |
| Changes implementation-complete | 0/25 | **25/25** | 24/25 ✗ |
| Changes merged to `main` | 0/25 | **25/25** (PRs #112–#131) | 16/25 ✗ |
| Rust source files / lines (`src/`) | 263 / 68,290 | 285 / 73,707 | 284 / 73,332 (~) |
| `unwrap()/expect()` in `src/uar/` | 382 | **449** (regression, +67) | 456 (~) |
| `anyhow!()` in `src/uar/` | 130 | **137** | 137 ✓ |
| `src/config.rs` lines | 2,046 | **2,191** (migration did not land) | 2,191 ✓ |
| `src/server.rs` lines | 5,271 | **5,388** | 5,380 (~) |
| `src/uar/error.rs` (central `UarError`) | — | **305 lines, real** | ✓ |
| GitHub Actions workflows | 9 | **23** | 18 ✗ |
| ADRs in `docs/adr/` | 0 | **12** (0001–0011 + index + legacy ADR-007) | 1 ✗ |
| Fuzz targets | 0 | **4** (chunker, json_schema_validator, mcp_message_parser, rag_verification) | 4 ✓ |
| `proptest!` property-test files | 0 | **3** (`rag/retrieval.rs`, `governance/engine.rs`, `domain/prompt_caching.rs`) | 0 ✗ |
| Coverage baseline doc | — | **`docs/coverage-baseline.md` exists** | "not documented" ✗ |
| `security.txt` endpoint | — | **served at `/.well-known/security.txt` (RFC 9116), `src/server.rs:819`** | "missing" ✗ |
| Prose lint (Vale) | — | **`.vale.ini` + `.github/styles/UAR/` exist, wired in `docs.yml`** | "missing" ✗ |
| Standalone cookbook | — | **`docs/cookbook/` exists** (runtime + SDK + A2UI dirs; 8 runnable files there + 25 SDK example files) | "missing" ✗ |
| Entity components in a2ui-uar | 1 | **7** (EntityCard + 6 in `entity-extensions.tsx`) | 5 ✗ |
| SLSA provenance attestation | — | **`actions/attest@v4` + `actions/attest-sbom@v4` + cosign sign, with an independent `verify` job** in `supply-chain.yml` | "workflow missing" ✗ (see §6.1 for the real, narrower issue) |
| Reproducible builds check | — | **`Offline Reproducible Source` job in `ci.yml`, every run** | "missing" ✗ |
| cosign verify documented for users | — | **yes, README §Security with exact commands** | "missing" ✗ |
| Pricing bands in `LICENSE-COMMERCIAL.md` | — | **absent** (verified by grep) | ✓ |

Legend: ✓ draft claim verified correct · ✗ draft claim falsified · (~) minor drift, immaterial.

## 2. What the draft got wrong, and why it matters

The draft was written against a checkout that predated the final two merges
and, more importantly, made **absence claims without checking `main`**.
Eleven of its checkable claims were false. The errors ran in *both*
directions, which is worth stating plainly because it changes the verdict:

**Under-credited (draft was too pessimistic):**
1. **Change 21 shipped.** `a2ui-world-class-theming-a11y-i18n` merged as
   PR #131: theme tokens in `styles.css` + `tailwind.config.js`, an
   `i18n.tsx` locale layer, `SurfaceErrorBoundary`, dedicated
   `accessibility.test.tsx` and `ux.test.tsx`, and a11y fixes across the
   protocol components. Note the shape differs from the plan's imagined
   `themes/`/`i18n/`/`a11y/`/`animation/` directory layout — leaner, but the
   capability is real and tested (26 functional + 2 performance tests pass
   per the merge record). Motion/animation is CSS-level (including
   reduced-motion), not a Motion-library integration.
2. **Change 22 shipped.** Lit + Svelte renderers, the A2UI Inspector (with
   its Storybook addon export), and cross-renderer semantic conformance
   merged as PR #130.
3. **The documentation layer is stronger than scored**: 12 ADRs, Vale prose
   lint, the standalone cookbook, hosted-docs pipeline — all present.
4. **The supply chain is stronger than scored**: SLSA provenance is
   attested via GitHub's native `actions/attest@v4`, SBOMs via
   `attest-sbom@v4`, keyless cosign signatures, an *independent* verify job
   (not self-certified by the producing job), reproducible-source
   verification on every CI run, and user-facing verification commands in
   the README.

**Over-credited or internally inconsistent (draft was too optimistic):**
5. The draft's §1 said composite **4.2** while §4/§8 said **4.4** — two
   different rubrics conflated into one headline.
6. "Nothing material is missing in the implementation layer" sat next to a
   list of ~61 hours of implementation work. Both could not be true.
7. The competitive table awarded UAR 5s partly from self-assessment;
   several cells (e.g., "Test posture 5 vs LangGraph 5") are not backed by
   any comparable external measurement. Treat that table as directional,
   not measured (§5).

**Correctly identified (and still true):**
8. `unwrap()/expect()` **regressed**: 382 → 449 in `src/uar/`. The central
   `UarError` architecture is real; the hygiene sweep did not keep pace
   with new code.
9. The `anyhow!()` → `UarError` call-site migration was partial (137 remain).
10. The `config-rs`/schemars macro migration did not land; `src/config.rs`
    grew to 2,191 lines. The secret hardening (`secrecy::SecretString`,
    Vault adapter, hot-reload watcher) did land.
11. `LICENSE-COMMERCIAL.md` has **no pricing bands**, and the contributor
    open letter has not gone out — both operator items, both still open.
12. The release evidence track (external installs, soak, RC tag, public
    verification) **has not executed**. Scripts and workflows all exist.

## 3. Corrected scorecard

Letter grades are self-assessed against the 2026-07-13 rubric; they are a
management summary, not an external audit.

| Area | 2026-07-13 | 2026-07-15 (validated) | Notes |
| --- | --- | --- | --- |
| §2 SDK | C | **A** | 3 SDKs at 1.0.0; 25 example files; typed errors (miette/discriminated unions); MIT; BREAKING.md each |
| §3 Configuration | B | **B+** | Secret hardening + Vault + hot-reload landed; the <800-line macro rewrite did not (2,191 lines) |
| §4 RAG | B+ | **A** | citation_stream; candle/voyage/cohere/openai backends; RAGAS+DeepEval golden set; monthly BEIR |
| §5 Error handling | B | **A−** | Central `UarError` (305 lines, stable `E_*` codes, IntoResponse, SpanTrace) is real; unwrap count regressed 382→449; 137 `anyhow!()` remain |
| §6 Build/test/lint | B | **A** | coverage gate (60%), nightly mutants, 4 fuzz targets, 3 proptest files, perf budget CI, storybook a11y gate |
| §7 Supply chain | B+ | **A−** | attest + attest-sbom + cosign + independent verify + reproducible source; held at A− for the L3-wording accuracy issue (§6.1) and until a signed release has actually shipped through it |
| §8 License | C | **A−** | Dual license + MIT SDKs + commercial doc shipped; held at A− until pricing bands + contributor letter close |
| §9 Documentation | B+ | **A** | 12 ADRs, Docusaurus portal, typedoc/rustdoc pipelines, Vale, cookbook, assessments |
| §10 A2UI | — | **A−** | 9-component certified catalog + 7 entity components + theming/a11y/i18n + Inspector + Lit/Svelte renderers + cross-renderer conformance + Storybook/Chromatic; held at A− because theming/i18n shipped leaner than the plan's spec and Chromatic needs a token to actually run |
| **Composite (mean of 9)** | **3.5** | **≈4.3** | One number, one rubric, stated once |

## 4. What "world-class A2UI" actually looks like now

The A2UI stack — the area the draft scored hardest — is, verified on `main`:

- `a2ui-core` (vendored `@a2ui/web_core` 0.10.4) + `a2ui-react` (vendored
  Google reference, cross-test only)
- `a2ui-uar`: the UAR-owned React renderer — 9 certified protocol
  components, 7 entity components, theming tokens, i18n layer, a11y-tested,
  error-boundary-wrapped, perf-budgeted (16ms initial / 8ms streaming
  budgets in CI)
- `a2ui-lit` + `a2ui-svelte`: framework-parity renderers over the same
  `web_core` state model, with a shared semantic-conformance fixture
  asserting equivalent roles/names/states/text across all three frameworks
- `a2ui-inspector`: dev-only SSE inspector with freeze/resume and a
  Storybook addon export
- Realtime: `StatePatch` conversion + replay backbone with two HTTP
  endpoints (Change 20); durable multi-process replay via
  flint-realtime-fabric remains deferred pending a vendoring decision
- Storybook 10.5 with 38 stories, fail-closed axe a11y gate, and a
  Chromatic workflow (needs `CHROMATIC_PROJECT_TOKEN` to activate)

No other runtime in the compared set ships a validated declarative
agent-UI contract with three framework renderers and semantic conformance
testing. That claim survives the fact-check.

## 5. Competitive position (directional, re-verified where checkable)

External claims re-verified by web search this pass:

- **CrewAI 1.14.6 (May 28, 2026) + June 11, 2026 pluggable backends** for
  memory/knowledge/RAG/flow is real and is the most material competitive
  move of Q2 2026 in the RAG-adjacent space. UAR's answer (golden-set
  evals, 4 embedding backends, monthly BEIR publication) is the right kind
  of response; CrewAI's velocity is worth tracking quarterly.
- The SLSA landscape shifted: GitHub's native artifact attestations give
  **SLSA v1.0 Build L2 by default**, and **L3 requires the build to run in
  a dedicated reusable workflow** (or `slsa-github-generator`). This is
  the basis for §6.1.

The draft's 14-row competitive matrix is retained in spirit but should be
read as **directional self-assessment**: UAR leads clearly on protocol
coverage (A2A+MCP+AG-UI+A2UI), runtime Cedar governance, and declarative
agent UI; it is at parity on SDKs, docs, and CI discipline with the
best-funded competitors; it trails nobody in the compared set on supply
chain *mechanics* but has not yet shipped a signed release through them.

## 6. Critical items before customer exposure

### 6.1 Fix the SLSA level claim (accuracy, ~2h) — do this before customer security collateral
`README.md` and `SECURITY.md` describe the posture as "SLSA L3
self-declared." Per GitHub's own documentation, native artifact
attestations from a workflow that builds in-repo achieve **Build L2**;
**L3 additionally requires the build steps to run in a separate reusable
workflow** (provenance non-falsifiable by the build-step author) — or use
of `slsa-github-generator`. Two acceptable fixes: (a) reword to "SLSA
Build L2 attested, L3-track" — 30 minutes; or (b) move the build/sign
steps into a reusable workflow and keep the L3 claim — roughly a day.
Overclaiming a compliance level in security collateral is the kind of
thing procurement teams check.

### 6.2 Unwrap sweep on production hot paths (~6h)
449 `unwrap()/expect()` in `src/uar/` (up from 382). Add
`clippy::unwrap_used`/`expect_used` as warn-level lints scoped to
`src/uar/{api,runtime}/` and burn down the hot-path subset. Not a GA
blocker by itself; it is the largest hygiene debt.

### 6.3 Operator items (not code)
- **Pricing bands** in `LICENSE-COMMERCIAL.md` — without public pricing the
  commercial path is effectively "contact us," which suppresses conversion.
- **Contributor open letter** for the SDK relicense (30-day window).
- **`CHROMATIC_PROJECT_TOKEN`** to activate visual regression.
- **Merge/branch hygiene**: all grade-A branches are merged; several stale
  worktrees under `~/.claude/worktrees/` can be reaped.

### 6.4 The release evidence track (the actual GA gate)
Everything below exists as tested scripts/workflows and none has executed:

1. Tag `v1.0.0-rc.1` → `candidate-certification.yml`
2. 3 external installs on supported platforms (`certify-release-candidate.sh`)
3. 1-week soak (`certify-operational-resilience.sh`)
4. Signed artifact publication (`release.yml` + `supply-chain.yml`)
5. Public verification from a clean machine (README cosign/gh-attestation commands)
6. No-rebuild GA promotion (`promote-release-candidate.sh`)

## 7. Final verdict

| Question | Answer |
| --- | --- |
| Implementation complete? | **Yes — 25/25, all merged** (verified `main` @ `ed44924`) |
| Composite vs 2026-07-13 | 3.5 → **≈4.3 / 5** (single rubric, self-assessed) |
| Can customers use it today? | **Design partners / early access on a tagged RC: yes.** The code is production-quality and the quality gates are real. |
| Can it be called GA? | **Not yet.** The release evidence track (§6.4) has not run. Calling it GA before external installs + soak + verified signed artifacts would be asserting evidence that does not exist. |
| Critical pre-customer changes? | One accuracy fix (§6.1 SLSA wording) before security collateral ships; the unwrap sweep (§6.2) strongly recommended; the rest is operator/process, not code. |
| Single biggest risk | Skipping the evidence track because the implementation feels done. The 2026-07-13 plan classified these as non-implementation for a reason: they are the only proof a customer can independently check. |

## 8. Corrections applied to this report's own method

1. **Absence claims now require a fresh grep/ls against `main`**, not
   memory of a worktree. Eleven draft claims failed this test.
2. **One composite number, one rubric.** The draft's 4.2-vs-4.4 confusion
   came from mixing the 9-area grades with a 14-row competitive matrix.
3. **Self-assessed grades are labeled as such.** Letter grades here are a
   management summary; the external-verifiable artifacts (CI runs, signed
   attestations, published benchmarks) are the actual evidence.
4. Linguistic sycophancy scan (S-01..S-08) returned 0.0 on the verdict
   text; the substantive risk was factual drift, addressed by §1–§2.

## 9. Sources

### Repo (verified 2026-07-15 against `main` @ `ed44924`)
`git log` PRs #112–#131 · `src/uar/error.rs` · `src/config.rs` ·
`src/server.rs:819` (security.txt route) · `docs/adr/` (12 files) ·
`docs/coverage-baseline.md` · `docs/cookbook/` · `.vale.ini` ·
`fuzz/fuzz_targets/` (4) · `proptest!` in `rag/retrieval.rs`,
`governance/engine.rs`, `domain/prompt_caching.rs` ·
`.github/workflows/` (23 files; `supply-chain.yml` lines 214/221/245/278
for attest/attest-sbom, 232–241 cosign sign, 311–330 independent verify;
`ci.yml:200` Offline Reproducible Source) ·
`frontend/packages/{a2ui-core,a2ui-react,a2ui-uar,a2ui-lit,a2ui-svelte,a2ui-inspector}/` ·
PR #131 merge diff (`i18n.tsx`, `SurfaceErrorBoundary.tsx`,
`accessibility.test.tsx`, `ux.test.tsx`, theme tokens)

### External (web, July 2026)
- [CrewAI changelog](https://docs.crewai.com/en/changelog) and
  [CrewAI release notes](https://releasebot.io/updates/crewai) — 1.14.6
  (2026-05-28); pluggable memory/knowledge/RAG/flow backends (2026-06-11)
- [Best AI Agent Frameworks 2026 (Alice Labs)](https://alicelabs.ai/en/insights/best-ai-agent-frameworks-2026)
- [GitHub Blog — Reach SLSA Level 3 with Artifact Attestations](https://github.blog/enterprise-software/devsecops/enhance-build-security-and-reach-slsa-level-3-with-github-artifact-attestations/)
- [GitHub Docs — artifact attestations + reusable workflows for SLSA v1 Build L3](https://docs.github.com/actions/security-guides/using-artifact-attestations-and-reusable-workflows-to-achieve-slsa-v1-build-level-3)
- [slsa-framework/slsa-github-generator](https://github.com/slsa-framework/slsa-github-generator)

---

*End of validated post-grade-A re-assessment. 25/25 changes merged. The
implementation phase is closed. The remaining work is one wording fix, one
hygiene sweep, four operator decisions, and the execution of a release
evidence track that already exists in full.*
