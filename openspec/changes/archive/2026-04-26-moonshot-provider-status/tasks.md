## 1. OpenSpec Artifacts

- [x] 1.1 Create the proposal for Moonshot provider-status closure.
- [x] 1.2 Create the design for additive catalog diagnostics.
- [x] 1.3 Create the provider diagnostic status spec delta.
- [x] 1.4 Create this task checklist.

## 2. Backend

- [x] 2.1 Add provider catalog diagnostic status classification.
- [x] 2.2 Include additive `status` and `status_detail` fields in `/api/catalog` provider summaries.
- [x] 2.3 Add deterministic tests for credential-blocked and configured provider status.

## 3. Frontend

- [x] 3.1 Add provider status fields to catalog summary types.
- [x] 3.2 Render credential-blocked status in the providers list/detail surfaces.
- [x] 3.3 Preserve secret-safe rendering and avoid displaying API key values.

## 4. Verification

- [x] 4.1 Run focused backend provider-status tests.
- [x] 4.2 Run frontend typecheck.
- [x] 4.3 Run frontend lint.
- [x] 4.4 Run `openspec validate moonshot-provider-status --strict`.
- [x] 4.5 Run `openspec validate --changes`.
- [x] 4.6 Record KBD and refiner evidence.

## 5. Archive

- [x] 5.1 Sync the provider diagnostic status spec.
- [x] 5.2 Archive `moonshot-provider-status`.
- [x] 5.3 Advance KBD execution to runtime-console archive readiness.
