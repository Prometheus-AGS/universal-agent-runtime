# Verification: track-ui-ux-pro-max-skill

## Repository ownership

| Check | Observed result |
|---|---|
| Canonical payload | Git index contains 71 files under `.agents/skills/ui-ux-pro-max/`, including `SKILL.md`, data, references, scripts/tests, and `LICENSE` |
| Ignore boundary | `.agents/skills/ui-ux-pro-max/SKILL.md` is not ignored; `.agents/other-state/probe` remains ignored by `.agents/*` |
| Installer metadata | `skills-lock.json` identifies `nextlevelbuilder/ui-ux-pro-max-skill`, source type `github`, the `.claude` entry point, and the verified 71-file payload hash `488bb8d098d328a3c7925477eac390463907c1b825d5958ea84c1d3ef023ffd3` |
| Upstream license | GitHub repository metadata reported SPDX `MIT`; the official upstream `LICENSE` text was added to the canonical subtree |
| Tool links | Five Git mode `120000` relative links (`.claude`, `.kilocode`, `.qwen`, `.roo`, `.windsurf`) all resolve to the canonical `.agents` directory |
| Unrelated state | `.kbd-orchestrator/phases/fix-broken-session-configuration-ui/prior-context.md` and `versions.toml` remain untracked and unstaged |

## Instruction routing

| Check | Observed result |
|---|---|
| Durable roster | Names `.agents/skills/ui-ux-pro-max/SKILL.md`, lock metadata, MIT license, current counts, and the smallest-query-mode contract |
| AGENTS/CLAUDE routing | Both files point to `.kbd-orchestrator/references/uiux-skill-roster.md` and require “UI/UX Pro Max analysis” as ordered step 2 |
| Tool discovery | `npx skills list --json` reports the canonical `.agents` payload for Codex, Claude Code, Cursor, OpenCode, and the other supported agents; all five installer-created links resolve |

## Skill operation

| Command | Observed result |
|---|---|
| `python3 .agents/skills/ui-ux-pro-max/scripts/validate_data.py` | PASS — validated 12 domain files, 22 stack files, and `ui-reasoning.csv` |
| Installed-payload-compatible `unittest` modules | PASS — 130 tests in 3.905 seconds |
| `search.py "rerender memo list" --stack react` | PASS — three React 19.2.x results; first result was measured `React.memo` guidance |

The unfiltered discovery run found 132 tests but two modules failed during import. `test_catalog_refresh.py` requires upstream repository refresh scripts that are not part of the installed skill payload, and `test_relevance_evaluator.py` requires the upstream `scripts/evaluate-relevance.py`. The vendored tests were preserved unchanged; the 130 tests that exercise the installed payload passed.

## Final gates

| Check | Observed result |
|---|---|
| Strict change validation | `openspec validate track-ui-ux-pro-max-skill --strict` passed |
| Main spec validation | `openspec validate --specs` passed all 104 specifications |
| Delta sync | The added requirement and all six scenarios are present in both the delta and `openspec/specs/uar-uiux-skill-routing/spec.md` |
| Git integrity | `git diff --cached --check` passed; the unmerged-file list and real conflict-marker prefix scan were empty |
| Vendored whitespace | `.gitattributes` disables whitespace diagnostics only for the byte-preserved third-party skill path; first-party paths retain Git's normal checks |
| Link and routing checks | 71 tracked payload files, five resolving relative links, and two managed instruction entry points were observed |
| Ignore boundary | The canonical skill is trackable and an unrelated `.agents/` probe remains ignored |
| Unrelated state | The pre-existing KBD `prior-context.md` and `versions.toml` remain untracked and unstaged |

## Verification report

| Dimension | Status |
|---|---|
| Completeness | 8/8 tasks; one requirement and six scenarios implemented |
| Correctness | 1/1 requirement and 6/6 scenarios covered by repository, CLI, validator, search, and path evidence |
| Coherence | Canonical `.agents` payload, installer links, lock metadata, roster routing, and OpenSpec contract follow the design |

No critical issues or design divergences were found. The two upstream-layout-only test modules remain the single disclosed limitation; all 130 installed-payload-compatible tests pass. Ready for archive.
