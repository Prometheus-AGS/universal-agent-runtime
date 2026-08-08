# KnowMe §12 acceptance checklist

Run date: 2026-08-08

Classifications are intentionally narrower than a blanket phase pass. `Verified` means
repeatable evidence was produced in this phase. `Separately owned` means an earlier change
or release process owns the claim and C-15 does not recertify it. `Not applicable` records
a binding operator divergence or the absence of a UAR Flutter application. `Backend-bound`
identifies product behavior that cannot be proven by deterministic frontend mocks.

## Visual

| Statement | Classification | Evidence / disposition |
|---|---|---|
| No visible borders, divider lines, or layout shadows remain. | Separately owned | C-03 owns the Flat 2.0 migration boundary. Its fail-closed gate passes at 376 tracked legacy findings and 0 new findings; this is not represented as global zero. |
| Adjacent regions are distinguishable through approved background tokens. | Separately owned | C-02/C-14 own token and surface composition. The representative responsive matrix remained operable in all cells. |
| Light and dark modes use the KnowMe palette and pass contrast checks. | Verified | Eight light/dark matrix cells plus fail-closed Storybook axe; reproduced ember and faint-text failures were corrected. |
| Ember is restrained; cyan and status colors retain their meanings. | Separately owned | Semantic token ownership is C-02/C-05. C-15 changed only reproduced contrast/focus values and retained the role mapping. |
| Empty, loading, streaming, degraded, offline, and error states are intentionally designed. | Separately owned | State design is owned by the migrated feature surfaces and their unit/Storybook suites, not re-audited exhaustively by C-15. |

## Components and architecture

| Statement | Classification | Evidence / disposition |
|---|---|---|
| React uses Shadcn UI and Assistant UI. | Not applicable as written | Operator decision D1 deliberately uses Base UI instead of Shadcn; C-01 records this as a divergence, not compliance. Assistant UI remains the chat/thread authority. |
| Flutter uses token-driven/shadcn_flutter patterns. | Not applicable | This repository has no UAR Flutter application surface in the migration scope. |
| Prometheus Entity Management 3.x owns entity reactivity and mutations. | Separately owned | C-14 migrations and the entity/runtime unit suite own this contract. |
| Zustand contains transient UI state, not the durable conversation database. | Separately owned | C-11/C-14 own PGlite persistence and transient store boundaries. |
| Web conversations persist in PGlite; desktop conversations persist through Rust in pglite-oxide. | Separately owned | Web PGlite is covered by earlier persistence work; desktop Rust persistence is outside this React certification. |
| Visual components call hooks rather than stores, database clients, or Tauri invoke directly. | Verified | `node scripts/check-frontend-boundaries.mjs` passed with 0 production violations and negative fixtures rejected all 10 forbidden patterns. |

## Chat

| Statement | Classification | Evidence / disposition |
|---|---|---|
| A new user can send a prompt using a local model without provider configuration. | Backend-bound | The deterministic no-provider guard and basic chat specs pass. End-to-end local-model generation without provider configuration is not claimed by C-15. |
| Multiple conversations can be created, searched, resumed, renamed, and archived. | Separately owned | Conversation lifecycle and persistence are owned by C-11/C-14; the full unit suite passes. |
| Streaming handles structured text, thinking, citations, tools, memory, artifacts, and media. | Separately owned | C-10 owns the exhaustive chunk catalog and fail-closed component tests. |
| Markdown, Mermaid, sanitized SVG, images, and video have safe renderers and fallbacks. | Separately owned | C-07 through C-09 own the single sanitized Markdown pipeline and lazy renderers. |
| Refresh/relaunch restores history and per-thread drafts. | Separately owned | C-11 owns durable thread state; C-15 does not repeat persistence certification. |
| Raw transport/runtime errors never appear as assistant responses. | Separately owned | Chat stream/store tests own error projection; the complete unit suite passes. |

## Responsive and cross-platform

| Statement | Classification | Evidence / disposition |
|---|---|---|
| Review at 320, 768, 1024, and 1440 CSS pixels in both themes. | Verified | All eight required matrix cells passed. |
| Desktop uses sidebar/rail navigation; phone uses bottom navigation. | Verified | The matrix asserts the desktop rail at 1024/1440 and compact bottom navigation at 320/768, with the inactive mode absent. |
| Flutter goldens cover representative phone/tablet states in both themes. | Not applicable | No UAR Flutter application exists in this migration scope. |
| React and Flutter token hashes/parity checks pass. | Not applicable | There is no UAR Flutter token consumer against which to calculate parity. |
| Keyboard checks pass. | Verified | Desktop and compact tests traverse by Tab/Shift-Tab, exercise Enter and Escape, execute a selected palette command, cycle every compact-dialog action, and verify trigger focus return. |
| Screen-reader checks pass. | Separately owned | C-15 verifies semantic landmarks, accessible names, polite status content, and axe; a manual screen-reader usability session remains a release-level check and is not falsely reported as executed. |
| Reduced-motion checks pass. | Verified | The reduced-motion browser profile resolves non-essential transition duration to 1ms while retaining interaction. |
| Text-scaling checks pass. | Separately owned | The responsive and overflow matrix is complete, but a dedicated browser text-zoom visual review remains release-level work and is not reported as executed. |

## C-15 blocking result

No C-15-owned automated accessibility, keyboard, focus, target-size, landmark,
reduced-motion, high-contrast, or responsive-matrix requirement remains failed. The
separately owned and not-applicable rows above are retained so completion cannot be read as
a blanket claim over the entire product estate.
