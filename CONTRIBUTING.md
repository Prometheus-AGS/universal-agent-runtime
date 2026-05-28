# Contributing

Thanks for contributing to Universal Agent Runtime (UAR).

## License for Contributions

By submitting a contribution, you represent that you have the right to submit it and agree that it is licensed under `AGPL-3.0-only` as part of this repository.

This enables maintainers to distribute the open-source version under AGPL and offer separate commercial terms for AGPL-incompatible use.

## Sign-off

Use Developer Certificate of Origin (DCO) sign-off on commits:

```bash
git commit -s -m "your message"
```

The sign-off certifies you have the right to submit the work under the project license.

## Code of Conduct

Contributors are expected to collaborate respectfully and professionally.

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
