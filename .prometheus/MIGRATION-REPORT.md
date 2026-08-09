# Migration report — v3 base rules to bootstrapped structure

Generated 2026-08-09. Source: `AGENTS.md` (4720 words).
Archive: `.prometheus/knowledge/AGENTS.pre-migration-2026-08-09.md`

The archive is the authority for anything below. Nothing was deleted.

## Coverage

49 of the mapped rule IDs were present in the source; 0 were not.
"Present" means the ID appeared in the source and its content is covered by the
destination. It does not mean the destination is byte-identical — most rules
were condensed, and several moved from prose to enforcement.

| v3 ID | In source | Destination | Note |
|---|---|---|---|
| `§0` | present | .claude/hooks/reanchor.sh + AGENTS.md "Position and authority" | session bootstrap becomes a hook, not prose |
| `A-1` | present | AGENTS.md "Scope" + scaffold "Before executing" | think-first is scaffold on non-frontier models |
| `A-2` | present | AGENTS.md "Evidentiary standard" | verbatim intent, condensed |
| `A-3` | present | AGENTS.md "Evidentiary standard" (boundary exception) | kept as standing exception |
| `A-4` | present | AGENTS.md "Scope" | surgical diff |
| `A-5` | present | AGENTS.md "Evidence over assertion" + scaffold "Do not fabricate" | split: principle resident, mechanics in scaffold |
| `A-6` | present | AGENTS.md "Evidence over assertion" | verified vs self-reported |
| `A-7` | present | AGENTS.md "Scope" | preserve behavior |
| `A-8` | present | AGENTS.md "Phase order" | architecture before code |
| `A-9` | present | .claude/rules/<stack>.md + .claude/hooks/tier-guard.sh | ENFORCED, no longer advisory |
| `A-10` | present | .claude/hooks/single-writer.sh + rules-rust.md | ENFORCED |
| `A-11` | present | AGENTS.md "Scope" (irreversible actions) |  |
| `A-12` | present | structural: human gates in hooks + artifact-critic |  |
| `A-13` | present | scaffold "Self-check before reporting completion" | scaffold only; frontier models supply this |
| `A-14` | present | AGENTS.md "Architecture" (no hidden state) |  |
| `B-1` | present | AGENTS.md "Architecture" (open standards) |  |
| `B-2` | present | AGENTS.md "Architecture" (feature-based) |  |
| `B-3` | present | AGENTS.md "Architecture" (strict layering) |  |
| `B-4` | present | .claude/rules/typescript.md, rules-flutter.md | layer responsibilities are stack-specific |
| `B-5` | present | AGENTS.md "Architecture" | UI is a projection of state |
| `B-6` | present | dropped | language-invariance is implied by having per-stack rules files |
| `B-7` | present | .claude/rules/<stack>.md | strong typing is a per-language concern |
| `B-8` | present | AGENTS.md "Architecture" (portability) |  |
| `C-1` | present | .claude/rules/<stack>.md | tier philosophy lives with the ladders |
| `C-2` | present | AGENTS.md "Evidence over assertion" |  |
| `C-3` | present | AGENTS.md "Scope" | small reviewable changes |
| `D-1` | present | AGENTS.md "Learning and memory" |  |
| `D-2` | present | AGENTS.md "Learning and memory" | write events |
| `D-3` | present | AGENTS.md "Learning and memory" + reanchor.sh | hook surfaces recent gotchas |
| `D-4` | present | AGENTS.md "Learning and memory" | surreal-memory fallback |
| `D-5` | present | .prometheus/ curation, human-run | noise control is not an agent behavior |
| `D-6` | present | structural: artifact-critic + human gate |  |
| `E-1` | present | AGENTS.md "Anti-sycophancy" + .claude/hooks/sycophancy-gate.sh | ENFORCED |
| `E-2` | present | .claude/agents/artifact-critic.md | ENFORCED structurally by subagent isolation |
| `E-3` | present | AGENTS.md "Anti-sycophancy" | when review is required |
| `E-4` | present | AGENTS.md "Anti-sycophancy" + artifact-critic | reflection contract |
| `E-5` | present | .claude/hooks/sycophancy-gate.sh | graceful degradation implemented in the hook |
| `F-1` | present | AGENTS.md "Skills may be absent" |  |
| `F-2` | present | AGENTS.md "Skills may be absent" |  |
| `F-3` | present | AGENTS.md "Skills may be absent" |  |
| `F-4` | present | .claude/hooks/reanchor.sh | compaction re-anchor is a hook |
| `F-5` | present | AGENTS.md "Phase order" | M1-first folds into gating expensive work |
| `G-1` | present | AGENTS.md "Architecture" (verify versions) |  |
| `G-2` | present | structural: .claude/rules/ and project sections | repo rules override base |
| `G-3` | present | AGENTS.md "Learning and memory" + .prometheus/ | auditability |
| `G-4` | present | rules-rust.md + .claude/hooks/single-writer.sh | worktree coordination |
| `APPENDIX-A` | present | .claude/rules/rust.md, typescript.md, flutter.md, go.md, python.md | one file per stack, loaded on file read |
| `APPENDIX-B` | present | .claude/agents/artifact-critic.md | taxonomy moves into the critic that applies it |
| `APPENDIX-C` | present | .prometheus/ directory itself | the schema becomes the structure |

## What became enforcement rather than prose

These stopped being advisory. A hook now blocks the action, so the prose that
asked for the same behavior is redundant and was not carried over.

| v3 ID | Enforced by |
|---|---|
| A-9 tier discipline | `.claude/hooks/tier-guard.sh` |
| A-10 single-writer | `.claude/hooks/single-writer.sh` |
| E-1, E-5 sycophancy gate | `.claude/hooks/sycophancy-gate.sh` |
| E-2 critic isolation | `.claude/agents/artifact-critic.md` |
| §0, F-4 bootstrap and re-anchor | `.claude/hooks/reanchor.sh` |

Verify they actually fire. A hook that is installed but not wired into
`settings.json` enforces nothing, and `verify.sh` checks exactly that.

## Project-added content — REQUIRES A HUMAN

6 heading(s) in the source are not part of the canonical v3 skeleton.
A script cannot tell whether these are project rules that must survive or notes
that have expired. They were **not** carried into the new file.

Read each one in the archive and decide: move it into the managed region's
project section, into a `.claude/rules/` file, into a skill, or drop it.

| Archive line | Heading |
|---|---|
| 4 | Active production-completion execution lock |
| 20 | Build, Lint & Test |
| 29 | Code Style & Conventions |
| 36 | Architecture & UI |
| 55 | OpenSpec workflow |
| 83 | Worktree convention |

## Verify the migration

```bash
bash scripts/verify.sh --path .
```

Then re-run a fixed task set and compare pass rate against the archive-era
baseline. Word count going down is not evidence that anything improved.
