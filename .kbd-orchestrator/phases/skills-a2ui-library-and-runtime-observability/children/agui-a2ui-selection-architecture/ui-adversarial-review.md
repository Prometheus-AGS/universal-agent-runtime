# Fresh-context design review — 2026-09-04

Reviewer: presentation_ui_adversarial. Artifact-only/source review; no prior critique reports or generation history. No code, tests or browser activity.

Result: bounded acceptance of catalog/editor and inert preview, with two contract gaps. First, the existing graph uses a shared persistent key and the inspected auth transport uses a build-time token; the plan had assumed a reusable auth-reset mechanism that does not exist. Second, global pending-action replay conflicts with the plan's prohibition on uncertain write retries.

Resolution in ui-plan.md: use the existing configured credential without trusting decoded hints; obtain a host-derived principal key even for an empty catalog, gate hydrated rows/drafts behind fresh admission, filter exact owners, and reject stale responses across admissions. Execute template writes once outside the pending-action queue, then ingest confirmed records. The plan also now explicitly covers shell navigation and browser exit for unsaved drafts.

Acceptance does not cover later assignment/selection UI. Those need their own task-specific interaction contract and review. It does not constitute visual acceptance or evidence that these mechanisms have been implemented.
