# Exit contract — startup-config-lifecycle-root-cause (frozen before code)

Child of `function-first-release-certification`, descended 2026-10-19 14:14 UTC
@ revision 3094. Candidate of record: v4 sha256
`96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154`.

## Frozen conditions (all must be evidenced for PASS)

1. **One evidence-backed trace** of the startup configuration lifecycle with
   file/line citations, covering: file/env/CLI bootstrap; first-boot vs
   existing persisted DB rows; service construction and ownership ordering;
   REST update propagation; truthful activation/rollback reporting; restart
   hydration; embedding-space compatibility.
2. **One common-cause defect identified, or an explicit conclusion that no
   architecture change is justified** — each candidate competing authority and
   captured stale dependency adjudicated with a citation.
3. **At most one minimal implementation correction** (zero permitted),
   distinguished from fixture/reporting errors, with preserved interfaces
   named if any correction is made.
4. **One bounded real boundary verification** (already-executed P1–P4
   receipts qualify where they exercise the exact boundary) plus relevant
   focused regressions; no expanded exit tests, no new builds beyond the v4
   candidate already of record.
5. **Defect classes separated**: product defect vs fixture/reporting error
   (explicitly: the `KbConfigResponse` DTO omission of `indexed_embedding`,
   and the FastEmbed fixed-model behavior).
6. Verdict: **PASS** only if 1–5 are fully evidenced; otherwise **NOT READY**
   with exactly one concrete blocker named.

## Hard limits

No new scope, no broad redesign, no publication, no spending/provider
fallback, no security-surface changes, no manual KBD projection edits, no
second build unless condition 3 requires it (it currently does not — the
diagnosis evidence below is read-only). Parent certification state is
preserved; exit returns via canonical `kbd-child-exit.sh`.
