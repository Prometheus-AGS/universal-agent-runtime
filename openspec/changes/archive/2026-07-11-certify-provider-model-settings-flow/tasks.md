## 1. Layer migration
- [x] 1.1 Add provider/model/settings store actions; only stores import services.
- [x] 1.2 Make hooks subscription/action façades; remove page fetcher/service imports.
- [x] 1.3 Preserve UI-only selection/filter/draft state in hooks/components.
## 2. Contracts
- [x] 2.1 Test provider configure/default/remove/health success and failure.
- [x] 2.2 Test model catalog/filter/add/remove/default/compare and missing metadata.
- [x] 2.3 Generate settings namespace round-trip tests from schema; verify secret redaction and invalid values.
- [x] 2.4 Add real server Playwright configure→route journey.
## 3. Gates
- [x] 3.1 Remove these domains from layering allowlist.
- [x] 3.2 Run frontend tests/typecheck/build, Rust API tests, E2E, OpenSpec validate.
