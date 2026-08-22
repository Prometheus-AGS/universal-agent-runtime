# Root pnpm lock consistency verification summary

Scope: root pnpm workspace on macOS with pnpm 11.15.0. Install verification
disabled lifecycle scripts. Results transfer to no browser, runtime profile,
release candidate, deployment, or other platform.

- Frozen root lock: PASS. Lock-only and clean full frozen installation exit 0,
  validate 1,482 supply-chain entries, link 1,345 packages, and preserve
  `645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`.
- Stale negative control: PASS as a fail-closed control. Clean source
  `fa4ffb96af63131b831c4f30a1b2c16aca599808` exits 1 with
  `ERR_PNPM_OUTDATED_LOCKFILE`, 17 additions, and 12 mismatches.
- Minimum delta: PASS after correction. Direct HEAD audit restores the
  noncausal config-array/minimatch and y-webrtc/ws edges while retaining the
  changed sync importer's direct ws 8.21.1 pin. Two clean regenerations agree
  but move lucide-react and collapse the preserved y-webrtc edge.
- Source scope: PASS locally. The entity-management Git link remains
  `0352c83d7b386db56ffea8304ffdf3e2edb00fc8`; no manifest or product source is
  part of the child repair; denied dirty paths are explicit commit exclusions.
- Tier 0: PASS. `pnpm typecheck && pnpm lint` exits 0.
- OpenSpec: PASS. Strict validation exits 0 before review.
- Parent limit: no prior browser bundle can be promoted. The parent must mint
  fresh certification from the new committed source after this child exits.

Independent artifact review: PASS after correction. The first critic and judge
reviews BLOCKED the false minimum-delta claim and incomplete receipts. Fresh
history-free critic and judge reviews independently replayed the corrected
hash, causal delta, schemas, chronology, scope, and command/output provenance
and found no remaining blocker.
