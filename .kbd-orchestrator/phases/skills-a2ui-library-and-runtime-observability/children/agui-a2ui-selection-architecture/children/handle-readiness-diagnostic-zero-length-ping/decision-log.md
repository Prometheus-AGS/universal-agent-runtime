# Decision Log — handle-readiness-diagnostic-zero-length-ping

## D-001 · one-shot exact-frame continuation [plan · 2026-09-06]

**TL;DR:** Accept direct Text or exactly one empty Ping→masked Pong→Text; every other sequence stops.

**Why:** The accepted observation proves only one final, RSV-clear, unmasked, zero-length Ping. General frame handling would exceed the evidence and introduce behavior that this diagnostic does not need.

**Alternatives:** reusable WebSocket loop (too broad) · modify the accepted collector in place (rewrites diagnostic history)

**Learn more:** `plan.md` → Exact collector contract.

---

## D-002 · one non-resetting sign-in-exchange deadline [plan · 2026-09-06]

**TL;DR:** Sign-in write, first frame, Pong write and following Text share the existing 15-second response budget.

**Why:** Resetting a timeout after Ping would let peer progress extend the observation. The canonical spec already supplies the 15-second first-frame and 30-second outer limits.

**Alternatives:** one timeout per operation (permits deadline extension) · new larger budget (unsupported by evidence)

**Learn more:** `openspec/specs/readiness-latency-diagnostics/spec.md` → Rejected frame observation is single-attempt and header-only.

---

## D-003 · behavioral tests at the child phase boundary [plan · 2026-09-06]

**TL;DR:** Tasks 1–3 use Tier 0 checks; Task 4 runs the entire diagnostic-only synthetic suite.

**Why:** This follows the operator's test-timing constraint. The live attempt remains one-shot and non-mutating, but running it before behavioral fixtures is a deliberate residual risk.

**Alternatives:** focused tests after collector implementation (conflicts with operator timing) · omit behavioral tests (fails acceptance contract)

**Learn more:** `design.md` → Run the behavioral suite only at the child phase boundary.

---

## D-004 · sign-in is terminal, not readiness evidence [plan · 2026-09-06]

**TL;DR:** The collector ends at sanitized sign-in success/error and sends no `use`, query, health or readiness request.

**Why:** Crossing the Ping boundary proves collector compatibility only. Continuing would silently expand this change back into the broader latency investigation.

**Alternatives:** resume the old readiness sequence (new authority and scope required) · report sign-in as recovery (unsupported claim)

**Learn more:** `proposal.md` → Impact and `plan.md` → Trade-off and deferred work.
