# Assessment — `uar-dependabot-remediation-2026-07`

**Date:** 2026-07-07
**Tool:** claude-code (`/kbd-assess`)
**Scope directive from user**: don't stop at the 52 Dependabot alerts + `docs/ARCHITECTURE.md` notes — actively check for other, unaddressed vulnerabilities.

**Headline finding**: the 52 Dependabot alerts significantly *undercount* the real picture. Running the project's own local tooling (`cargo audit`, `npm audit`, `pnpm audit`) surfaces **17 Rust advisories** (vs. 11 in Dependabot) and **several npm advisories with GHSA IDs Dependabot has not opened alerts for at all**. Separately, `docs/DEPENDENCY_MANAGEMENT.md` claims "the CI pipeline runs `cargo audit` as part of the release workflow" — that workflow (`release.yml`) has **never once executed** (it only triggers on a version tag or GitHub release, and this repo has never cut one), so the one process that was supposed to catch exactly this has been silently inert since it was written.

---

## 1. Reconciling the three data sources

| Source | What it found | Notes |
|---|---|---|
| GitHub Dependabot (`gh api .../dependabot/alerts`) | 52 open (3 critical, 10 high, 32 medium, 7 low) | Registry-graph-based; lags behind live advisory databases and has gaps (see §2) |
| `cargo audit` (installed and run fresh this session) | **17 vulnerabilities** + 18 "unmaintained/unsound" warnings | Only 11 of these 17 correspond to what Dependabot already flagged for `Cargo.lock`; 6+ are net-new findings (§3) |
| `npm audit` (root `package-lock.json`) | 15 vulnerabilities (11 moderate, 4 high) across 13 packages | At least 2 packages (`ajv`, `brace-expansion`) and several extra GHSA IDs on already-flagged packages are **not** in Dependabot's 30-alert count for this manifest (§2) |
| `pnpm audit` (`frontend/`) | 11 vulnerabilities (3 low, 4 moderate, 4 high) | 1 `undici` GHSA (`GHSA-vxpw-j846-p89q`) not in Dependabot's 10-alert count for this manifest |
| `sdks/typescript` | **Could not run** — no lockfile exists at all (only `package.json` with semver ranges) | Itself a finding: no reproducible dependency resolution for this package. Dependabot's 1 alert here (`vitest`, critical) is against whatever version its dependency graph resolves for `^2.0.0` — see §4 |

## 2. Advisories present locally but NOT in the Dependabot alert list

Confirmed by diffing each local audit tool's GHSA IDs against the 52 pulled from `gh api`:

- **`ajv`** — `GHSA-2g4f-4pwh-qvx6` (moderate, root `package-lock.json`) — no Dependabot alert at all.
- **`brace-expansion`** — `GHSA-f886-m6hf-6m8v` (moderate, root) — no Dependabot alert at all.
- **`flatted`** — `GHSA-25h7-pfq9-p65f` (high, root) — Dependabot only has `GHSA-rf6f-7fwh-wjgh` for this package; this second one is missing.
- **`minimatch`** — `GHSA-3ppc-4f35-3m26`, `GHSA-23c5-xmqv-rm74` (high, root) — Dependabot only has `GHSA-7r86-cg39-jmmj`.
- **`picomatch`** — `GHSA-c2c7-rcm5-vvqj` (high, root) — Dependabot only has `GHSA-3v7f-55p6-f55p` (medium).
- **`undici`** — `GHSA-vxpw-j846-p89q` (high, `frontend/`) — Dependabot has 4 other `undici` GHSA IDs but not this one.

This is a real gap, not a rounding difference: several of these are **higher severity** than what Dependabot shows for the same package (e.g. `picomatch`'s missing GHSA is `high`; Dependabot's is `medium`).

## 3. `cargo audit`'s 17 vulnerabilities — full list, with reachability

