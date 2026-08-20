PLAN: fix-pnpm-lock-submodule-consistency
Project: universal-agent-runtime
Date: 2026-08-20
OpenSpec available: YES
Changes to implement: 1

CHANGE LIST (ordered)
1. fix-pnpm-lock-submodule-consistency: commit a root lockfile consistent with the pinned workspace submodule without changing dependency intent
   - Scope: root dependency lock, OpenSpec evidence, child KBD handoff
   - Depends on: NONE
   - Recommended agent: Codex
   - Est. complexity: S
   - Customer value: HIGH
   - Details: Adopt the existing operator lock candidate because it retains the dependency resolutions already exercised by the preceding child and passes pnpm 11.15.0 frozen validation unchanged. Add a `frontend-build-tooling` requirement that any committed workspace manifest or workspace-submodule advance must have a matching frozen-installable root lock. Do not regenerate to current allowed-range latest versions, edit manifests, or run the parent browser suite inside this child.

EXECUTION ROUND ORDER
Round 1: fix-pnpm-lock-submodule-consistency

VERIFICATION ORDER
1. Retain the observed stale-commit negative control: clean `fa4ffb96` exits 1 with `ERR_PNPM_OUTDATED_LOCKFILE`.
2. Record the candidate SHA-256, run `pnpm install --lockfile-only --frozen-lockfile --ignore-scripts`, and prove the SHA-256 is unchanged.
3. Run `pnpm install --frozen-lockfile --ignore-scripts` to prove a fresh dependency installation accepts the lock.
4. Run the root supply-chain lock validator, scoped diff checks, and strict OpenSpec validation.
5. Refine and obtain history-free critic and judge approval before archive and commit.

SCOPE CUTS AND TRADE-OFFS
- Preserve `lucide-react` 1.32.0 from the exercised candidate. A fresh non-frozen resolution now selects 1.33.0; accepting that upgrade would enlarge this repair without an observed need.
- Preserve existing manifest ranges and every product source file. Dependency-policy cleanup is not part of this child.
- Parent browser preparation, videos, reports, and certification remain blocked until this child produces a clean source commit.

COMMANDS TO RUN
/opsx:new fix-pnpm-lock-submodule-consistency

PLAN COMPLETE
