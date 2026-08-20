# Frontend pnpm lock consistency — verification summary

Artifact type: `direct:content`
Date: 2026-08-20

## Candidate

- Source commit: `1274039a28f0072bc0e6629a9dab327bdcd9417d`
- Entity-management pin: `0352c83d7b386db56ffea8304ffdf3e2edb00fc8`
- Nested lock SHA-256: `43c00bbfe5b85e42c12a5fda74ab987750863794f00104a12ecd24a59f822593`
- Root lock SHA-256: `645e3af883e8d62b74d13be20453c083431ed3cf2ef3ca20a5b1a84152273350`

## Blocking constraints

1. `frozen-nested-lock`: PASS — pnpm 11.15.0 accepted frozen metadata and a
   1,191-package empty-tree installation without changing either lock.
2. `fail-closed-stale-control`: PASS — the committed nested lock exited 1 with
   `ERR_PNPM_OUTDATED_LOCKFILE`, naming 17 added and 12 mismatched dependencies,
   while preserving its stale digest.
3. `minimum-resolution-delta`: PASS — two clean regenerations were
   byte-identical; an exact retained patch replays the three raw-to-accepted
   restorations; the machine-readable audit classifies all 693 mutations against
   44 pinned-manifest edges with zero unclassified, zero blanket all-edge or
   empty anchors, zero common package-body mutations, and only three causal
   common snapshot-body mutations.
4. `bounded-child-scope`: PASS — manifests, root lock, gitlink, product source,
   static output, parent evidence, settings, and registry are excluded.
5. `truthful-verification`: PASS — iteration 1
   exposed four receipt defects; iteration 2 exposed a blanket causal-anchor
   fallback; iteration 3 traced every peer token but copied three pnpm importer
   section labels; iteration 4 resolves all 44 anchors against actual manifest
   sections and values. The final independent critic and judge both returned
   PASS. No parent browser, runtime, release, install-script, or cross-platform
   claim is made.

## Current hashes

- `verification.md`: `aba41bf91c1f964bd3663ca5dfd8da6d39cd98b86284fcd9c0b54cac0dc5b44b`
- delta spec: `f074b0356429f93cc9e4ab835005c7dd6ae5712f2bf0d4b968c53a600466b308`
- delta-audit receipt: `4d6f9a72430e490519711aa3f8c29d14994ceaef288a91e0ccedbf2a0dbddbdd`
- frozen-install receipt: `68b0573f511a4bec08943eed7461e96ce4956d3343b8155d827be1516147fcdc`
- delta-classification JSON: `e986720672df17d0c2c826e6b42fa630554d0405cff68b1866a6703818d2ce87`
- refiner state: `40317ff219dd533a1359ade9103ce0fedd5605f1009eb831183413796109e952`

The direct-content artifact converged only after the final independent
artifact-only critic and judge PASS decisions.