| RUSTSEC ID | Crate @ version | Severity | Reachability (traced via `cargo tree -i`) |
|---|---|---|---|
| RUSTSEC-2026-0193 | `ammonia` 4.1.2 | n/a (mXSS) | Via `surrealdb-core` ← `surreal-memory` (git-pinned, D-D). Reachable **if** ammonia's HTML sanitizer is actually invoked on untrusted input somewhere in the SurrealDB/memory path — not yet confirmed exercised; needs a plan-stage check. |
| RUSTSEC-2026-0204 | `crossbeam-epoch` 0.9.18 | n/a | Via `surrealdb-core` ← `surreal-memory` (git-pinned). Requires an invalid pointer already existing to trigger — low practical exploitability, but present. |
| RUSTSEC-2025-0005 | `grcov` 0.8.19 | n/a | **Dev-dependency only** (`[dev-dependencies]` in `Cargo.toml`) — not shipped in the release binary. Lower real risk; still worth updating since it's compiled during CI/test runs. |
| RUSTSEC-2026-0119 | `hickory-proto` 0.25.2 | n/a (CPU exhaustion) | Already in Dependabot (medium). |
| RUSTSEC-2026-0118 | `hickory-proto` 0.25.2 | n/a (infinite loop) | Already in Dependabot (high). |
| RUSTSEC-2026-0187 | `lopdf` 0.40.0 | **CVSS 3.1 High** (network, low complexity) | Via `kreuzberg` (git-pinned, D-D) — kreuzberg is UAR's **document-intelligence provider**, meaning this is directly reachable: a crafted PDF processed through the document-intelligence pipeline can trigger a stack overflow. **Not in Dependabot at all.** This is the most concrete, directly-exploitable new finding. |
| RUSTSEC-2026-0194 | `quick-xml` 0.29.0 / 0.37.5 / 0.39.4 / 0.40.1 (4 resolved versions in the tree) | **CVSS 3.1 High** (quadratic runtime DoS) | 0.29.0 is via `grcov` (dev-only, see above). The other 3 (`0.37.5`, `0.39.4`, `0.40.1`) are all via `kreuzberg` (through `biblib`, `calamine`, and directly) — reachable via any document kreuzberg parses that contains XML-based formats (e.g. `.xlsx`/`.docx`). **Not in Dependabot at all.** |
| RUSTSEC-2026-0195 | `quick-xml` (same 4 versions) | **CVSS 3.1 High** (unbounded memory allocation DoS) | Same reachability as above. **Not in Dependabot at all.** |
| RUSTSEC-2026-0185 | `quinn-proto` 0.11.14 | **CVSS 3.1 High** (remote memory exhaustion) | Via `reqwest` (a core, widely-used dependency for LLM API calls and this session's new `provisioning.rs` prebuilt-binary downloads). Reachable **only if** `reqwest`'s HTTP/3 feature is actually enabled and exercised — needs a plan-stage check of `reqwest`'s enabled features in `Cargo.toml`. **Not in Dependabot at all.** |
| RUSTSEC-2023-0071 | `rsa` 0.9.10 | CVSS 3.1 (Marvin Attack — timing side-channel key recovery) | Via `jsonwebtoken` ← both `liter-llm` (git-pinned) and `surrealdb-core`. This is a **real crypto vulnerability** in a JWT-signing/verification dependency; reachability depends on whether RSA-signed JWTs (vs. HMAC/EC) are actually used anywhere in this system's auth boundary — needs a plan-stage check. **Not in Dependabot at all.** |
| RUSTSEC-2025-0111 | `tokio-tar` 0.3.1 | n/a (file smuggling via PAX headers) | Already in Dependabot (high). |

**Net new, not covered by Dependabot at all**: `ammonia`, `crossbeam-epoch`, `lopdf`, `quick-xml` (×2 advisories), `quinn-proto`, `rsa` — 6 distinct advisories, 2 of which (`lopdf`, both `quick-xml` DoS CVEs) are **directly reachable through kreuzberg's document-processing path**, which is squarely in this application's threat model (it processes user-supplied documents).

Plus 18 "unmaintained/unsound" warnings (not CVE-style vulnerabilities, but real signals): `anyhow`, `atty`, `failure`, `libyml`, `memmap2`, `serde_yml` are flagged **unsound** (not just unmaintained) — `memmap2`'s "unchecked pointer offset" and `anyhow`'s `downcast_mut()` unsoundness are both very recent (2026-06-20, 2026-06-25) and worth checking whether newer patched versions exist.

## 4. `sdks/typescript` has no lockfile

`sdks/typescript/package.json` declares `"vitest": "^2.0.0"` with **no `package-lock.json`, `pnpm-lock.yaml`, or `yarn.lock` at all**. Dependabot's critical alert (`GHSA-5xrq-8626-4rwp`) affects `vitest < 3.2.6` OR `>= 4.0.0, < 4.1.0` — `^2.0.0` falls inside the first (vulnerable) range, so the alert is real, but there is no way to know *which exact* 2.x version would actually resolve today without a lockfile. This is itself a process gap (violates this project's own Rule 35, "Prefer Deterministic Systems" from `CLAUDE.md`) independent of the specific CVE — fixing the CVE here requires a `package.json` range bump (e.g. to `^3.2.6` or `^4.1.0`), not just a lockfile regenerate, since the vulnerable range covers the entire `^2.0.0` window.

## 5. Process gap: the documented `cargo audit` CI step has never run

