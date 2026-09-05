---
target: frontend/src/features/chat/ui/run-inspector.tsx
total_score: 27
max_score: 40
na_heuristics:
p0_count: 0
p1_count: 1
target_identity: "file:/Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/features/chat/ui/run-inspector.tsx"
target_fingerprint: "sha256:92a6ead7c69048e8aec177a3d5ef2e231b7abde63d788c11a6924b6564ce30ad"
target_path: /Users/gqadonis/Projects/prometheus/universal-agent-runtime/frontend/src/features/chat/ui/run-inspector.tsx
timestamp: 2026-09-05T05-34-02Z
slug: frontend-src-features-chat-ui-run-inspector-tsx
---
Method: dual-agent (A: run_provenance_ui_a · B: run_provenance_ui_b)

The proposed Presentation section fits UAR's run inspector. It separates permission from evidence without adding another dashboard. Its main risk is temporal ambiguity: latest run details will sit beneath a selected-event heading.

| Heuristic | Design-contract score | Finding |
|---|---:|---|
| System status | 2/4 | Clarify latest-run versus selected-event scope. |
| Familiar language | 2/4 | Explain that admission means permission. |
| User control | 3/4 | Native disclosure and retry. |
| Consistency | 3/4 | Incumbent tokens and standard structure. |
| Error prevention | 3/4 | Missing evidence cannot imply success. |
| Recognition | 3/4 | Modes, outcome and revisions colocated. |
| Efficiency | 2/4 | Compact disclosure; execution unverified. |
| Minimalism | 3/4 | No additional card or decoration. |
| Error recovery | 3/4 | Explicit load error and retry. |
| Help | 3/4 | Explain publication versus display. |
| Total | 27/40 | Provisional contract assessment only. |

The detector returned[], exit0, for the existing inspector: zero findings or false positives. The new component does not exist yet, so this is not implementation or visual acceptance.

## Priority issues

- P1 — Time scope: label the section `Latest recorded details for this run.` Selecting an earlier event must not make a later publication appear contemporaneous. Suggested command: impeccable clarify.
- P2 — Permission wording: explain `Admitted output` as `Output permitted by the host; publication is recorded below.` Do not substitute actual output for the admitted mode. Suggested command: impeccable clarify.

Strengths: distinct missing/unsupported/failed/cancelled states, frozen revision receipts, and independent field subscriptions. Planned cognitive load is low—three main rows and one disclosure. Alex needs accurate incident timing; Jordan needs the permission explanation; Sam needs stable focus and restrained announcements. Calm inspection is appropriate; temporal ambiguity would undermine trust.

Minor implementation reminder: give the disclosure summary the same usable target size and visible focus as retry. Browser, keyboard, contrast and zoom checks remain deferred. Source-only scores do not certify implemented behavior or visual quality.

Questions skipped: 2 priority issues, both covered by bounded corrections within the approved scope.
