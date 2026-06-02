# Plan: uar-production-readiness-gaps

**Phase start:** 2026-06-02
**Backend:** OpenSpec (`openspec/changes/<id>/{proposal,tasks}.md`)
**Strategy:** 4 focused changes this phase (C1–C4); C5 partial-deferred; C6 → follow-on phase
**Evolver:** none — active evolution `uar-production-readiness-2026-04` has only an assessment + state.json, **no plan.json**, so there is no evolver plan to bridge. No `evolver-bridge.json` written.
**Product decision (R3):** Config authority = **env/YAML authoritative on every boot** (confirmed by user). UI-only provider/model edits do not persist across restart; write-back-to-YAML deferred to P3.

---

## Change Roster

| # | Change ID | Goals | Priority | Depends on | In this phase? |
|---|---|---|---|---|---|
| C1 | `fix-worker-pool-graceful-shutdown` | G1,G2,G3 | **P0** | — | ✅ Yes |
| C2 | `make-config-authoritative-on-boot` | G4 | **P0** | — | ✅ Yes |
| C3 | `persist-builtin-agents` | G5 | **P1** | — | ✅ Yes |
| C4 | `wire-runtime-console-events` | G6, partial G7 | **P1** | C1 (hook bus) | ✅ Yes |
| C5 | `add-otel-agent-tracing` (observability) | G7 | P2 | C1, C4 | ⚠️ Deferred-partial — scaffold only if time remains |
| C6 | parity backlog (cancellation-through-tools, resumable streams, durable workflows, evals, guardrails) | G7 | P3 | C1, C4, C5 | ⛔ Deferred → new phase `uar-harness-parity` |

C1 and C2 are independent and both P0 — they can run in parallel worktrees. C4 depends on C1's parking-lot `Hook`/`LifecycleEvent` bus. C3 is independent.

---

## Execution Order

```
C1 fix-worker-pool-graceful-shutdown  (P0, blocking) ──┐
C2 make-config-authoritative-on-boot  (P0, parallel) ──┤
C3 persist-builtin-agents             (P1, parallel) ──┤
                                                        └─→ C4 wire-runtime-console-events (P1, needs C1 hook bus)
                                                              └─→ (C5 add-otel-agent-tracing, if time)
                                                                    └─→ C6 → next phase
```

**Critical path (minimum releasable):** C1 → C4 (clean shutdown + a Runtime Console that isn't a facade).
**Full phase:** C1 ∥ C2 ∥ C3 → C4 → (C5 partial).

**Rationale for ordering:** C1 is the blocking defect (can't shut down) and unlocks the hook bus C4 needs — it goes first. C2 fixes the most visible day-to-day lie (UI not reflecting configured providers/models) and is independent, so it runs alongside. C3 is small and high-user-value. C4 is the largest and depends on C1.

---

## Recommended Agent Assignment

| Change | Agent | Notes |
|---|---|---|
| C1 §1 bump + compile-fix | claude-code | Cargo edit + `PoolError` match-site fixes (CR-05 taxonomy) |
| C1 §2–§4 shutdown wiring | claude-code | `server.rs` startup + `ingestion_worker.rs`; consult `async-patterns` + `axum-patterns` skills |
| C2 config | claude-code | `config.rs` + `settings/manager.rs` + `registry.rs`; regression tests |
| C3 backend seed | claude-code | `defaults.rs` idempotent upsert |
| C3 frontend selector/store | claude-code | **UI/UX routing applies** — run UI/UX Pro Max + `/impeccable audit` per CLAUDE.md before editing `agent-selector.tsx` / chat store |
| C4 backend events + approvals | claude-code | `runtime/manager.rs` + new approvals endpoint; reuses C1 hook bus |
| C4 frontend Approve/Deny | claude-code | **UI/UX routing applies** — wire dead buttons; gate un-backed panels |

> UI/UX work in C3/C4 must follow the CLAUDE.md "UI/UX work routing" block (memory recall → UI/UX Pro Max → `/impeccable audit`+`critique` → frontend-design/ux-designer → summarize → code).

---

## Risk Notes

1. **R1 — CR-05 error taxonomy breakage (C1):** the new `#[non_exhaustive]` `PoolError` may break match sites. Mitigation: land bump + compile-fix as the first commit so breakage is isolated.
2. **R2 — feature gating (C1):** confirm `shutdown`/`cancellation`/`hooks` are available under `features=["tokio-runtime"]`; add a flag if gated.
3. **R3 — config behavior change (C2):** env/YAML-authoritative-on-boot will overwrite DB-drifted provider/model config on existing deployments. Ship a migration note; this is the accepted, user-confirmed tradeoff.
4. **R4 — Runtime Console scope (C4):** a full `Runtime*` model is large. Ship a minimal real subset (runs/steps/tool-calls/approvals) and **gate** the rest rather than fake-complete. Do not leave dead panels.
5. **Cross-cutting:** C1 and C4 share the hook bus contract — define `LifecycleEvent → RuntimeEntity` mapping once in C1 to avoid rework in C4.

---

## Phase Success Criteria

- [ ] **C1:** SIGTERM drains in-flight ingestion within `shutdown_timeout_secs` and exits cleanly — no `kill` required; only one worker pool constructed; cancellation interrupts a wedged job.
- [ ] **C2:** Setting `UAR_LLM__MODEL` / `OPENAI_API_KEY` / YAML `providers:` is reflected in the providers-and-models UI **after a restart on a non-empty DB**; `configured` implies a usable key; configured-models shows the operator's selection, not the full catalog.
- [ ] **C3:** both built-in agents appear in the admin list and chat selector on a fresh DB, survive realtime ChangeSets, and can actually be selected and chatted with (no silent fallback).
- [ ] **C4:** a live chat run populates Cockpit/Runs; Approve/Deny buttons perform a real HITL gate; no admin panel ships a dead empty state in prod (gated instead).
- [ ] `cargo build --release` + `cargo clippy` + `cargo test` green; frontend tests green.
- [ ] Each change verified via `/opsx:verify` before archive.

---

## Out of Scope (explicitly deferred)

- **C5 observability** (OTel/OTLP tracing, token/latency/cost dashboards, Prometheus export wiring) — scaffold only if phase time remains; otherwise first item of the next phase.
- **C6 parity backlog** — cancellation propagated through *tool calls* (not just ingestion), resumable streaming, durable/checkpointed workflows, evals, prompt-injection guardrails, gateway support. → new phase **`uar-harness-parity`**.
- **Config write-back** — persisting admin-UI provider/model edits back to YAML (P3 follow-up to C2).
- Credentials admin UI (carried over from prior phase; separate phase `uar-credentials-admin-ui`).

---

## Next Step

Run **`/kbd-execute uar-production-readiness-gaps`** to select an execution backend and dispatch. Start with C1 (blocking, unblocks C4). C2 and C3 may dispatch in parallel worktrees.
