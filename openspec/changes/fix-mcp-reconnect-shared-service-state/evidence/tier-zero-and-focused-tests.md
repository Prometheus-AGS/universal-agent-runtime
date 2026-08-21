# Tier 0 and focused MCP tests

Date: 2026-08-21
Profile: `universal-agent-runtime`, `server-full`, no default features, local macOS arm64

## Tier 0 check

Command:

```bash
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
```

Observed exit: `0`

Observed output tail:

```text
warning: constant `MAX_BODY_BYTES` is never used
warning: constant `MAX_REDIRECTS` is never used
warning: type does not implement `std::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
warning: `universal-agent-runtime` (lib) generated 3 warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.73s
```

The three warnings are in the out-of-scope pre-existing files
`src/uar/tools/fetch_guard.rs` and `src/uar/runtime/skills/wasm_runtime.rs`.
No warning references `src/mcp/registry.rs`.

## Package-scoped Clippy

Command:

```bash
set -o pipefail
cargo clippy --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib --no-deps \
  --message-format=json 2>/dev/null \
  | node -e 'let input=""; process.stdin.on("data", chunk => input += chunk); process.stdin.on("end", () => { let messages = []; for (const line of input.split("\n")) { if (!line) continue; const event = JSON.parse(line); const message = event.message; if (event.reason === "compiler-message" && message && message.spans.some(span => span.file_name === "src/mcp/registry.rs")) messages.push(message.message); } console.log(`MCP_REGISTRY_CLIPPY_MESSAGES=${messages.length}`); for (const message of messages) console.log(message); });'
clippy_status=${PIPESTATUS[0]}
echo "CARGO_CLIPPY_EXIT=$clippy_status"
exit "$clippy_status"
```

Observed exit: `0`

Observed filtered output:

```text
MCP_REGISTRY_CLIPPY_MESSAGES=3
called `map(<f>).unwrap_or(<a>)` on a `Result` value
`map_err(|_|...` wildcard pattern discards the original error
called `map(<f>).unwrap_or(<a>)` on a `Result` value
CARGO_CLIPPY_EXIT=0
```

The first post-edit replay also reported a fourth
`clone_on_ref_ptr` warning on the newly shared slot. That warning was corrected
to explicit `Arc::clone`; the final replay contains only the same three
pre-existing call-metrics warnings present before the test edit. The full
package command emits existing pedantic warning debt outside this child and is
not represented as a zero-warning repository verdict.

## Filtered/merged crash recovery and authorization

Command:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  mcp::registry::tests::reconnect_replacement_is_shared_without_widening_filtered_views \
  -- --exact --nocapture
```

Observed exit: `0`

Observed result:

```text
running 1 test
test mcp::registry::tests::reconnect_replacement_is_shared_without_widening_filtered_views ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 611 filtered out; finished in 0.49s
```

This test records one crash call in the fixture, asserts a new process for the
next independently filtered request, asserts a pre-existing merged view uses
that same replacement, and asserts excluded server/tool views remain empty.

## Upsert propagation

Command:

```bash
cargo test --locked -p universal-agent-runtime \
  --no-default-features --features server-full --lib \
  mcp::registry::tests::upsert_replaces_service_in_an_existing_filtered_view \
  -- --exact --nocapture
```

Observed exit: `0`

Observed result:

```text
running 1 test
test mcp::registry::tests::upsert_replaces_service_in_an_existing_filtered_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 611 filtered out; finished in 0.44s
```

## Local certification contracts and fail-closed controls

Command:

```bash
node --check scripts/validate-candidate-certification-workflow.mjs
node --check scripts/validate-mcp-process-boundary-evidence.mjs
node --check scripts/validate-candidate-certification.mjs
bash -n scripts/certify-release-candidate.sh
pnpm release-local-contracts:validate
pnpm github-actions-policy:validate
```

Observed exit: `0`

Observed output:

```text
Local supply-chain producer, provenance, manifest, validator, and schema contracts passed.
MCP_PROCESS_BOUNDARY_CONTRACT_PASS positive=1 negative_controls=6 success_substitution=reject duplicate_event=reject duplicate_execution=reject missing_transition=reject stale_after_crash=reject stale_after_timeout=reject
Local installed-candidate certification, packaging, and bundle contracts passed.
GitHub Actions policy validation passed (deployment workflows only: deploy.yml, docs.yml, typescript-sdk-docs.yml).
```

These results prove the child implementation and synthetic evidence contract at
Tier 0/Tier 1. They do not prove the packaged binary's real 30-second timeout;
that remains the immutable installed-artifact preflight.
