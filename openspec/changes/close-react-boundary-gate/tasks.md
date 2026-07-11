## 1. Closure
- [ ] 1.1 Re-scan all production frontend files and classify remaining violations.
- [ ] 1.2 Move remaining I/O/mutation logic to owning stores/services.
- [ ] 1.3 Remove legacy allowlist; encode only asset/transport infrastructure exceptions.
## 2. Gate
- [ ] 2.1 Make checker blocking in PR/main/release CI.
- [ ] 2.2 Add negative fixtures proving each prohibited dependency direction fails.
## 3. Verify
- [ ] 3.1 Run lint/typecheck/tests/build and OpenSpec validation with zero production violations.
