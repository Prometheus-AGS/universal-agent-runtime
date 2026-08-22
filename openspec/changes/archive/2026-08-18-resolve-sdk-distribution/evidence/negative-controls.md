# Negative controls — `resolve-sdk-distribution`

Date: 2026-08-18

## Runtime-first Rust publication order

Command from `sdks/rust`:

```bash
RUSTC_WRAPPER= cargo package --locked --allow-dirty --no-verify
```

Observed output and exit:

```text
Packaging universal-agent-runtime-sdk v1.0.0 (.../sdks/rust)
Updating crates.io index
error: failed to prepare local package for uploading

Caused by:
  no matching package named `universal-agent-runtime` found
  location searched: crates.io index
  required by package `universal-agent-runtime-sdk v1.0.0 (.../sdks/rust)`
[exit 101]
```

This is the expected first sequencing control. Cargo uses the local path during
local development and the exact registry requirement in a published package.
It proves the runtime must precede the SDK, but not that the runtime is itself
ready to publish.

Command from the repository root:

```bash
cargo metadata --locked --no-deps --format-version 1 | jq '[.packages[] | select(.name == "universal-agent-runtime") | .dependencies[] | select(.path != null and .kind == null and .req == "*") | {name, req, path}]'
```

Observed output:

```text
liter-llm               req=*  path=vendor/git/liter-llm/crates/liter-llm
prometheus_parking_lot  req=*  path=vendor/git/prometheus-parking-lot-rs
surreal-memory          req=*  path=vendor/git/surreal-memory-server/crates/surreal-memory
sycophancy-core         req=*  path=vendor/git/sycophancy-correction/crates/sycophancy-core
```

The complete blocking order is: publish or replace/reconcile these four
dependencies; add verified registry versions alongside their runtime paths and
prove the runtime package; publish runtime 1.0.0; then prove and publish SDK
1.0.0. This change does not remove embedded support or report either Rust crate
as registry-publishable today.

## Hosted routine-workflow exclusion

Command:

```bash
if test -e .github/workflows/ci.yml; then
  echo 'unexpected routine CI workflow remains'
  exit 1
else
  echo 'legacy routine CI workflow absent'
fi
```

Observed output and exit:

```text
legacy routine CI workflow absent
[exit 0]
```

## Production TypeScript dependency audit

Command from `sdks/typescript`:

```bash
npm audit --omit=dev
```

Observed output and exit:

```text
found 0 vulnerabilities
[exit 0]
```

The earlier `npm ci` audit reported one high-severity development dependency
finding. No claim is made that the full development graph is vulnerability-free.
