# Historical Document Inventory

This inventory prevents point-in-time designs and assessments from being mistaken for current release contracts. Canonical replacements are [README](../README.md), [architecture](ARCHITECTURE.md), [frontend architecture](frontend-architecture.md), and the [product support matrix](product-support-matrix.md).

| Material | Historical reason | Treatment |
|---|---|---|
| `docs/htmx/` | Standalone HTMX/Web Component prototypes from the retired first-party UI direction | Directory-level historical marker; artifacts retained as research |
| `docs/htmx-docs/` | Rendered point-in-time RFC and assessment artifacts | Directory-level historical marker; artifacts retained for provenance |
| `docs/full-implementation/` | Exploratory implementation prompts, including no-React and HTMX proposals | Directory-level historical marker; not a release contract |
| `CLAUDE_ASSESSMENT.md`, `CODEX_ASSESSMENT.md` | Dated tool assessments of the former UI | Supersession banner |
| `OPENCODE_ASSESSMENT.md` | Dated tool assessment of the former UI, plus a retracted testing claim: its "30+ API test cases" counts declarations in `tests/integration/api/comprehensive.rs` whose tests issue no HTTP request | Supersession banner + inline retraction of the testing and frontend claims |
| `STATE_MANAGEMENT.md` | Alpine/localStorage/Web Component state design | Supersession banner |
| `THEME_AND_MOBILE.md` | Former Web Component theme/mobile proposal | Supersession banner |
| `COMPREHENSIVE_TESTING_INFRASTRUCTURE_SUMMARY.md` | Unverified point-in-time completion claims | Supersession banner |
| `TAURI_STRATEGY.md` | Design proposal rather than current platform certification | Historical-design banner |

Files in `docs/assessments/`, `docs/plans/`, and `docs/future/` are dated inputs or proposals by directory contract. They may describe targets that were never shipped and must not be cited as current capability evidence.
