## 1. Repository Ownership

- [x] 1.1 Add a narrow `.gitignore` exception for `.agents/skills/ui-ux-pro-max/`; verify the skill subtree is trackable and an unrelated `.agents/` probe remains ignored
- [x] 1.2 Add the upstream MIT license and track the canonical skill payload plus `skills-lock.json`; verify Git enumerates `SKILL.md`, data, references, scripts, license, upstream source, and computed hash
- [x] 1.3 Track the installer-created tool links; verify each is relative and resolves to the canonical repository payload

## 2. Instruction Routing

- [x] 2.1 Update the durable UI/UX skill roster with the canonical local path, current catalog counts, and query contract; verify AGENTS.md and CLAUDE.md still point to the roster and mandate UI/UX Pro Max analysis
- [x] 2.2 Verify Codex can discover the canonical `.agents` skill, Claude resolves its tracked link, and Cursor/OpenCode receive the mandatory routing through AGENTS.md without adding unverified tool links

## 3. Verification and Archive

- [x] 3.1 Run the skill data validator, the 130 installed-payload-compatible Python tests, and a representative React stack search; verify all commands succeed, record the two upstream-layout-only test omissions, and confirm the search returns React guidance
- [x] 3.2 Run strict OpenSpec validation, link/path checks, a conflict-marker scan, and `git diff --check`; record the observed results in `verification.md`
- [x] 3.3 Sync the `uar-uiux-skill-routing` delta, verify archive readiness, and prepare only scoped files for commit; verify pre-existing untracked KBD context and `versions.toml` remain untouched
