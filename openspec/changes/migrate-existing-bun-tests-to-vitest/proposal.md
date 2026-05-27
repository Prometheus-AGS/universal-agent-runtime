## Why

The repo has 6 unit-test files written against `bun:test`. None of them are invoked by an `npm` script today, so they've effectively been dead code. Now that Vitest is wired up (`vitest-runner-stand-up`), we want all existing tests running on the same runner — uniform CI signal, one set of conventions, no parallel `bun test` vs `vitest run` invocations.

## What Changes

Per file, swap the `bun:test` import for the equivalent `vitest` import:

```diff
- import { beforeEach, describe, expect, test } from "bun:test";
+ import { beforeEach, describe, expect, test } from "vitest";
```

Files:
- `frontend/src/index.cursor-policy.test.ts`
- `frontend/src/stores/chat-message-store.test.ts`
- `frontend/src/entities/runtime-ingest.test.ts`
- `frontend/src/features/chat/use-message-stream.test.ts`
- `frontend/src/features/chat/use-thread-naming.test.ts`
- `frontend/src/admin/pages/skills-page.utils.test.ts`

For each file, audit and remap any of:
- `mock(...)` → `vi.fn()`
- `spyOn(...)` → `vi.spyOn(...)`
- `mock.module(...)` → `vi.mock(...)`

The `describe / test / expect / beforeEach` API surface is API-compatible between `bun:test` and `vitest`, so most files require only the import swap.

## Acceptance

- `pnpm --filter ./frontend test` runs all 6 tests; all pass.
- No `bun:test` references remain in `frontend/src`.
- Pre-existing test behavior preserved (no new assertions, no skipped tests).
