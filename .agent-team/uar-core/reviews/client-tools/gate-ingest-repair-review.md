# Bounded ingest and cleanup source review

Phase: `uar-post-lifecycle-verification-readiness`, Execute. Verification lock remains CLOSED for this reviewer. All commands were source inspection (`cat`, `sed`, `rg`, `git diff HEAD -- ...`) or this authorized documentation write. No import, test, static analysis, startup, network, compiler, build, or verification command ran.

Scope: current HEAD-relative diff for `scripts/release/client_tools_local_gate.py` and `openspec/changes/openai-client-tools/handoffs/local-gate.md`; inspected `src/config.rs` and `src/server.rs` only to establish the new ingestion binding. Read the complete Python configuration function, Supervisor class, and main function, applicable OpenSpec deltas, waypoint, AGENTS guidance, team policy/map, and review skills.

## Artifact evidence and disposition

The reviewed configuration hunk is:

```diff
-            "cors_allowed_origins": [], "mcp_shared_transports_enabled": True},
+            "cors_allowed_origins": [], "mcp_shared_transports_enabled": True, "ingest_directory": str(evidence / "ingest")},
```

`main` resolves evidence to an absolute path and creates its private `ingest` directory. The inspected Rust field is `pub ingest_directory: std::path::PathBuf` with serde default `/data/ingest`; the server clones `config.server.ingest_directory` into the watcher after directory creation. No concrete key/path mismatch found in this bounded source review. This is not evidence that the selected prebuilt candidate contains the new Rust implementation.

The reviewed cleanup hunk is:

```diff
-            accepted = joined and child.returncode in expected and not kill_attempted and not errors
-            clean = clean and accepted
+            cleanup_confirmed = joined and not kill_attempted and not errors
+            exit_expected = joined and child.returncode in expected
+            accepted = cleanup_confirmed and exit_expected
+            clean = clean and cleanup_confirmed
```

The surrounding report now retains initial startup/matrix failures and their diagnostic while separately recording cleanup status and child exit expectations. A passed matrix followed by unexpected candidate exit selects `failed_child_exit`; unsuccessful cleanup selects `failed_cleanup` after matrix success and forces process exit 1 in either case. Nonzero candidate exit no longer alone means failed physical cleanup. Forced kill, failed join, cleanup errors, or log-close errors still prevent clean cleanup. No concrete cleanup/reporting regression found in this bounded source review.

The appended handoff describes these source changes consistently. Its gate001 runtime account is lead-supplied evidence, not independently reproduced here. Prior signal/HOME findings were outside this corrective scope and were not reopened.

## Limits and handoff

No production files changed; only this assigned review record was added. No speculative guard or additional implementation was requested. Runtime startup, watcher readiness, actual physical release, all five interface results, and binary/source identity remain unverified by this reviewer. The lead owns any explicitly authorized commands.

Reviewer: isolated native Codex same-family fallback; producer identity was not supplied. Received bounded artifacts, intent and lead failure receipt, without generation conversation. This weaker same-family review does not supply different-family certification. No further judge was dispatched under the bounded assignment.

Manual sycophancy screening: no praise quota, no unsupported runtime-success claim, and no findings fabricated to meet a quota. Machine sycophancy detection was not run because this assignment permits source reads and the review-document write only; no machine score is claimed. Independent cross-family certification remains pending.
