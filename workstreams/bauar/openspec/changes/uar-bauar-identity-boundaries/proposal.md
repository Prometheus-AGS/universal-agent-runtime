# Proposal: UAR identity boundaries

## Why

Current key issuance accepts requested roles unchanged, loses tenant, and lacks owner-bound revoke. Remote policy and role-derived host trust do not establish the approved authority boundary.

## What Changes

- Preserve verified issuer, subject, tenant, attenuated roles and typed provenance across direct key and exchange paths.
- Add explicit local/remote policy, trusted workspace mapping and bounded JWKS refresh.
- **BREAKING**: remote incomplete/legacy authority and self-minted host authority are rejected; key management requires verified authority.

## Capabilities

### New Capabilities
- uar-principal-authority: materializes the approved parent authority contract in this isolated execution home.

### Modified Capabilities
None in this new nested spec root. Original jwt-hardening, tenant-isolation and multi-tenant-isolation remain preserved outside the explicit strict remote addition.

## Impact

Child project UUID: 06d6112e-346a-4417-8296-3b0ba3dba220. Product root: /Users/gqadonis/.claude/worktrees/bauar-uar. Accepted product base: a7cb972992d4f83db6585449ea81af0fe4a1c990. Parent phase: bossfang-uar-authorization-and-execution. Only this nested workstream owns the child KBD state. Original repository KBD state and C05 delivery gates remain untouched.

Exact source inventory, dependency closure, contracts and diagnostic protocol: [UAR prerequisites](/Users/gqadonis/.codex/worktrees/bossfang-uar-architecture/prometheus-skills-mini/workstreams/bossfang-uar-architecture/.kbd-orchestrator/phases/bossfang-uar-authorization-and-execution/evidence/execute/uar-prerequisites.md).

Parent: bauar-01-identity-boundaries, numeric tasks 1–7. No dependency upgrades, database migration, new IdP, UI redesign or remote custody implementation. Only the named U-ID/U-HOST claims and exact extensions in design.md are eligible.
