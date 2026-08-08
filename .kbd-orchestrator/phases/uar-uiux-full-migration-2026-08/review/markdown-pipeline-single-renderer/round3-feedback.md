# Final review scope and remediation feedback

- The Tailwind 4, Vite, Radix icon, Chromatic, and other unrelated dependency
  hunks visible in `frontend/package.json` belong to earlier completed KBD
  changes in this long-lived uncommitted worktree. For C-08, review only the
  seven additions named in `dependency-evidence.md`; the real package hunk is
  included so their manifest presence is directly verifiable.
- Round-one and round-two actionable markdown findings were addressed:
  fenced and inline code styles are distinct; assistant-ui chain propagation
  and AST-node stripping are tested; malformed KaTeX is non-throwing;
  DOMPurify now implements the standalone raw-SVG sanitizer; unimplemented
  custom elements and autoplay/loop permissions were removed.
- Plain safe fenced code until C-09 and the spacer implementation for `hr` are
  explicit binding design decisions in `design.md` and
  `docs/ui/uar-frontend-migration-plan.md` §7. C-09 installs Shiki next.
