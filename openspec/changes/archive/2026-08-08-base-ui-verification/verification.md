# Verification: base-ui-verification

## Result

PASS for the C-14d scope. The application-owned command facade now uses Base UI
Autocomplete, `cmdk` is absent, direct Radix usage remains zero, and the remaining
Radix packages are traceable to `vaul` and Assistant UI only.

## Deterministic evidence

- Root `pnpm install --frozen-lockfile --ignore-scripts`: pass across all 21 workspace projects.
- Nested `pnpm -C frontend install --frozen-lockfile --ignore-scripts`: pass across all 10 frontend projects.
- Root-authoritative `pnpm --filter uar-frontend typecheck`: pass.
- Root-authoritative `pnpm --filter uar-frontend lint`: pass.
- Manifest/root-lock/nested-lock importer parity audit: pass; all three are free of
  `cmdk` and direct `@radix-ui/*` declarations.
- Root and nested `pnpm why cmdk`: both empty.
- `node scripts/check-frontend-boundaries.mjs`: pass, zero production violations.
- `bash scripts/ci-grep-gates.sh`: pass, including boundary negative fixtures and
  Flat 2.0 at 376 tracked legacy findings with zero new findings.
- `pnpm -C frontend settings:structure`: pass, 11 modules and all 29 keys preserved.
- Root-authoritative `pnpm --filter uar-frontend test`: pass, 69 files and 330 tests.
- `pnpm -C frontend build:manifest`: pass, 8,032 modules.
- `pnpm -C frontend budget:bundle`: pass at 217,476 / 250,000 gzip bytes, 32,524
  bytes of headroom. This is 13,957 bytes below the C-14c entry result.
- `pnpm -C frontend exec playwright test e2e/chat-agent-selection.spec.ts --workers=1`:
  pass, 2/2. The selector opens, filters two agents, selects the highlighted result
  with Enter, updates its trigger, and closes.
- `pnpm -C frontend exec playwright test e2e/runtime-console-visual.spec.ts --grep
  "command palette" --workers=1`: pass, 1/1. The palette filters and navigates with Enter.
- `pnpm -C frontend exec playwright test -c playwright.performance.config.ts --workers=1`:
  pass at 995.5 / 1,000 ms under the required serial production-preview profile.
- `openspec validate base-ui-verification --strict`: pass.
- `openspec validate frontend-component-primitives --strict`: pass.

## Broad E2E classification

The required `pnpm -C frontend test:e2e` probe ran in the default no-backend profile:
36 passed, 4 skipped, and 8 failed. None exposed a command-facade defect.

- The agent-selector failure came from a nondeterministic guard probe against an absent
  backend. It was replaced with controlled API routes and stronger filter/keyboard
  assertions; both focused cases now pass.
- `chat-basic` and `chat-no-provider` inspected async guard state before the model check
  settled. Their failure snapshots show the valid `No Model Configured` guard.
- `knowledge-rag-real` and `provider-route-real` require the dedicated real-server profile;
  the default run had no backend on port 1906.
- The performance spec ran concurrently under the default config and reported 1,055.9 ms
  and 1,027.2 ms. Its binding serial production-preview profile passes at 995.5 ms.
- Two runtime-replay assertions (`runs` detail and `tool_call.delta`) also fail in an
  isolated serial rerun while three sibling replay cases pass. They predate and do not
  import or exercise the changed command facade; they remain explicit C-15 evidence.

## Dependency audit

- `cmdk`: absent from source, manifest, root and nested lockfiles, and both root and
  nested `pnpm why` graphs. The initial artifact-only review found and blocked on a
  stale root importer; regenerating the authoritative root lock removed four packages.
- Direct `@radix-ui/*` imports and dependencies: zero.
- `vaul@1.1.2`: owns Dialog 1.1.15 in the nested graph and is deduplicated to
  Dialog 1.1.19 in the root graph.
- `@assistant-ui/react@0.14.26`: owns `radix-ui@1.6.2`, including
  `@radix-ui/react-dialog@1.1.19` and `@radix-ui/react-tooltip@1.2.12`.
- `@prometheus-ags/prometheus-entity-management@3.0.0-alpha.0`: no Radix dependency.
- The current Assistant UI release checked during implementation still declares Radix;
  upgrading would not satisfy the removal goal and was not introduced.

## Review remediation and attribution limits

- A repeated-selection facade test now activates Alpha and then Beta without replacing
  the empty action query; the same mounted host handles both actions.
- Registry metadata was refreshed to Assistant UI 0.15.10.
- The Chromatic import migration and broad runtime-console navigation rewrite visible in
  the branch diff predate C-14d. C-14d owns only the strengthened agent selector and
  command-palette filter/keyboard assertions in those files. Because those files did not
  have per-change entry hashes, this attribution is recorded as operator-session history,
  not independently reproducible proof.
- No human manual-interaction receipt is claimed; automated acceptance evidence is mapped
  explicitly in `tasks.md`, and real-backend gaps remain for C-15.

## Scope integrity

The exact protected-path closeout hash is
`07e74ad94dc137e9574e411bc99d6f0fcd631879c5a0e52a1b87ca999cf43dc4`, identical to
entry. No staging or commit was performed. No new security boundary was introduced.
