## Verification Report: agui-event-parity-and-normalizer

### Summary

| Dimension | Status |
|---|---|
| Completeness | 9/9 tasks complete; 4/4 requirements implemented |
| Correctness | 11/11 scenarios have code paths; 10/11 have executed focused or suite evidence |
| Coherence | Design decisions and repository layering followed |

### Completeness

- `frontend/src/platform/agui/agui-normalizer.ts` owns the typed message,
  event-row, phase mapping, and terminal timing projections.
- `frontend/src/platform/agui/agui-adapter.ts` normalizes only after official
  profile validation and before compatibility reduction reaches consumers.
- `frontend/src/stores/chat-stream-store.ts` consumes message chunks and event
  rows and upserts terminal `phase_timings` through the existing entity layer.
- `src/uar/api/routes.rs` selects the attach/resume cursor and
  `src/uar/api/sse.rs` reconstructs snapshots and synthesizes exactly-once tool
  starts before mapped args/end frames.
- `complete-agui-event-parity` is archived with `--skip-specs`; its obsolete
  `agui-spec-parity` capability was not applied.

### Correctness

- Frontend focused proof: 3 files / 22 tests pass, including phase mapping,
  message/event projections, duplicate identity rejection, terminal success
  and error timing, state recovery, and opaque RAW behavior.
- Rust focused proof: 5 replay-filter tests pass, including cursor state/message
  reconstruction, invalid-patch fail-closed behavior, and exactly-once
  `TOOL_CALL_START` ordering. After adversarial review, the tool projection
  regression was rerun independently with eight buffered argument chunks and
  passed with unique ordinals and non-overlapping event sequences.
- Wave 2 boundary: 36 frontend test files / 171 tests and production build pass.
- Compiler/gates: frontend typecheck, lint, architecture boundaries,
  `server-full` check, strict OpenSpec validation, and `git diff --check` pass.

### Coherence

- No new dependency was introduced.
- Frontend flow remains store-owned and uses the existing entity ingestion
  boundary; no component or hook reaches a service.
- RAW is retained as an official event row but is not interpreted as trusted
  UAR domain state.
- Snapshot state fails closed when retained RFC 6902 operations cannot be
  reconstructed from the initial UAR state shape.
- Legacy chat/thinking frames retain their prior reduction behavior, while
  official frames exclusively use typed message chunks.
- Replay argument deltas wait for the real tool name before the projector emits
  `TOOL_CALL_START`, removing placeholder tool identities.
- Projected frames are renumbered from their emitted order, while frames from
  one retained event share its ordering sequence; arbitrarily long argument
  streams therefore cannot collide with a lifecycle frame or the next event.
- Legacy SSE attach retains cursor-scoped history fetching; only `agui_spec`
  attach loads full retained history for snapshot reconstruction.

### Adversarial Review

- Round 1 blocked at 1 critical / 7 warnings / 1 suggestion. The critical was
  contradicted by the existing `run_updated` mapping; three useful warnings
  were absorbed into direct persistence coverage, explicit legacy fallback,
  and real-name tool-start projection.
- Round 2 blocked at 1 critical / 1 warning / 1 suggestion and passed the
  anti-sycophancy gate at score 0.0. The valid critical ordinal collision and
  valid legacy-history warning were fixed and revalidated. The Vite duplicate
  suggestion belongs to cumulative earlier change content, not C-06.
- Round 3 passed at 0 critical / 2 warnings / 0 suggestions, with verified
  producer/judge separation and anti-sycophancy score 0.0. One warning asks to
  reverse the explicit high-frequency-row acceptance criterion; the other
  concerns cumulative local permission configuration outside C-06.

### Issues by Priority

#### CRITICAL

None.

#### WARNING

1. The two live AG-UI seam cases did not execute. The integration target fails
   during compilation at `tests/integration/live/harness.rs:193` because its
   pre-existing `Cli` initializer omits the required `strict_config` field and
   later passes `Arc<AppConfig>` where `Arc<ConfigManager>` is required.
   Recommendation: repair that shared harness in its owning change, then rerun
   `agui_spec_mode_emits_official_event_vocabulary` and
   `agui_spec_mode_maps_tool_call_lifecycle_to_official_vocabulary`.
2. Repository-wide `cargo fmt --all -- --check` remains red on unrelated dirty
   Rust files outside C-06. The three C-06 Rust files pass direct `rustfmt
   --check`.

#### SUGGESTION

None.

### Final Assessment

Independent review confirms no unresolved C-06 critical issue remains.
External verification limitations are recorded accurately; all C-06-owned
checks pass. Ready to archive.
