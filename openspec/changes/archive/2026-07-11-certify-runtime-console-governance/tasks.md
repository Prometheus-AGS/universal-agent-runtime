## 1. Governance semantics
- [x] 1.1 Introduce `Allow | RequireApproval | Deny` through policy, runtime, API and event contracts.
- [x] 1.2 Ensure Cedar deny cannot create or resolve an approval request.
- [x] 1.3 Test permit, approval, deny, timeout, channel close and duplicate resolution.
## 2. Console architecture
- [x] 2.1 Add runtime-console service/store/hooks; remove page fetch/direct graph mutation.
- [x] 2.2 Consume the shared AG-UI adapter for live/replay state.
## 3. Certification
- [x] 3.1 E2E Cockpit/Protocols/Runs/Approvals with live updates and errors.
- [x] 3.2 Verify health, memory, routing, surfaces and artifacts are real/correlated.
- [x] 3.3 Remove Console from boundary allowlist; run all gates.
## 4. Default HTTP port
- [x] 4.1 Set the runtime default to configurable port `1906` and test override precedence.
- [x] 4.2 Align first-party development, deployment, SDK and operator contracts on port `1906`.
