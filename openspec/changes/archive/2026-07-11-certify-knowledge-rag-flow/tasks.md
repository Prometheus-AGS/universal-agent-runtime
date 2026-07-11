## 1. Ownership
- [x] 1.1 Add knowledge store actions for bases, documents, upload, search, delete, retry.
- [x] 1.2 Rewrite knowledge hooks as store façades; remove service imports.
## 2. Behavior
- [x] 2.1 Test loading/empty/error/auth/optimistic rollback/realtime reconciliation.
- [x] 2.2 Add real create→upload→indexed→ranked-search→delete page E2E.
- [x] 2.3 Extend chat BDD to prove retrieved content affects the answer.
- [x] 2.4 Test invalid file, indexing failure, and retry.
## 3. Verify
- [x] 3.1 Remove Knowledge from boundary allowlist; run all relevant suites and OpenSpec validation.