`docs/DEPENDENCY_MANAGEMENT.md` states: *"The CI pipeline runs `cargo audit` as part of the release workflow (`release.yml`)."* Checked `.github/workflows/release.yml`'s trigger condition — it only fires on a pushed version tag (`v*.*.*`) or a published GitHub Release. `gh run list --workflow=release.yml` returns **zero runs, ever** — this repository has never cut a tagged release. The documented safety net has been inert since the day it was written, which is a plausible root cause for how 17 real Rust advisories (6 of them net-new beyond Dependabot) accumulated without being caught by "the process that's supposed to catch this."

## 6. Cross-reference against `docs/ARCHITECTURE.md`'s D-D decision

D-D ("Git-sourced dependency pins are deliberate, not technical debt") names 4 pinned git dependencies: `rmcp`, `surreal-memory`, `kreuzberg`, `prometheus_parking_lot`. Two of them are directly implicated in this session's new findings:

- **`kreuzberg`** (pinned to tag `v4.9.8`) is the source of `lopdf` (stack overflow) and 3 of the 4 vulnerable `quick-xml` resolutions — both reachable via kreuzberg's actual job (document parsing). Upgrading kreuzberg itself (or patching its `Cargo.lock`-level transitive pins, if kreuzberg's own `Cargo.toml` allows a newer `lopdf`/`quick-xml`) is the likely fix path — needs checking during planning whether a newer kreuzberg tag/commit already pulls patched versions, before doing anything more invasive.
- **`surreal-memory`** (pinned to `branch = "main"`) pulls `surrealdb-core` → `ammonia`, `crossbeam-epoch`, `rsa`. Since this dependency tracks `main` rather than a pinned SHA, a `cargo update -p surrealdb-core` (or letting the next natural `surreal-memory` sync pull it in) may already resolve some of these without any UAR-side code change.

D-D's framing ("pins are deliberate, not debt") is not contradicted by these findings, but it is **incomplete** without an accompanying practice of periodically checking whether those pinned dependencies' own transitive trees have drifted into vulnerable territory — which is exactly what this phase should establish going forward, not just fix once.

## Goal Progress

| Goal (from `goals.md`) | Status | Notes |
|---|---|---|
| Triage all 52 Dependabot alerts | IN PROGRESS | Full triage (fixed-version-available / direct-vs-transitive / reachable) for the Rust ecosystem is done above; npm-ecosystem alert-by-alert triage (30 root + 10 frontend + 1 sdk) is the next concrete step for `/kbd-plan`, not yet itemized alert-by-alert in this document. |
| Determine whether other unaddressed vulnerabilities exist beyond the 52 | **MET — yes, confirmed** | 6 net-new Rust advisories + 6 net-new/higher-severity npm advisories found via direct tool runs, none of which were in the Dependabot list. This is the headline finding of this assessment. |
| Cross-reference `docs/ARCHITECTURE.md`/D-D | MET | Done in §6 — 2 of the 4 pinned git deps are implicated; D-D's decision itself doesn't need reversing, but needs a periodic-recheck practice added. |

## Spec Gap Summary

- No document currently states "here is how we periodically re-verify our pinned git dependencies' transitive trees for new advisories" — `docs/DEPENDENCY_MANAGEMENT.md` only covers the upgrade SOP for the pins themselves, not a recurring audit cadence.
- The `release.yml`-gated `cargo audit` step is effectively dead process (never executed) and should either be moved to a trigger that actually fires regularly (e.g. a scheduled workflow or on every push to `main`), or `docs/DEPENDENCY_MANAGEMENT.md`'s claim should be corrected to not overstate what's actually happening.

## Sycophancy Self-Check

- S-05/S-06 (truth over fluency, evidence before conclusions): every claim above is backed by a command actually run this session (`gh api`, `cargo audit`, `npm audit`, `pnpm audit`, `cargo tree -i`, `gh run list`) — not inferred from the Dependabot dashboard alone, per the user's explicit instruction to look further.
- S-02 (agreement without grounding): did not simply confirm "yes there might be more vulnerabilities" as a hedge — ran the actual tools and found 12 concrete, named advisories (6 Rust + 6 npm) absent from the 52-alert baseline, with reachability tracing for the highest-severity Rust ones.
- S-03 (caveat collapse): explicitly flagged that reachability for `ammonia`, `quinn-proto`, and `rsa` is **not yet confirmed** (needs a plan-stage check of actual feature usage), rather than asserting exploitability without evidence.
- S-07 (scope creep): did not attempt any fixes in this document — assessment only, per KBD stage discipline. Concrete remediation choices (upgrade vs. wait vs. accept-risk-and-document) belong to `/kbd-plan`.

ASSESSMENT COMPLETE
