# Decision Log — uar-uiux-full-migration-2026-08

### 2026-08-07 — Operator decisions at analyze entry (AskUserQuestion)

**D1 — Component primitive library.** Keep **Base UI**; amend Goal 4.
**AMENDED 2026-08-07 after adversarial review:** this is an OPERATOR OVERRIDE of the
KnowMe standard §6.1 and §6.3 (line 227), which name Shadcn UI as the REQUIRED owner of
general controls, navigation, overlays and sidebars. Recorded as a divergence in the
vendored standard header, not as compliance. Base UI is technically adequate; no claim is
made that it is less total work than shadcn (no effort comparison was performed).
Rationale: the Radix→Base UI migration already landed in code (0 Radix imports, 34 Base UI
files, `base-ui-foundation` 24/24). Goal 4's "shadcn" wording is superseded by fact.
Consequences: queued `base-ui-{composition-patterns,icon-migration,verification}`
(0/40, 0/28, 0/33) become this phase's work; 27 orphaned `@radix-ui/*` declarations pruned.
Provenance: user | Elicitation: AskUserQuestion, criticality high.

**D2 — Rebuild scope.** **Per-surface scoping**, not literal greenfield.
Rationale: ~49 UI-owning OpenSpec changes exist, several at 100% (a2ui-uar-renderer 49/49,
theming 20/20, inspector 21/21). Greenfield discards finished work plus 153 passing tests
against an already-red coverage gate (19.45%/60%).
Consequence: analyze must return a per-surface rebuild-vs-preserve matrix.
Provenance: user | Elicitation: AskUserQuestion, criticality high.

**D3 — Design authority materialization.** **Copy** `knowme-ui-ux-standard.md` into UAR
`docs/` with a provenance header.
Rationale: rank-1 authority was unresolvable in-repo; agents fell back to the plan's
paraphrase. Local copy makes drift visible in git.
Provenance: user | Elicitation: AskUserQuestion, criticality high.
