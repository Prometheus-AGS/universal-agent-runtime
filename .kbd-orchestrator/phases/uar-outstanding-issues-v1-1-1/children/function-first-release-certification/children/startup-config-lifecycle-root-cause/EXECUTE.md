# EXECUTE — bounded verification record (no new code)

No implementation was performed in this child (PLAN decision: zero
corrections). Execution consists of binding each frozen exit condition to its
already-executed, receipt-backed evidence and confirming nothing in the
diagnosis requires the one permitted correction.

## Condition → evidence binding

1. **Evidence-backed trace** — ANALYZE.md §1–7. Every lifecycle stage carries
   file/line citations verified by direct source reads this session
   (server.rs, settings.rs, manager.rs, providers.rs, vector.rs, swappable.rs,
   ingest.rs, fastembed.rs, knowledge.rs, policy.rs, runtime/manager.rs).
2. **Common-cause adjudication** — ANALYZE.md candidates a–h each resolved as
   NOT a competing authority / NOT a stale capture, with the mechanism cited.
   Conclusion: common cause closed by v1–v4 boundary repairs.
3. **≤1 correction** — zero taken. Interfaces named in PLAN.md verified
   unchanged in the dirty checkout (no edits this child).
4. **Bounded real boundary verification** — receipts
   `receipts/config-lifecycle-map-2026-10-19.md` §9. Attribution by candidate
   (corrected 14:20 UTC):
   - P1 (**v3, prior candidate — NOT v4**): 5/5 legs — seed preserve → test
     200 → keyless PUT + row intact → restart → test 200 (4523ms) →
     key-bearing PUT 503 explicit, row untouched. Reused as prior
     same-source/provider-boundary evidence: the provider mutation/restart
     boundary it exercises was not re-cut by the v4 delta (v4 changed only
     embedding honored-value activation + overlay check), so it remains valid
     boundary evidence — but it is never claimed as v4 execution.
   - P2 (**v4**): bogus backend 422 (row unchanged, retrieval 0.71 intact);
     unhonored model PUT 422 (row byte-identical); boot-overlay refusal of
     out-of-band bogus row; reset If-Match → `activation: live` → honored
     swap bge-small/384 → next-request search 0.6399.
   - P3 (**v4**): `rag.embedding_provider` 422 explicit pointer;
     `rag.chunk_size` 200 + persisted + `restart_required`, retained across
     two restarts.
   - P4 (**v4**): memory rows persisted + `restart_required`; v4 boot
     `MemoryService initialized` from rows (config `enabled: false`); live
     tools `active=true`; disable → restart → "Memory system disabled".
   - Binding: probe KB `{fastembed, bge-small-en-v1.5, 384}` per surreal KV
     readback 13:22 UTC (server on v3 at readback time; binding row carried
     forward unchanged through the v4 restart per P2 overlay evidence).
5. **Defect classes separated** — ANALYZE.md §Defect classes: 4 fixed product
   defects; 1 product limitation now honestly contracted (FastEmbed fixed
   model); 1 fixture/reporting error (`KbConfigResponse` DTO omission,
   `api/knowledge.rs:89-97`); 1 environment gap (query-guard proof).

## Deviation from plan

None. No builds, no tests, no deploys, no projection edits, no scope growth.
