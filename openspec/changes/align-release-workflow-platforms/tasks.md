## 1. Workflow truth
- [x] 1.1 Replace Node 18/Bun with pinned Node 22/pnpm and current lockfiles/commands.
- [x] 1.2 Replace `--all-features -D warnings` with authoritative release bundles and warning policy.
- [x] 1.3 Remove Redis/test-config/static-path assumptions not present in supported deployment.
## 2. Platform jobs
- [x] 2.1 Add Linux x64/arm64 build, archive install, startup and health tests.
- [x] 2.2 Add macOS x64/arm64 build, archive install, startup and health tests.
- [x] 2.3 Add Windows x64 build, archive install, startup and health tests.
- [x] 2.4 Drop or mark preview any platform that cannot pass.
## 3. Verify
- [ ] 3.1 Dispatch candidate workflow from a non-GA test tag.
- [ ] 3.2 Retain logs/artifacts and validate OpenSpec.
