# Goals — uar-post-dependabot-followup-2026-07

Seeded from `uar-dependabot-remediation-2026-07`'s `reflection.md`
(2026-07-08) — specifically its "§7 Next Phase Recommendations" and the
one goal that phase did not meet (Goal 4: re-affirm or revise the D-D
architectural decision).

## Why this phase exists

`uar-dependabot-remediation-2026-07` closed 8/8 changes and 3/4 goals, but
explicitly flagged one honest gap: two of `docs/ARCHITECTURE.md`'s D-D
pinned git dependencies (`kreuzberg`, `surreal-memory`) were directly
implicated by that phase's findings, yet D-D itself was never revisited —
and in the process of writing the reflection, D-D's text was found to be
**factually backwards** about which dependency floats (it says `kreuzberg`
tracks `branch = "main"`; `Cargo.toml` shows `kreuzberg` is actually
tag-pinned and `surreal-memory` is the one on a floating `branch =
"main"`). That phase also shipped a brand-new scheduled CI workflow
(`.github/workflows/security-audit.yml`) whose trigger has never actually
fired — only locally simulated, per that phase's own disclosure.

## Goals

1. **Correct `docs/ARCHITECTURE.md`'s D-D bullet.** Fix the
   kreuzberg/surreal-memory pin-type swap (kreuzberg = tag, not
   branch=main; surreal-memory = branch=main, not a SHA). This is a
   factual correction, not a decision by itself.
2. **Make an explicit, human-reviewed decision on `surreal-memory`'s
   `branch = "main"` pin.** It's the one dependency in D-D's list that
   actively undermines D-D's own stated "reproducible builds" rationale
   — every build resolves whatever the upstream `main` branch currently
   points to. Decide: pin to a fixed SHA (matching `rmcp` and
   `prometheus_parking_lot`'s pattern), or explicitly re-affirm the
   floating pin with a documented reason. Either outcome satisfies this
   goal — the point is that it's a deliberate choice, not silent drift.
3. **Verify `security-audit.yml` actually fires on GitHub.** Confirm a
   real `workflow_dispatch` run (or wait for/trigger the first scheduled
   Monday 06:00 UTC run) succeeds with the expected 4 jobs and exit
   codes, not just the local command simulation the prior phase relied
   on. If it doesn't fire as expected, that's the prior phase's root
   problem (a CI mechanism assumed correct but never actually run)
   recurring one layer up — fix it for real this time.
4. **Triage the 9 never-assigned unmaintained/unsound `cargo audit`
   warnings** (`atomic-polyfill`, `bincode`, `instant`, `number_prefix`,
   `paste`, `rustls-pemfile`, `ttf-parser`, `scc`, `proc-macro-error2`) —
   determine reachability for each (mirroring this project's established
   practice from the prior phase) and either fix, mitigate, or disclose
   with rationale in `docs/DEPENDENCY_MANAGEMENT.md`. Low urgency (none
   are CVE-style vulnerabilities), but currently invisible since
   `security-audit.yml` doesn't fail on them by design.

## Non-goals

- Re-litigating any of the 8 already-completed and archived changes from
  `uar-dependabot-remediation-2026-07` — those are closed.
- Deciding whether OpenSpec needs a lighter-weight schema for hygiene-only
  changes (raised in the prior phase's reflection as a process question)
  — that's a tooling/process decision for a human to weigh in on
  directly, not something to resolve unilaterally inside a KBD phase.
- Full dependency modernization beyond the specific items above.
