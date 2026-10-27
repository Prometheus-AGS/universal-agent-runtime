# A21/P16.006 current-source review

Verdict: **BLOCK_SOURCE_ONLY**. One required timing boundary remains unresolved. Review stops at this finding.

## Finding

[MEDIUM] iOS settlement can return successfully after its settlement deadline

File: `scripts/acceptance/ios/runner.py:230–253`, with database access at lines 175–186.

The loop establishes its deadline at line 230, then calls `ledger_snapshot(config)` at line 232. That function opens and reads SQLite at lines 180–183 without receiving the remaining settlement allowance. When the resulting snapshot has no pending reservations, lines 236–253 validate the new terminal receipts and return without checking the deadline again. The only deadline check is in the still-pending branch at line 254.

Concrete failure: a settlement iteration begins just before its 90-second deadline and waits for a concurrent SQLite writer. The writer commits after that deadline, leaving completed reservations. The snapshot returns, terminal validation succeeds, and line 253 reports settlement success after the permitted wait. The same return path does not recheck the original overall deadline.

The shared terminal validator checks receipt identity, timestamps, status, response hash, and usage. It does not enforce the caller’s waiting deadline. The subprocess deadline likewise does not enforce the narrower 90-second settlement limit when the overall invocation has longer remaining.

Required resolution: bound snapshot access by the remaining settlement/original allowance and check both deadlines after snapshot acquisition, before returning settled success. This finding requests no execution or repair authorization.

## Scope and evidence

Reviewed the relayed current text for all 42 paths in `changed_regular_inputs_since_A15`: the acceptance controllers, fixture composer, budget forwarder, process owner, runtime observers, capability cases, preparation helpers, iOS host/project/schemas/scenarios, Swift tests, Rust ABI, Rust SDK runtime and manifest, dependency pins, vendor provenance, release/user documentation, and three authored regression modules.

Also reviewed all three supplied inert preparation configs. The artifact config’s 29,045-entry `source_files` preimage was projected to its count and selected helper rows; it was **not individually reviewed**. Cargo.lock coverage was limited to the supplied package/graph projection. Unrelated lockfile records and transitive source implementations were not reviewed.

Relevant inspected mechanisms include:

- Shared terminal validation: `budget_proxy.py:54–91`.
- Server transactional reconciliation and bounded settlement: `packet_processes.py:90–195`.
- Failure retention, owned cleanup, settlement ordering, and corrective continuation: `failure_sweep.py:208–340`.
- iOS item evidence, cleanup, correction admission, and aggregation: `ios/runner.py:350–519`.
- Serial platform execution and source-rebind blocking: `platform_suite.py:176–238`.
- Inert preparation authority and production/runtime disclaimers: `prepare_platform_inputs.py:101–162,320–359` and `prepare_supporting_inputs.py:138–184,250–306`.

No commands, tools, tests, imports, builds, network calls, or writes were performed by this reviewer. Authored tests were inspected as text only.

## Limits

Runtime behavior remains **UNVERIFIED**. Source identity `sha256:4ace47fbfffc400f775f2fc5b3b0ace29e3e41fcf46cb877d54319b271d2ea52` was supplied and was not independently certified. This review establishes no compilation, ABI, installed-artifact, provider, persistence, platform, or whole-phase acceptance result.

The verification gate remains **CLOSED**. Reported cumulative accounting remains external supplied metadata; this review does not reset allowances or authorize further work. Native same-family reviewer isolation is weaker than cross-model review.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | none reported |
| HIGH | 0 | none reported |
| MEDIUM | 1 | required boundary unresolved |
| LOW | 0 | none reported |

Verdict: **BLOCK_SOURCE_ONLY** — the iOS settlement wait does not fully enforce its required deadline.