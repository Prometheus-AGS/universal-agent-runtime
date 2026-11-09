## Context and decision
Preserve existing test assertions and existing release evidence ownership. The defect is a duplicate RunStreamState field, so remove only that assignment. Use the existing lefthook pre-commit mechanism for a shell script compiling the library test target serially with --locked --no-default-features and explicit features. No new dependencies.

Default local mode compiles the working tree and makes no committed acceptance claim. Acceptance mode refuses dirty/uninitialized/mismatched dependency sources, records commit/tree/toolchain/profile identity and refuses changed source after commands. This traces directly to the observed temporary-patch acceptance failure. Final release input provenance remains the existing release tooling's responsibility.

## Limits
A compile pass cannot prove request behavior, persistence or installed-platform qualification. Existing named history/embeddings/OpenAPI tests establish only their own assertions. Baseline work is scoped to #462; unrelated diagnostics are recorded separately. Tests execute only after implementation completion under the operator's October31 phase-exit rule. External actions are not authorized by local acceptance.
