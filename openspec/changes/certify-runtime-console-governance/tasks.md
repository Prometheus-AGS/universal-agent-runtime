## 1. Governance semantics
- [ ] 1.1 Introduce `Allow | RequireApproval | Deny` through policy, runtime, API and event contracts.
- [ ] 1.2 Ensure Cedar deny cannot create or resolve an approval request.
- [ ] 1.3 Test permit, approval, deny, timeout, channel close and duplicate resolution.
## 2. Console architecture
- [ ] 2.1 Add runtime-console service/store/hooks; remove page fetch/direct graph mutation.
- [ ] 2.2 Consume the shared AG-UI adapter for live/replay state.
## 3. Certification
- [ ] 3.1 E2E Cockpit/Protocols/Runs/Approvals with live updates and errors.
- [ ] 3.2 Verify health, memory, routing, surfaces and artifacts are real/correlated.
- [ ] 3.3 Remove Console from boundary allowlist; run all gates.
