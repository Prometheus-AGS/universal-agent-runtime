## Context

See `proposal.md` for motivation. `frontend/package.json` declares
`@prometheus-ags/prometheus-entity-management` as `workspace:*`; both lockfiles
therefore resolve the checked-out `3.0.0-rc.1` workspace package. Registry
release `3.0.2` declares `@prometheus-ags/entity-graph-core ^3.0.2` as a peer.
The exact tarballs and signed `v3.0.2` source were inspected during Analyze.

## Goals / Non-Goals

**Goals:**

- Make both supported UAR install roots resolve the same published 3.0.2 pair.
- Keep one core singleton and preserve the existing platform facade.
- Make the registry-versus-workspace choice mechanically inspectable.

**Non-Goals:**

- Editing or removing the checked-out entity-management workspace sources.
- Claiming that 3.0.2 fixes atomic list ingestion; the upstream change owns that defect.
- Changing UAR entity behavior in this change.

## Decisions

### Pin the product and its peer exactly

Set both `@prometheus-ags/prometheus-entity-management` and
`@prometheus-ags/entity-graph-core` to `3.0.2` in `frontend/package.json`.
Declaring the peer directly makes singleton ownership explicit and prevents a
peer auto-install from drifting within `^3.0.2`. A caret range was rejected
because it would not enforce the reviewed code.

### Preserve the vendored workspace but exclude it from product resolution

Do not edit `frontend/packages/prometheus-entity-management/**` or workspace
membership. An exact `3.0.2` dependency cannot be satisfied by the workspace's
`3.0.0-rc.1` package, so pnpm selects the registry artifact. Removing the
workspace would affect unrelated package tooling and is outside this change.

### Reconcile both lockfile authorities

Regenerate the root and nested frontend lockfiles from their corresponding
workspace roots. Verification inspects importer entries, package snapshots,
integrity metadata, and `pnpm list`/`pnpm why` output to prove that neither app
dependency is a `link:` target and that one 3.0.2 core instance satisfies the
React package peer.

## Risks / Trade-offs

- **Peer duplication caused by an incomplete lock update** → Declare core directly and verify the resolved dependency graph from both roots.
- **Local source changes appear to affect the app but are no longer consumed** → Document the registry boundary and retain the platform-facade check.
- **3.0.2 retains the list-ingestion defect** → The next UAR change uses only the bounded configured set; the separate upstream track fixes the general defect.

## Migration Plan

1. Change the two exact frontend dependency declarations.
2. Regenerate both lockfiles without modifying workspace package source.
3. Verify registry integrity, singleton resolution, facade imports, typecheck, and production build at the change's allowed tier.
4. If resolution is incorrect, revert only the manifest and lockfile change; no data migration is involved.
