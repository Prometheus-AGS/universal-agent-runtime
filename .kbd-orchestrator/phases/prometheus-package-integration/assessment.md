# Assessment — `prometheus-package-integration` (resumed)

**Date:** 2026-07-06
**Tool:** claude-code (`/kbd-assess`)
**Prior state:** Phase last touched 2026-05-26 (original assessment/plan preserved at `assessment-2026-05-26-original.md`). `progress.json` recorded `changes_completed: 10, changes_total: 14` but the phase was never execute/reflect-flagged and had no further activity for over a month of subsequent phases. Resumed at the user's explicit request (`uar-carryover-audit`'s "any more planned features left in KBD?" investigation surfaced it as the clearest genuinely-abandoned-mid-flight phase in the whole 40-phase history).
**Method:** direct code inspection of all 14 originally-planned changes — no trust in the stale `10/14` count, per this project's now-established lesson (repeated across `uar-frontend-typecheck-cleanup` and `uar-carryover-audit`) that carried completion claims must be re-verified, not assumed.

---

## 1. Verification of the original 14 changes

| # | Change | Status | Evidence |
|---|--------|--------|----------|
| 1 | `fix-kb-document-count` | **DONE** | `KnowledgeBaseResponse.document_count` present, `src/uar/api/knowledge.rs:77` |
| 2 | `surreal-live-query-bus` | **DONE** | `src/uar/realtime/{mod,surreal_bus,postgres_bus}.rs` — both backends implemented, not just Surreal |
| 3 | `add-skill-kind-and-origin` | **DONE** | `SkillKind`/`SkillOrigin` enums + `kind`/`origin` fields, `src/uar/domain/skills.rs` |
| 4 | `binary-instance-discovery` | **PARTIAL — real gap, redesign requested** | `src/uar/orchestrator/process_supervisor.rs` exists (201 lines) but only implements "probe port → adopt if answering, else spawn" — it assumes the binary is **already present on `PATH`**; there is no installation/provisioning logic at all. Pidfile path (`$XDG_RUNTIME_DIR`/`~/.uar/run`) is Linux/Unix-idiomatic, not cross-platform. See §3. |
| 5 | `add-skill-system-submodule` | **DONE** | `crates/prometheus-skill-system/` submodule present; `src/uar/runtime/skills/builtin_loader.rs` walks `SKILL.md` manifests |
| 6 | `wasm-component-skill-runtime` | **DONE** | `src/uar/runtime/skills/wasm_runtime.rs` uses `wasmtime::component::{Component, Linker}`; `wit/uar-skill.wit` + `wit/uar-plugin.wit` both present |
| 7 | `frontend-pnpm-workspace-migration` | **DONE** | Confirmed extensively this session (`uar-frontend-typecheck-cleanup`) — `frontend/pnpm-workspace.yaml`, root scripts use `pnpm -C frontend` |
| 8 | `add-entity-management-submodule` | **DONE** | `frontend/packages/prometheus-entity-management/` submodule present (`git submodule status` confirms, v1.2.0-rc-26-g79b2a62) |
| 9 | `configure-entity-engine-and-realtime-bridge` | **DONE** | `frontend/src/lib/entity-engine.ts` calls `configureEngine(...)`; `frontend/src/lib/realtime/` adapter present |
| 10 | `migrate-admin-knowledge-to-entity-mgmt` | **DONE (evolved design)** | `knowledge-page.tsx` uses `useKnowledgePage` from `@/entities/hooks/use-knowledge-page` — a direct per-entity hook, not the plan's originally-envisioned generic `useEntityList("knowledge_base")` call. Functionally equivalent; delivered via the later `direct-entity-migration-*`/`migrate-*-page-direct-*` phase lineage instead of under this phase's own name. |
| 11 | `migrate-providers-models-agents-to-entity-mgmt` | **DONE (evolved design)** | `agents-page.tsx` uses `useAgents` from `@/entities/hooks/use-agents` (confirmed directly — this session edited this exact file for `ch06`). Same direct-hook pattern as #10; providers/models pages follow the same convention per the `direct-entity-migration-{providers,models}` phases already in this repo's history. |
| 12 | `builtin-skills-ui-affordance` | **DONE** | `skills-page.tsx:317+` — `isBuiltin` check disables edit/delete with tooltip `"System skill — cannot be edited/removed"`, exactly matching the original spec |
| 13 | `dockerfile-multistage-with-submodules` | **DONE — CORRECTION, see below** | An initial quick check (`grep -c "^FROM"` = 3) was wrongly read as "not the polyglot design." Direct content inspection shows the 3 stages *are* exactly the planned `toolchain` → `builder` → `runtime` design: Rust (via `/usr/local/cargo`), Go (`/usr/local/go/bin` on `PATH`), Node, Python, and the `wasmtime` CLI are all installed in the `toolchain` stage; all 5 planned volumes (`skills-user`, `skills-derived`, `cache/{huggingface,cargo,pnpm}`) are declared. **This item is fully done, not a gap.** |
| 14 | `integration-tests-and-docs` | **PARTIAL, needs follow-up** | No dedicated tests found for live-bus latency or the builtin-delete-409 contract specifically (a quick search found no match); given items 1-13 are otherwise done, this item's remaining scope is really "cover whatever #4 becomes," not the original list verbatim. |

