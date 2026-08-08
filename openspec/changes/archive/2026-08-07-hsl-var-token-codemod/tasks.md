## 1. Scope and token contract

- [x] 1.1 Record the measured 29-occurrence C-05 file census and the 307-occurrence admin-page exclusion.
- [x] 1.2 Add semantic aliases for admin-terminal color roles used by shared components.

## 2. Mechanical migration

- [x] 2.1 Replace the 14 shared-stylesheet HSL-channel call sites.
- [x] 2.2 Replace the three assistant-thread HSL-channel call sites.
- [x] 2.3 Replace the ten shared admin-component HSL-channel call sites.
- [x] 2.4 Replace the two KnowMe logo HSL-channel call sites.
- [x] 2.5 Preserve each migrated alpha percentage through semantic-color mixing.

## 3. Enforcement and baseline

- [x] 3.1 Add a deterministic check for the six C-05 migration files.
- [x] 3.2 Add a case-variant negative fixture and integrate both checks into the root CI grep-gate harness.
- [x] 3.3 Refresh changed Flat 2.0 allowlist strings without altering diagnostic count.
- [x] 3.4 Prove the migrated set is clean and the deferred admin-page set is untouched.

## 4. Verification

- [x] 4.1 Run frontend typecheck, lint, and architectural boundary checks.
- [x] 4.2 Run the root CI grep gates and Tailwind development compilation.
- [x] 4.3 Strictly validate and verify the OpenSpec change.
- [x] 4.4 Complete isolated adversarial review and resolve blocking findings.
- [x] 4.5 Record canonical KBD completion and prepare the verified change for archive.
