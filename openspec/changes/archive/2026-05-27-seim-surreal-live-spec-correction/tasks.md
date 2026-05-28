# Implementation Tasks — seim-surreal-live-spec-correction

> Documentation-only change. No code in any repo. The "implementation"
> is verifying the corrected spec is internally consistent, the
> reconciliation preamble is correct, the companion skill on disk
> aligns with the corrected spec, and the archive operation overwrites
> the right file.

## 1. Corrected spec sanity checks

- [ ] 1.1 Confirm the corrected spec exists and parses as Markdown:
  ```sh
  test -s openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 1.2 Count requirements — expect exactly 10:
  ```sh
  grep -c '^### Requirement:' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```
- [x] 1.3 Count scenarios — actual count is 33 (drafted more per-requirement scenarios than the design's estimate of 27; both spec and design.md updated to 33):
  ```sh
  grep -c '^#### Scenario:' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 1.4 Confirm the **reconciliation preamble** exists and names the supersession path:
  ```sh
  grep -q '^> \*\*Reconciliation note\.\*\*.*archive/2026-05-27-ssed-entity-surreal-live-adapter' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 1.5 Confirm the spec returns `RealtimeAdapter`, not `SyncAdapter`:
  ```sh
  ! grep -q 'returns? `SyncAdapter`' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md \
    && grep -q 'returns? `RealtimeAdapter`\|: RealtimeAdapter' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 1.6 Confirm no `start(handler)` / `stop()` lifecycle references remain in the corrected spec:
  ```sh
  ! grep -E 'start\(handler\)|adapter\.stop\(\)|onSynced' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 1.7 Confirm the corrected spec references `RealtimeManager.register` (per-channel) rather than `registerAdapter` (global):
  ```sh
  grep -q 'RealtimeManager\.register(adapter, channels' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md \
    && ! grep -q 'registerAdapter' \
    openspec/changes/seim-surreal-live-spec-correction/specs/entity-surreal-live-adapter/spec.md
  ```

## 2. Companion-skill alignment check

The companion skill `entity-realtime-surreal-live/SKILL.md` was shipped via the change 1 PR. This change does NOT modify it, but the corrected spec requires the skill to match. Spot-check:

- [ ] 2.1 Confirm the skill file exists on disk:
  ```sh
  test -s ~/.claude/skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md
  ```
- [ ] 2.2 Confirm the skill describes `RealtimeAdapter`, not `SyncAdapter`:
  ```sh
  grep -q 'RealtimeAdapter\|createSurrealLiveAdapter' \
    ~/.claude/skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md
  ```
- [ ] 2.3 Confirm the skill mentions `manager.register` per-channel registration (not a global `start`):
  ```sh
  grep -qE 'manager\.register|register\(adapter, channels' \
    ~/.claude/skills/prometheus-entity-skills/entity-realtime-surreal-live/SKILL.md \
    || echo "WARN: skill may not name per-channel registration explicitly — inspect manually"
  ```
- [ ] 2.4 If §2.3 emits the WARN, manually inspect the skill and decide:
  - If the skill's wording is ambiguous but consistent with the corrected spec → no follow-up needed
  - If the skill describes the wrong interface → open a corrective change in W2 to amend the skill (since PR #3 is already merged, this would be a separate skill-system PR)

## 3. Historical archive verification

- [ ] 3.1 Confirm the originally-archived spec still exists at its historical path (the supersession does NOT delete history):
  ```sh
  test -s openspec/changes/archive/2026-05-27-ssed-entity-surreal-live-adapter/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 3.2 Confirm the originally-archived spec describes the obsolete `SyncAdapter` shape (this is the documentation of WHY the correction was needed):
  ```sh
  grep -q 'SyncAdapter' \
    openspec/changes/archive/2026-05-27-ssed-entity-surreal-live-adapter/specs/entity-surreal-live-adapter/spec.md
  ```
- [ ] 3.3 Confirm the reconciliation preamble in the *new* spec correctly cites this historical path verbatim — if §1.4 passed, §3.3 is automatically satisfied

## 4. Archive operation (run during `/opsx:archive`)

The archive logic is the same as every prior spec-driven change with one wrinkle: this change overwrites an existing promoted spec.

