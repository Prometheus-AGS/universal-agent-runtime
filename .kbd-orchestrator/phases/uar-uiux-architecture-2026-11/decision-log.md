# Decision log — uar-uiux-architecture-2026-11

### 2026-11-19T19:06:35Z — Analyze: stack mode and build-vs-adopt calls
Mode: stack specified (versions.toml, frontend/package.json, operator decisions 2026-11-19). No stack contest.
Adopt: Base UI Combobox + @tanstack/react-virtual (pickers); arc-swap ConfigManager, jsonschema/schemars, notify (config); RealtimeBus Surreal LIVE / Postgres LISTEN/NOTIFY (push); Impeccable CLI (design context); PEM 5.0.1 as-is.
Adapt: PGliteWorker (seed and migrations move into worker init).
Reference: JSON Forms (renderer dispatch pattern). Reject: RJSF, comlink.
Build: 23 items in library-candidates.json build_required (round 2), plus 4 unresolved critical review findings in analysis.md.
Provenance: research (source inspection + registry + docs). Evidence: analysis.md, library-candidates.json.

### 2026-11-19T19:06:35Z — Analyze corrects the remote-realtime premise
The assessment's "no push with remote databases" was wrong at the route level: /api/live serves all backends. Gaps are per topic (conversation policy none; providers mis-topiced on Surreal; no models writer; team topics internal). Operator decision 3 stands; scope is topic fixes plus building and verifying the non-default postgres-backend feature.

### 2026-11-19 — Analyze accepted at review cap with unresolved findings
Two harness-native review rounds (BLOCK 5/9/3, then BLOCK 4/5/4). Round-2 findings are carried as "Unresolved review findings" in analysis.md. Recommendation BLOCKED on operator question 5 (multi-replica scope for realtime model changes).

### 2026-11-19 — Multi-replica scope decided; Analyze READY to spec
Operator: realtime model changes must be correct across replicas sharing one remote store. Build item 24 added (cross-replica refresh via server-side subscribers to the existing change feed). Recommendation changed from BLOCKED to READY to spec. Provenance: user.
