# Contributing

Thanks for contributing to Universal Agent Runtime (UAR).

## License for Contributions

By submitting a contribution, you represent that you have the right to submit it and agree that it is licensed under `MIT` as part of this repository.

That is the whole obligation. There is no CLA, no copyright assignment, and no
dual-licensing clause: the runtime, the SDKs under `sdks/`, and everything else
in this repository are MIT.

> **Relicensed 2026-08-07.** The runtime was previously `AGPL-3.0-only` with a
> separate commercial exception, which required a CLA-lite clause so commercial
> licensees could receive the same functionality as AGPL users. MIT removes the
> reason for that machinery entirely. Contributions made before this date were
> made under the previous terms; the relicense applies going forward and git
> history is left intact.

## Sign-off

Use Developer Certificate of Origin (DCO) sign-off on commits:

```bash
git commit -s -m "your message"
```

The sign-off certifies you have the right to submit the work under the project license.

## Code of Conduct

Contributors are expected to collaborate respectfully and professionally.

## Commit message convention

This repo uses [Conventional Commits](https://www.conventionalcommits.org/). All
commit messages must follow the format:

```
<type>(<optional scope>): <short description>

[optional body]

[optional footer(s)]
```

Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`.

If you have `lefthook` installed (`pnpm exec lefthook install`), the
`commit-msg` hook runs `commitlint` and blocks non-conventional commits in the
JS workspace. `release-plz` also uses conventional commits to generate
changelogs and version bumps.

## Questions

For licensing or contribution process questions, contact project maintainers.

## Local setup — worktrees

Create new git worktrees of this repo under `~/.claude/worktrees/`, never inside
the repo tree. The repository's own `.claude/` directory holds checked-in tool
configuration; putting worktrees alongside it collides namespaces. See the
"Worktree convention" section in `CLAUDE.md` and `AGENTS.md` for the full
rationale.

Use the helpers:

```bash
scripts/worktree-new.sh <name> [--base <ref>]
scripts/worktree-list.sh
scripts/worktree-rm.sh <name> [--force]
```

### Optional advisory pre-commit hook

If you want a warning whenever a path under `.claude/worktrees/` lands in the
index, install this one-liner into `.git/hooks/pre-commit` (`chmod +x`):

```bash
#!/usr/bin/env bash
# Advisory only — prints a warning, does not block the commit.
if git diff --cached --name-only | grep -q '^\.claude/worktrees/'; then
  printf 'warning: a path under .claude/worktrees/ is staged for commit.\n' >&2
  printf '         use scripts/worktree-new.sh; worktrees belong under ~/.claude/worktrees/.\n' >&2
fi
```

A managed installer for this hook is out of scope for the worktree-persistence
change; the snippet above is the manual install path.

### Tests for the worktree helpers

The helpers ship with [bats-core](https://github.com/bats-core/bats-core) tests
under `scripts/tests/`. Install bats (`brew install bats-core` or your package
manager) and run:

```bash
bats scripts/tests/worktree-new.bats
bats scripts/tests/worktree-rm.bats
```

bats is **not** a required dependency for the repo — the test target is gated
on its presence.

## Disclaimer

This document is informational and not legal advice.