- [ ] 4.1 `mkdir -p openspec/changes/archive/2026-MM-DD-seim-surreal-live-spec-correction/`
- [ ] 4.2 Move `proposal.md`, `design.md`, `tasks.md` from `openspec/changes/seim-surreal-live-spec-correction/` to the archive directory
- [ ] 4.3 Move `specs/` from `openspec/changes/seim-surreal-live-spec-correction/specs/` to the archive's `specs/` (the historical record of what this correction shipped)
- [ ] 4.4 **Overwrite** `openspec/specs/entity-surreal-live-adapter/spec.md` with the corrected file from the archived `specs/entity-surreal-live-adapter/spec.md` — this is the promotion step that differs from a normal archive (a normal archive *creates* `openspec/specs/<cap>/spec.md`; this one *replaces*)
- [ ] 4.5 Remove the now-empty `openspec/changes/seim-surreal-live-spec-correction/` directory
- [ ] 4.6 Verify the promoted spec carries the reconciliation preamble (this is the operator's final cross-check before committing):
  ```sh
  head -5 openspec/specs/entity-surreal-live-adapter/spec.md | grep -q 'Reconciliation note'
  ```

## 5. Cross-tool reachability check

Other AI tools (Roo, Cursor, Codex, OpenCode) read `openspec/specs/` for capability context. After the promotion:

- [ ] 5.1 Confirm `openspec/specs/entity-surreal-live-adapter/spec.md` is the *new* spec (the one with the reconciliation preamble), not the old one
- [ ] 5.2 Confirm the historical archive path resolves and is still the *old* spec
- [ ] 5.3 No further action needed — the file is at the canonical path; tools that re-sync their spec index will pick it up automatically

## 6. Documentation

- [ ] 6.1 No README/index changes required — `openspec/specs/` is a flat directory consumed by spec-index machinery
- [ ] 6.2 The reconciliation preamble IS the documentation of the change for spec consumers
- [ ] 6.3 No CHANGELOG update — spec corrections are tracked via OpenSpec's own archive trail

## 7. Cross-repo commit

- [ ] 7.1 All file edits in this change are confined to `universal-agent-runtime/openspec/`. **No cross-repo commit required.**
- [ ] 7.2 Local UAR commit: `chore(openspec): correct entity-surreal-live-adapter spec to RealtimeAdapter (seim-surreal-live-spec-correction)`. Includes:
  - The new change directory before archive (or the archive entry after step §4)
  - The promoted `openspec/specs/entity-surreal-live-adapter/spec.md`
- [ ] 7.3 Push to the same UAR branch that received the back-reference commit from change 1 (or open a fresh branch — operator's choice; no convention enforced)

## 8. Closeout

- [ ] 8.1 Update `.kbd-orchestrator/phases/submodule-entity-management-implementation/progress.json`:
  - [ ] 8.1.1 `changes_completed: 2`, append `seim-surreal-live-spec-correction` to `completed_changes`
  - [ ] 8.1.2 Set `active_change: "seim-em-worktree-setup"`, state `ready_for_opsx_new`
- [ ] 8.2 `/opsx:verify seim-surreal-live-spec-correction` — recommended `--skip-qa` since the change is documentation-only and below the 3-file threshold (changes only `openspec/specs/entity-surreal-live-adapter/spec.md` plus its own change directory which gets archived)
- [ ] 8.3 `/opsx:archive seim-surreal-live-spec-correction` — executes §4 above
- [ ] 8.4 Refresh `current-waypoint.json` to point at change 3 (`seim-em-worktree-setup`); `exactNextCommand` becomes `/opsx:new seim-em-worktree-setup`
- [ ] 8.5 W1 complete → W2 can start. W2 produces a new persistent worktree at `~/.claude/worktrees/seim-entity-management` for the remaining TypeScript work.

## Rollback (only if verification surfaces a critical defect)

- [ ] R1 `git revert` the UAR commit produced in §7.2
- [ ] R2 The historical archive at `2026-05-27-ssed-entity-surreal-live-adapter/` remains untouched and authoritative again
- [ ] R3 The companion skill on disk is unchanged either way (no rollback needed there)
- [ ] R4 Open a corrective change (`seim-surreal-live-spec-correction-v2`) addressing the defect; do not amend this one once archived
