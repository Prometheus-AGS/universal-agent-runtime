## 1. Repository Policy

- [x] 1.1 Add the real-model integration testing rule outside the managed region
  in `CLAUDE.md` and verify the section prohibits synthetic inference evidence,
  fails closed when real inference is unavailable, and requires a justified
  objective for multi-hour tests.
- [x] 1.2 Add the identical rule outside the managed region in `AGENTS.md` and
  verify the two policy sections are byte-for-byte equal.

## 2. Durable Decision Record

- [x] 2.1 Append the operator decision and rationale to
  `.prometheus/decisions.md` and verify existing history is unchanged.
- [x] 2.2 Append the policy-change session summary to
  `.prometheus/session-log.md` and verify existing history is unchanged.

## 3. Verification

- [x] 3.1 Run `openspec validate require-real-model-integration-certification
  --strict` and `git diff --check`, then inspect the scoped diff to verify no
  unrelated content was added.
