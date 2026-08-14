# 2026-08-13 — 19 skills were installed under names no tool searches for

**Impact.** The operator shipped software believing the update scripts made every
skill available to every tool. They did not. A Codex session blocked because
`deep-research` was not where it looked. The condition had persisted across
"many loops" of running the installer, each of which reported success.

**Detection.** Operator report, not by any check. Every automated signal was
green throughout.

---

## What was actually wrong

`scripts/install-plugin-generation.js` diverts placement when something it does
not own holds a skill's canonical name:

| Line | Function | Behaviour |
|---|---|---|
| 930 | `installLinkTarget` | canonical occupied → install to `prometheus-<name>` |
| 987 | `copySkill` | same, for copy targets |
| 1038 | `targetDestination` | **re-derives the same fallback**, so `verifyTargets` (1045) validates the renamed path |

`fail()` fired only when the *fallback* was also occupied. The canonical name
being taken was the trigger for the rename, never an error. `prometheus-${skill.name}`
appears three times in that file; none of the three was adjacent to any
diagnostic output. The receipt records `skillCount` from *intent*
(`targetPayloads`, 603), so it could not distinguish a clean install from one
where every skill was renamed.

**The install genuinely succeeded by its own definition.** `deep-research` was
current on disk the whole time — under `prometheus-deep-research`, which nothing
looks for. `~/.claude/skills/deep-research` held an unrelated April stub: one
file, 4,582 bytes, against the real skill's 13,519 bytes and eight
subdirectories.

Final count: **19 skills unreachable at their canonical name**, across 14
targets. Three were unowned directories. Sixteen were symlinks into a live
source checkout (`prometheus-skill-pack/skills/imported/`, April) or
`~/Projects/travisjames/skills/` (July). One of those, `artifact-refiner`, was
serving four-month-old content across six targets: checkout `615d98ed`,
generation `3c776104`.

## Why every check missed it

**The installer verified the wrong path by construction.** `targetDestination()`
re-ran the same occupancy test at verify time, landed on the same fallback, and
confirmed the file was correct there.

**The freshness check could not fail on this defect.** `skills-freshness.sh`,
written earlier the same day, compares one commit SHA and greps for duplicate
skill names. A renamed install produces neither symptom. It reported exit 0 on a
broken install and was presented as closing the drift class.

**The completeness claim came from a sample presented as a total.** After
reinstalling, a loop was run over
`skills/process/kbd-process-orchestrator/**/SKILL.md`, returned `25 identical,
0 different, 0 missing`, and was reported as "Step 2 verified — 0 drift." That
was 25 of 163 skills at 1 of 14 targets. The loop's scope was not a judgment
about coverage; it was the directory the investigation happened to be in.

**A later audit undercounted by 6×.** It asked "is this a symlink that
resolves?" Sixteen of the nineteen are symlinks and do resolve — into a source
checkout rather than the installed generation. The check could not tell those
apart. `isManagedSkillLink` in the installer already could.

## Root cause

Two distinct failures, and the second is the one that generalises.

**Mechanical.** Declining to clobber a foreign file is correct. Doing so while
reporting success is not. The two are incompatible and the code chose both.

**Method.** Repeatedly, the scope of an instrument was mistaken for the scope of
the problem: one directory read as the whole tree, one target as all fourteen,
one crate directory as the whole repository. The rule for guards — *a gate never
observed to fail is indistinguishable from one that always passes* — was written
into this repo's own execution contract, applied to the stage gate and the
runtime projection in the same session, and skipped for the install because the
installer had already printed a checkmark. **A tool's success output was treated
as a result rather than as a claim to be tested.**

## What changed

`prometheus-skill-system` at `b47b2a8`:

- **Collisions are recorded at all three sites and reported together** — occupant
  kind, mtime, and a per-case remediation command — then the run exits non-zero.
  `--allow-fallback` permits the rename for a genuine third-party conflict and
  still exits non-zero, because a renamed skill is unreachable either way.
- **`scripts/verify-skill-install.js`** enumerates from the generation and
  asserts every skill at every target — 163 × 14 — printing the denominator on
  every run so it cannot be accidentally scoped. Two checks are stronger than
  what existed: a symlink must resolve **into the active generation**, not merely
  somewhere under the plugin root; copy targets are hashed file-by-file rather
  than `SKILL.md` alone.
- **The gate runs as the install's own last step**, so "install ran" and "install
  is correct" are the same statement.
- **`scripts/tests/verify-skill-install.test.mjs`** plants six failure modes in
  an isolated fixture, observes each failing, repairs it, and observes it
  passing: unowned directory, absent skill, symlink outside the generation,
  dangling symlink, stale copy in a non-`SKILL.md` file, missing file in a copy.

## Verification

- 19 occupants archived reversibly to
  `~/.prometheus/skill-collisions-archive-20260813/` with a manifest and the
  original symlink targets recorded. Three were directories with content
  differing from the generation, so none was assumed redundant.
- Post-repair: **2282/2282 placements current (163 skills × 14 targets)**.
- Red/green suite: **13 passed, 0 failed**.
- `deep-research` resolves in `.claude`, `.codex`, `.opencode` as the full
  13,519-byte skill with all eight subdirectories.

## Two defects introduced while fixing this

Recorded because both were self-inflicted during the repair.

**The generation's integrity seal was broken.** An earlier `stampProvenance()`
wrote `.source-commit` and `.source-repo` *into* `current/`. That directory is a
content-addressed, signed generation whose `manifest.json` lists 1328 files and
neither of those. `install-plugin-generation.js --verify` failed for the whole
period. A strong integrity guarantee was traded for a weak freshness stamp.
Fixed by moving both files beside the generation; `--verify` passes.

**The reminder projection printed a true number under a false label** —
"Progress: 47 of 72" beneath "THIS phase's implementation counter", while the
phase was 0 of 6. `completion` is held on runtime state, not per phase. Caught
by running the rebuilt binary; the unit test asserted the file regenerates and
said nothing about whether its text was true.

## What remains

- The `--allow-fallback` path has never been exercised. It is written to exit
  non-zero, and that has not been observed.
- The gate covers the 14 declared targets. A tool reading skills from a path not
  in `TARGETS` is outside its denominator.
- A rule about exhaustive verification is **not** yet written.
  `AGENT_BASE_RULES.md` §D-6 forbids promoting a lesson to a rule from an agent's
  own evaluation of its own output without adversarial review, the sycophancy
  gate, and explicit human approval. Operator decision 2026-08-13: run the gates
  first. Until then this file is the record, not the rule.
