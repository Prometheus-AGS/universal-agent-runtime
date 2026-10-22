# AG-UI completed result replay source review

Phase: Execute. Reviewed the new pre-latest-run completed replay span in `src/uar/api/agui.rs`, `.agent-team/uar-core/agui-concurrent-replay-repair.md`, shared `prepare_frontend_continuation`/client-tool preparation, run admission and the `caller-tool-continuation` spec delta. Own report only; no production edits.

## Disposition

No blocking source finding in this scoped repair. Runtime concurrency, durable persistence and HTTP behavior remain unverified.

- The early branch requires no interrupt resume entries, a trailing tool result and explicit parent. It loads the predecessor under exact owner/agent/thread/parent and requires matching workspace. The predecessor's native run must resolve under the current verified context.
- The exact trailing call ID is looked up under principal/tenant/source-agent scope. The batch must already be Completed and must match predecessor native run, session and AG-UI protocol before calling shared preparation. Pending and Claimed do not enter this branch; the existing latest-active/parent guard remains on their execution path.
- Shared preparation rechecks source ownership and session/protocol, compares predecessor pending IDs, validates the complete caller history, declarations/controls and result set against durable evidence, and returns the stored completed response only for exact matching results. A conflicting call batch or forged history does not authorize attachment.
- The decoded saved receipt must match owner, path agent, thread, workspace and exact parent wire run. The winning durable row is reloaded by its scoped identity and must retain the saved native run identity. The path returns its stream without reserving another wire run, starting an executor or repeating tool effects.
- Current request validation, global principal/workspace admission and credential-derived ownership precede this branch. This is read-only completed replay: it does not re-resolve current catalog/model execution policy through `routes::admit_run`. The existing exact wire-ID replay has the same read/attachment contract. The reviewed spec requires fresh policy/credentials for a CAS-admitted continuation and separately permits completed retry replay without repeated effects; this patch admits no new continuation. Do not describe this path as fresh agent execution readmission. If product policy requires revocation to block viewing historical runs, that is not implemented by this change.

## Evidence and limits

Only `cat`, `sed`, `rg`, scoped `git diff`, and this documentation write ran. No tests, builds, checks, imports, formatting, runtime or network commands ran. Same-family isolated source review does not constitute cross-model adversarial evidence. Global verification stays CLOSED for this reviewer; the lead owns its scoped functional test/fix loop. No acceptance, CI/merge or publication claim is made. The prior forced-shutdown issue is outside this span and is not certified by this review.
