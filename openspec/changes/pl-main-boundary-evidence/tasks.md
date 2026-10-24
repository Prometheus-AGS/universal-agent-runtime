# Implementation / authoring tasks — pending review

- [x] 1.1 Assemble the scoped main-process integration target and frozen scenarios
- [x] 1.2 Author one small candidate-bound runner using existing receipt primitives
- [x] 1.3 Prepare complete implementation inventory and exact approval packet without running it

Task model assignments: .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/plan.md, keyed by full phase path + pl-main-boundary-evidence + each backend ID above. No worker dispatched. Test authoring may occur after Execute authorization; running anything waits for the complete implementation boundary and exact command approval.

Source-authoring reconciliation — 2026-10-24: the earlier pending boxes and “No worker dispatched” sentence describe the planning baseline. `tests/post_lifecycle_main.rs` now contains the frozen ten scenario IDs; `scripts/post_lifecycle/main_boundary.py` is authored. Phase artifacts `implementation-inventory.final.json`, `implementation-closeout.json` and `boundary-packet.proposed.json` record original13/13 source tasks complete, with task1.3 explicitly delivered as a non-runnable approval packet. These boxes record that authorship only. The historical build failed before compilation, the original runtime boundary is NOT RUN/BLOCKED, final fixture/manifest binding remains unresolved, and this source does not inherit acceptance from caller-tools receipts. Verification remains CLOSED pending a fresh whole-phase inventory and exact operator command approval. See the phase `v1.3.0-ledger-reconciliation.md` for the current ledger comparison.
