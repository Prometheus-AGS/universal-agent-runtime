# Handoff out — uar-outstanding-issues-v1-1-1› function-first-release-certification› startup-config-lifecycle-root-cause

**Status:** INCOMPLETE (projection-derived: `progress.json` completion tracking
is 0/0 PENDING — no tracked task ledger was registered for this read-only
diagnosis child. Generated projection left untouched per policy. The
evidence-backed verdict against the frozen EXIT-CONTRACT.md is **PASS** — see
reflection.md.)

## Deliverables

All at the canonical nested path
`.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/children/startup-config-lifecycle-root-cause/`:

- `EXIT-CONTRACT.md` — frozen before any code decision (6 conditions + limits)
- `ASSESS.md` — current-state inventory and constraints
- `ANALYZE.md` — full lifecycle trace (file/env/CLI → persisted DB → service
  construction/ownership → REST propagation → activation honesty → restart
  hydration → embedding compatibility) with file/line citations; adjudicated
  candidates a–h; defect-class separation
- `PLAN.md` — decision: no architecture change justified; zero corrections;
  preserved interfaces named; verification mapping with candidate attribution
- `EXECUTE.md` — condition→evidence binding; P1 attributed to v3, P2/P3/P4
  to v4 (corrected)
- `reflection.md` — delta-first review, verdict PASS

## Goal completion

See reflection.md. Finite goal met: the trace found no remaining competing
authority or captured stale dependency; the common cause (bootstrap ordering +
unowned authority transitions + dishonest activation reporting) is closed by
the v1–v4 boundary repairs with behavioral evidence. Corrections taken: zero
(≤1 permitted). No new builds, tests, deploys, or projection edits.

## Unresolved items

1. **Parent-owned:** query-time space-mismatch guard UNVERIFIED — needs a
   deployment with a second honored embedding space (bind KB to space A, swap
   to B, observe explicit query-time refusal). Parent gate stays NOT READY.
2. **Future build:** `KbConfigResponse` DTO omits `indexed_embedding`
   (`src/uar/api/knowledge.rs:89-97`) — reporting/observability error that
   produced phantom-null reads this session. One-field addition; not
   architecture.

## Recommendations to the parent (uar-outstanding-issues-v1-1-1› function-first-release-certification)

1. Resume the frozen R1–R23 reconciliation (R2(b–d), R10, R21–R23 prioritized)
   — no further architecture repair is a precondition; the boundary is closed.
2. Keep the configuration gate NOT READY with the single unwaived criterion
   exactly as recorded; do not accept any synthetic substitute for the
   second-space proof.
3. Schedule the DTO field addition with the next build that is otherwise
   justified; do not build for it alone.
