# Spec revision brief — round 1 (lead)

Source: `review/spec/findings-round1.json` (BLOCK, 8 critical, 13 warning). Same rules as
`spec-packet.md`: spec files only, OpenSpec runner only, re-run `validate --strict`, one writer
per change. Update the `## Scenario matrix rows` in design.md for every scenario you touch;
the lead regenerates the contract from those rows.

## Cross-cutting rules (apply in every change you own)

R1. **Oracle truth source is frozen now.** An oracle may not take its expected values from
anything the implementer writes during the phase (route tables, classifiers, audit files,
registers, critique self-counts) or from the system under test. Put the expected values in the
change now, as a frozen fixture listed in design.md under `## Frozen oracle fixtures` (exact
list of controls, routes, keys, rows, counts), with a stable file path the boundary runner
reads (e.g. `openspec/changes/<id>/fixtures/<name>.json` — write the file now).

R2. **Negative controls must be able to reject.** A mandatory scenario's negative control
must use a real invalid/conflicting input and name an observable rejection. It may not be the
scenario itself, NOT_EXERCISED, or missing. If it truly cannot run without a fault hook, make
the fault hook an implementation task and the control mandatory.

R3. **Declare nondeterminism.** Any scenario that depends on timing, concurrency, leader
election, network, model output or rendering: set `nondeterministic: true` and give a
`stochastic` block in the row: samples (integer), grader (what counts as pass per sample),
stoppingRule (fixed N, all must pass unless stated).

R4. **No paid model calls.** Scenarios use the local stub model server `src/bin/stub-llm.rs`
(confirm its invocation from source) instead of real model routes.

R5. **Prerequisites.** Name any example binary or baseline the scenario needs; the lead adds
build commands. Baseline worktrees go under `~/.claude/worktrees/` (project rule).

## Per-change assignments

| Writer | Change | Findings to fix |
|---|---|---|
| state-D | realtime | C2: keep refreshing the registry default from the `llm.default_provider` row on every replica; state that the row is written only by the live default route because the settings API rejects that key (owned by settings change). C3: add SCN-RTM-008 persist-before-publish with a fault hook (persist failure ⇒ no topic frame, live default unchanged on all replicas) and SCN-RTM-009 capability inversion (an agent-kernel tool call that attempts a settings or default-provider mutation is refused at the host boundary; check src for the governed-action/host-tool gate and cite it). W: SCN-RTM-004 expected status must match the settings change's classification of `server.port` (inert ⇒ `not_applied`), or pick a key that is genuinely restart-required. R1-R5. |
| state-D | config | C8 is not yours to build, but remove the hand-off in task 5.3 and instead reference the settings change's new tasks by id once console-C names them (lead will tell you; for now write "owned by uiux-settings-truthful-status-and-removals"). W: remote profile excluded from fallback contradicts "YAML never prevents startup": keep your design, but state it as an explicit limitation and add it to Unresolved for the operator. R1-R5. |
| console-C | settings | C2: the settings API SHALL reject writes to `llm.default_provider` (name status code and error body pointing to `POST /api/uar/providers/{id}/default`); replace the `restart` classification for that key. Update SCN-SET-003 accordingly. C3: add SCN-SET-009 masked-secret round trip (a secret setting read back is masked; saving the masked value unchanged keeps the stored secret; the secret never appears in API responses or logs). C8: add tasks and SCN-SET-010 for the console screens: boot-fallback warning naming the rejected file, drift view with one-click resolution, and the reload report — consuming the config change's API. C5: fix SET-002, SET-005 oracles per R1. R1-R5. |
| console-C | design | C5: fix DSN-001, DSN-003, DSN-007 oracles per R1 (critique pass criteria must come from an independent critic's findings file, not self-report; route expectations from a frozen list in this change). R1-R5. |
| console-B | identity | C6: SCN-IDN-001 must exercise the shared-bare-id collision (two providers offering the same model id); IDN-003 needs a baseline measured before the change, frozen in the change. W: add the Models-page list virtualization task (owned here). C5: IDN-002 compare against a frozen fixture, not the same rows before reconnect. R1-R5. |
| console-B | pickers | C5: PICK-004's free-text inventory must be a frozen list written now in this change (from assessment.md:211-223 and your source audit), not an implementer register. R1-R5. |
| console-A | persistence | C4: declare nondeterminism for PGL-005, PGL-009 and any real-route rows per R3; R4 (stub-llm). R1-R5. |
| console-A | workers | C6: WRK-007 needs a real expected rejection. C4: WRK-007, WRK-009 per R3. R1-R5. |
| providers-E | system model | C6: SYS-006 control must differ from the scenario. C7: SYS-004 must first assert the stored Global `model` exists, then show the turn ignores it; keep the operator default "ignore on write" out of the fixture path (seed the stored value directly in the scratch store). W: add SCN-SYS-009 agent-level model change takes effect in the next turn of an open conversation. R1-R5. |
| runtime-F | team | R1-R5 review of your rows only (no criticals named against TEAM). |

## Lead-owned (contract level)

- C1 runner: boundary packet adds a runner-integrity gate (runner authored from frozen rows; independent critic reviews the runner against the contract before B4; runner run against an unmodified pre-phase build must FAIL every mandatory scenario that asserts new behavior).
- C6: contract rule — a mandatory scenario whose negative control did not run is BLOCKED, never PASS.
- W: separate `--target-dir` for the two release builds; prerequisite builds.
