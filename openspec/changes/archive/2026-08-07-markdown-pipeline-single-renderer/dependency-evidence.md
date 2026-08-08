# C-08 Dependency Evidence

The C-08 package transaction added this complete trust-boundary/math set to
`frontend/package.json` and `frontend/pnpm-lock.yaml` together:

- `dompurify` `^3.4.13`
- `katex` `^0.18.1`
- `rehype-katex` `^7.0.1`
- `rehype-raw` `^7.0.0`
- `rehype-sanitize` `^6.0.0`
- `remark-breaks` `^4.0.0`
- `remark-math` `^6.0.0`

The package manager's supply-chain policy check passed. Other dependency
changes visible relative to `HEAD` belong to earlier completed KBD changes and
are intentionally excluded from the C-08 adversarial packet.
