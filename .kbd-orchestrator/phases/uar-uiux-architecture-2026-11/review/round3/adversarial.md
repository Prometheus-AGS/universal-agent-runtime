# Round 3 adversarial review: BLOCK (1 HIGH, 3 MEDIUM, 5 LOW); step and runner review PASS with limits

Patch d1c8062c… reviewed by source only.

| # | Severity | Finding | Triage |
|---|---|---|---|
| 1 | HIGH | Focus lost tabbing from a scrolled-away row | FIXED: `onBlur` removed (as a11y F1) |
| 2 | MEDIUM | R3-1 not established: 597 options mount in one task; configured rows and about 345 provider pills are not windowed; the test empties the configured list | LIMIT: R3-1 is not fixed until command 47 passes; on failure read the task attribution first |
| 3 | MEDIUM | BDD server falls back to `~/.uar/config.yaml`; with chat working, configured remote routes become reachable | FIXED: empty scratch `CONFIG_FILE` in the BDD config (confirmed in `src/config.rs:1830-1844`) |
| 4 | MEDIUM | Round 2 mutant exception does not cover this candidate | IN PLAN step 5; the inventory sentence is rewritten at regeneration |
| 5 | LOW | IDN `toHaveCount(1)` now sees only mounted rows | LIMIT, record in inventory |
| 6 | LOW | Non-S1 product-screen-validation depends on a row index under 9 | LIMIT, not a boundary command |
| 7 | LOW | Stale burst references; explicit profiles unchanged | LIMIT, reported to the operator |
| 8 | LOW | Seed checker looks for a missing v4 seed | SEPARATE defect |
| 9 | LOW | PICK-002 does not time dialog open and only partly times close | LIMIT, record in inventory |

Batch also applied: a11y F2, F3, F4, F6, F7, F10 and design D3. Deferred: a11y F5, design D2.
