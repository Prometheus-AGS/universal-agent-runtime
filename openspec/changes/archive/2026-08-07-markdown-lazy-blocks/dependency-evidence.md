# Dependency transaction evidence

The C-09 package transaction is present in the working tree; raw package and lockfile diffs are excluded from the adversarial-review file list because those tracked files also contain cumulative changes from already completed C-02/C-03 changes.

- `frontend/package.json`: `mermaid: ^11.16.1`; `shiki: ^4.4.2`.
- `frontend/pnpm-lock.yaml` importer: Mermaid specifier `^11.16.1`, version `11.16.1`; Shiki specifier `^4.4.2`, version `4.4.2`.
- Root `pnpm-lock.yaml` frontend importer: Mermaid specifier `^11.16.1`, version `11.16.1`; Shiki specifier `^4.4.2`, version `4.4.2`.
- `pnpm -C frontend install --frozen-lockfile --ignore-scripts`: pass; lockfile current and supply-chain policies pass.
- Root `pnpm install --frozen-lockfile --ignore-scripts`: pass; lockfile current and supply-chain policies pass.

The resulting application build resolves both packages and emits their named lazy entries. This file is review-packet evidence only; the package manifest and both maintained lockfiles remain the source of truth.
