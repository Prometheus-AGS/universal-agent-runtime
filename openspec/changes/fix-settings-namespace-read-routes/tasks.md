## 1. Workflow and Source Baseline

- [ ] 1.1 Merge `origin/main` into the UAR review branch, preserve the four local commits plus the two operator-owned untracked files, pin `crates/prometheus-skill-system` to upstream commit `f1e58b25b0a9926c24d1bb0ddb6c0678d16c6f49`, and verify the merge graph and exact gitlink

## 2. Settings Read Contract

- [ ] 2.1 Convert namespace keys through the existing canonical slug function inside the GET wrapper and verify focused tests cover provider pluralization, Context Management hyphenation, unchanged server routing, and existing non-2xx propagation
- [ ] 2.2 Add a local installed-service Playwright config/spec for port 1906 and verify it observes only canonical provider and Context Management routes while rejecting settings namespace 404s

## 3. Local Certification

- [ ] 3.1 Run typecheck, lint, the focused API test, frontend boundaries, full frontend tests, production build, static-bundle validation, strict OpenSpec validation, and the locked `server-full` Rust release build; record every observed result locally

## 4. Installed Runtime Proof

- [ ] 4.1 Capture LaunchAgent state and canonical provider IDs/count, install through `packaging/native/macos/install.sh`, and verify liveness, readiness, unchanged provider IDs/count, canonical browser routes, rendered providers, Context Management, and zero settings namespace 404s

## 5. Closeout

- [ ] 5.1 Record the root cause, deployment evidence, and residual risk under `.prometheus/`; verify and archive the OpenSpec change; complete KBD Reflect while leaving the successor run active with `/kbd-new-phase` as exact next work; push the review branch without merging it
