# Contributing

Thank you for contributing to the Universal Agent Runtime.

## Quick start

1. Clone the repository and create a worktree under `~/.claude/worktrees/` using `scripts/worktree-new.sh`.
2. Install Rust (see `rust-toolchain.toml`) and Node.js 20+.
3. Run `pnpm install --frozen-lockfile` at the root.
4. Run `cargo check --locked --no-default-features --features server-full`.

## License

- Runtime: MIT
- SDKs (Rust, Python, TypeScript): MIT
- Documentation: CC-BY-4.0

See [CONTRIBUTING.md](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/CONTRIBUTING.md) and the [license ADR](https://github.com/Prometheus-AGS/universal-agent-runtime/blob/main/docs/adr/0017-relicense-runtime-to-mit.md) for details.

## Commit conventions

All commits must follow [Conventional Commits](https://www.conventionalcommits.org/). If you have `lefthook` installed, the commit-msg hook will run `commitlint`.
