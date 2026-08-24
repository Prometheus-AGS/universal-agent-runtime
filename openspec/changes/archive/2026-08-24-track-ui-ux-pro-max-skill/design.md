## Context

See `proposal.md` for motivation. The installer placed the canonical payload at `.agents/skills/ui-ux-pro-max/`, created five relative tool links, and wrote `skills-lock.json`. The repository currently ignores all of `.agents/`, while the committed UI/UX routing block already mandates UI/UX Pro Max and points to a roster whose catalog counts predate this installed snapshot.

The payload is third-party MIT-licensed content from `nextlevelbuilder/ui-ux-pro-max-skill`; the installed subtree omitted the upstream root license file.

## Goals / Non-Goals

**Goals:**

- Make a fresh checkout contain one operational canonical skill payload.
- Preserve the existing machine-local `.agents/` policy outside this named subtree.
- Keep tool entry points lightweight and non-duplicative.
- Make the existing mandatory routing instruction resolve to the local payload through its durable roster.

**Non-Goals:**

- Modify UI components or make new visual-design decisions.
- Regenerate the installed skill, alter its data, or claim a newer upstream snapshot than the installer lock identifies.
- Add tool links that the installer did not create or whose discovery semantics have not been verified.
- Hand-edit the auto-managed UI/UX fenced regions in `AGENTS.md` and `CLAUDE.md`.

## Decisions

### Track the canonical payload under `.agents/`

Add a narrow `.gitignore` exception for `.agents/skills/ui-ux-pro-max/` while leaving all other `.agents/` content ignored. This preserves the installer's single-source layout and the path already discovered by Codex.

Copying the full payload into every tool directory was rejected because it multiplies a data-heavy skill and allows copies to drift.

### Track only installer-created relative links

Commit the existing `.claude`, `.kilocode`, `.qwen`, `.roo`, and `.windsurf` relative links. They resolve within any checkout and all target the canonical payload. No new `.codex`, `.cursor`, or `.opencode` link is invented; those tools can consume the repository-level `.agents` skill or the standing AGENTS instructions according to their own discovery behavior.

### Preserve reproducibility and licensing metadata

Track `skills-lock.json` with the upstream source and a computed hash of the repository-owned payload. Add the upstream MIT license inside the vendored skill subtree because the installer omitted the repository-root license from its selected payload. Preserve upstream bytes and disable Git whitespace diagnostics only for this vendored path; first-party whitespace checks remain active.

### Update the durable roster, not generated instruction blocks

The `uiux-routing` fenced regions already require UI/UX Pro Max. Update `.kbd-orchestrator/references/uiux-skill-roster.md`, which those regions explicitly reference, to name the canonical local path, current verified counts, and smallest-query-mode contract from the installed `SKILL.md`.

## Risks / Trade-offs

- [The vendored catalog is large] → Keep one canonical copy and links; verify the lock hash and data validator.
- [A broad unignore exposes machine-local agent state] → Use parent re-inclusion rules that expose only the named skill subtree.
- [Symlinks break on another checkout] → Require relative link targets and verify every tracked link resolves to the canonical directory.
- [Managed instruction text is overwritten later] → Leave it untouched and update the durable roster it already references.
- [Upstream advances after installation] → Record the installed computed hash; updates require an explicit reinstall/review rather than an automatic refresh.
- [The upstream snapshot contains trailing whitespace] → Preserve the vendored bytes and scope Git's whitespace exemption to this one third-party path.

## Migration Plan

1. Add the narrow ignore exception and upstream license.
2. Track the canonical payload, lockfile, and installer-created links.
3. Refresh the roster's local-path and catalog facts.
4. Run integrity, representative search, link-resolution, instruction-reference, OpenSpec, and Git checks.
5. Archive the verified OpenSpec change and commit only scoped files.

Rollback removes the tracked payload/links/lock and the narrow ignore exception while leaving other agent state untouched.
