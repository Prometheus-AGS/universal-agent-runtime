## Why

UAR retains architecture decisions across ADRs, OpenSpec changes, KBD phase
records, and append-only Prometheus history, but readers currently have no public
way to distinguish current authority from an accepted decision that was later
reversed. That makes old AGPL, HTMX, purple-theme, placeholder-publication,
AWS-LC, GitHub Actions testing, and synthetic-soak positions easy to repeat as
current guidance.

## What Changes

- Publish a reviewed architecture-history section with an ADR index, dated
  timeline, explicit corrections, and an explanation of KBD/OpenSpec provenance.
- Add a machine-readable history manifest that maps each public synthesis to its
  retained evidence and present authority without copying raw private records.
- Preserve reversals and uncomfortable consequences instead of rewriting the
  project as a straight-line success story.
- Reject direct wiki sourcing, missing supersession links, raw log/payload copies,
  machine-local paths, and unclassified historical claims through local controls.
- Keep current product behavior authoritative in the existing architecture and
  product guides; history explains how those boundaries were reached.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `documentation-publication-contract`: Require traceable, reviewed public
  synthesis of architecture history while raw Prometheus/KBD records remain
  private-synthesis-only.
- `dev-portal-2026`: Add stable public history guides, a complete ADR index, and
  explicit current-versus-superseded decision labels.

## Impact

- **Documentation:** history pages under `website/docs/history/`, the retained ADR
  index, a history manifest, and bounded local validators.
- **Runtime/UI:** no runtime or React application behavior changes.
- **Dependencies/APIs:** no package, lockfile, dependency, schema, or public API
  changes.
- **Publication:** no raw `.prometheus`, KBD event stream, session transcript,
  machine path, credential, or unreviewed wiki record becomes public content.
- **KBD:** after local content evidence passes, advance to
  `publish-uar-testing-methodology-history` without running the final site build.
