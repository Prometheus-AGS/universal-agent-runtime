## 1. Ownership
- [ ] 1.1 Add knowledge store actions for bases, documents, upload, search, delete, retry.
- [ ] 1.2 Rewrite knowledge hooks as store façades; remove service imports.
## 2. Behavior
- [ ] 2.1 Test loading/empty/error/auth/optimistic rollback/realtime reconciliation.
- [ ] 2.2 Add real create→upload→indexed→ranked-search→delete page E2E.
- [ ] 2.3 Extend chat BDD to prove retrieved content affects the answer.
- [ ] 2.4 Test invalid file, indexing failure, and retry.
## 3. Verify
- [ ] 3.1 Remove Knowledge from boundary allowlist; run all relevant suites and OpenSpec validation.