**Net: 12 of 14 already done** (several via an evolved, arguably-better direct-hooks design than originally spec'd — worth noting as a positive divergence, not a deviation to correct). **Only #4 is a genuine, unfinished gap.**

**Important correction to this session's framing**: earlier in this session, before this deeper check, #13 was believed to be an unfinished gap alongside #4, and the user directed a redesign of "both" away from the polyglot-toolchain/sidecar approach toward a git-install-based one. That framing was based on the same flawed quick check. **#13 is not unfinished work — it is already-shipped, working infrastructure**, wired into 5 `docker-compose*.yaml` files and 5 GitHub Actions workflows (`build-image.yml`, `deploy.yml`, `release.yml`, `comprehensive-tests.yml`, `image-uar-toolchain.yml`). Replacing it is not "completing an unfinished plan" — it is **deliberately replacing working, CI/deploy-wired production infrastructure** with a different, leaner design. That is a legitimate thing to want, but it is a materially different (and materially riskier) decision than filling a gap, and needs to be confirmed as such before `/kbd-plan` scopes it — see the handoff note.

## 2. Scope change directed by the user this session

The user initially directed a redesign of both #4 and #13, believing both were unfinished. §1's correction shows #13 is already shipped; the redesign below now applies **to #4 only**, pending the user's explicit re-confirmation on whether they still want to touch #13's already-working, CI-wired image:

- **Reject**: `process_supervisor.rs`'s current "assume the binary is already on `PATH`" gap, and the Dockerfile's "always pre-bake every toolchain" approach — both are inflexible and, for the Dockerfile, bloat the runtime image with build tooling it may never use.
- **Adopt instead**: a **pluggable provisioning-strategy system**. For each managed dependency (the 3 child binaries **and** the 5 skill-compilation toolchains), resolve via, in order:
  1. **Adopt** — already-running (binaries, via the existing port-probe+health-check) or already-installed (toolchains, via `which rustc`/`which node`/etc.) — reuse, don't reinstall.
  2. **Native package manager** — `apt`/`dnf` (Linux), `brew` (macOS), `winget`/`choco` (Windows) — when available for the current OS.
  3. **Git install** — clone the tool's own repo (submodule or on-demand) and build from source. Portable fallback; matches the pattern `prometheus-skill-system`/`prometheus-entity-management` already use as submodules.
  4. **Prebuilt binary** — download a release artifact (e.g. `wasmtime` ships prebuilt binaries for all 3 OSes) where that's faster than building from source.
- **Scope confirmed via `AskUserQuestion`**: covers exactly the 3 child binaries (MCP servers, `surreal-memory-server`, `liter-llm` proxy) + 5 skill-compilation toolchains (Rust, Node, Python, Go, wasmtime). **Database engines (SurrealDB/Postgres) are explicitly out of scope** — assumed pre-provisioned by whoever deploys UAR.
- **Cross-platform requirement**: equally robust on Linux, macOS, and Windows — confirmed explicitly, not just "Linux primary, others best-effort."

## 3. Design implications for #4 and #13

### #4 — `process_supervisor.rs` redesign

- Current `Supervisor::supervise()` takes a caller-provided `spawn_cmd: FnOnce() -> Command` closure that assumes the binary already exists. This needs a **provisioning step inserted before adoption/spawn**: given a `ManagedBinary` or a toolchain identifier, resolve *how* to get a runnable binary (one of the 4 strategies above) before attempting to spawn it.
- The pidfile mechanism (`$XDG_RUNTIME_DIR`/`~/.uar/run`) needs an OS-aware equivalent or a cross-platform library (the existing `dirs` crate already in use provides `dirs::home_dir()`; a Windows-appropriate runtime-dir equivalent needs sourcing — `dirs::runtime_dir()` returns `None` on Windows/macOS today per the crate's own docs, so a fallback to a per-OS convention is needed regardless of provisioning-strategy work).
- Existing `AdoptionResult`/`ManagedBinary`/`Supervisor` types are reasonable and should likely be *extended*, not replaced — the "already running, reuse it" case works today and shouldn't regress.

### #13 — Dockerfile: hold, pending explicit re-confirmation

Given #13 turned out to be already-shipped, CI/deploy-wired infrastructure (not an unfinished gap), this assessment does **not** scope a Dockerfile redesign by default. If the user still wants to replace the polyglot always-baked-toolchain image with on-demand provisioning after seeing this correction, that should be its own explicitly-confirmed, separately-risk-assessed decision — it touches `build-image.yml`, `deploy.yml`, `release.yml` directly, not just a source file. Left out of this phase's plan unless re-confirmed.

## 4. Spec Gap Summary

No canonical spec currently documents a "how UAR provisions its own runtime dependencies" contract. This phase's `integration-tests-and-docs` (#14) should produce one, given the strategy-selection logic is novel enough that future maintainers will need to understand the adopt → package-manager → git-install → prebuilt-binary precedence and how to add a new managed dependency to it.

## Goal Progress

| Goal | Status | Reason |
|---|---|---|
| Original 3 coupled goals (KB doc-count fix, entity-management embed, skill-system embed) | **MET** | Confirmed via direct code inspection — all of #1-3, #5-12 done. |
| Redesigned goal: cross-platform pluggable provisioning for the 3 child binaries | **NOT MET** | This is the actual remaining work — `process_supervisor.rs` needs the provisioning layer; tests/docs need to follow. Whether the Dockerfile is *also* touched is a separate, not-yet-reconfirmed decision (see §3). |

## Sycophancy self-check

- S-02: the "12/14 already done" finding is stated plainly, including that several were delivered via a *different* (arguably better) design than originally planned — not framed as "the plan was followed," which would be inaccurate.
- S-03: explicit trade-off surfaced — pidfile-based process tracking doesn't have a clean Windows equivalent via the `dirs` crate already in use; this needs a real decision during `/kbd-plan`, not glossed over.
- S-05/S-06 (truth over fluency, evidence before conclusions): this assessment's own first pass got #13 wrong (assumed "not done" from a shallow `grep -c` check) and is corrected here with direct evidence rather than left standing — the correction is the more important finding than the original mistake.
- S-07: no scope creep — the redesign scope (3 binaries + 5 toolchains, no DB engines) was explicitly confirmed with the user via `AskUserQuestion`, not assumed or expanded unilaterally. The #13 correction *narrows* scope rather than expanding it.

ASSESSMENT COMPLETE
