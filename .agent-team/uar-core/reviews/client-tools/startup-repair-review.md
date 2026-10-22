# Native startup repair source review

Phase: `uar-post-lifecycle-verification-readiness`, Execute. This is source inspection only; the lead owns execution and gate authorization. No tests, compiler, Cargo, lint, formatting, startup, network, or acceptance commands were run. Runtime certification is withheld.

## Scope and isolation

Reviewed `git diff HEAD -- src/config.rs src/server.rs tests/settings_persistence.rs example.config.yaml` and inspected the new `docs/releases/v1.2.2-kubo.md`. Read the applicable configurable-ingestion requirement in `openspec/changes/openai-client-tools/specs/caller-tool-continuation/spec.md:23`. The review received the repair intent and final artifacts, without producer conversation. Producer: coordinator/implementation agent, exact model identity not supplied. Judge: native Codex GPT-6-family reviewer, isolated subagent context. This explicitly authorized same-family fallback provides weaker independence than a cross-family judge.

## Findings

No demonstrated CRITICAL, HIGH, or WARNING defect found in the scoped repair.

- `src/config.rs:588` adds an owned configurable path with a serde default of `/data/ingest`, preserving omitted-field configuration compatibility.
- `src/server.rs:1333` prepares the configured directory before registering/spawning the watcher producer. A failed create now returns the configured path and original I/O message directly, instead of dropping the readiness sender inside the spawned task and reporting only missing acknowledgment.
- `src/server.rs:1340` retains supervisor ownership, cancellation token, readiness acknowledgment, and `mark_ready` ordering. Inspection of `src/uar/rag/ingest.rs:312` shows native watch registration still precedes its success acknowledgment.
- `tests/settings_persistence.rs:139` updates the explicit application `ServerConfig` literal. Source search found no other application-config literals requiring the field; the similarly named MCP configuration is a different type.
- `example.config.yaml:19` documents the configurable path and environment override. The loader at `src/config.rs:1848` uses the matching UAR prefix and double-underscore separator. The release note documents the same path/default contract.

The correction targets the reported native startup failure without weakening readiness or suppressing directory errors. It does not itself establish that a native candidate supplies a writable override or reaches readiness; those are execution obligations for the lead's authorized real-path gate.

## Verification limits and screening

Commands used were source reads/searches and `git diff HEAD` only. Cargo check, Clippy, formatting, tests, customer build, startup/retest, packaging, and CI were not run in this source-only assignment. Compilation, environment deserialization, runtime startup, watcher cleanup, and the published artifact remain unverified by this review. Existing CI/merge status was not inspected, and no green-CI or mergeability claim is made.

Sycophancy detection was available and run on the review's substantive conclusion: score `0.0`, classifications `[]`, correction mandatory `false`, audit timestamp `2026-10-22T17:41:21.833637Z`. Manual screening retained the execution limits and weaker-independence disclosure. No production files were edited; this report is the only review-owned write.
