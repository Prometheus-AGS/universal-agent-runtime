# Independent scoped source review

Verdict: PASS — no blocking correctness or security finding in the scoped repair. One MEDIUM packet accuracy finding remains below. This is source-review approval only, not runtime acceptance or permission to run a gate.

Scope: `handoffs/runtime-repair.patch`, `handoffs/bdd-repair.patch`, the five affected current source files, direct startup/registry/model-capture/storage callers, the existing TEAM-01 scenario and literal source-binding/build/activation/BDD packet. Phase: `agent-team-concurrency-repair`, task 1.4. No producer discussion or implementation handoff narrative was used.

## Findings

[MEDIUM] Packet omits embedding attempts caused by startup reconciliation

File: `verification-packet.md:5`.

Issue: The statement “This packet makes no new embedding calls” does not match the newly reachable configured-root reconciliation path. When a configured skill is missing or changed in durable storage, `src/uar/runtime/skills/service.rs:422` calls `register_checked(configured)`. `src/uar/runtime/skills/registry.rs:87` calls `embedding_for`, whose implementation at line 196 calls `VectorMatcher::embed_batch`. That invokes the selected backend; `src/uar/rag/embeddings/openai.rs:102` constructs the embedding POST and line 106 sends it. Startup supplies this registry with the configured vector matcher (`src/server.rs:1747`). The new provider filtering makes this path reachable for the server's `filesystem-skills-{index}` providers.

The retained activation config declares backend `openai`, model `not-exercised`, base URL `http://127.0.0.1:1/v1`, and a configured key (only its presence was inspected). Thus the config does not disable attempts: a missing/changed skill can cause a loopback embedding HTTP attempt. `embedding_for` catches failure and continues persistence with an empty vector, so its best-effort handling does not prevent that effect. Persisted embedding settings can also overlay the config; their live values were not queried.

Fix: Correct the packet's effect description to distinguish attempted loopback embedding requests from paid-provider embedding use, or, if zero embedding attempts is a strict requirement, revise the reconciliation implementation within a newly approved scope. This review does not claim a paid embedding request occurred, and does not recommend opening an additional runtime scenario.

## Source conclusions and limits

- Configured-root reconciliation preserves the durable enabled/scope settings and non-configuration ownership; later configured roots retain overwrite precedence. API-owned dynamic files are excluded by provider identity.
- The checked CAS response preserves typed failures; only a typed transaction conflict followed by an observed changed catalog generation becomes a CAS miss. The locally vendored `Response::check` implementation was inspected directly.
- Omitted binding profile/settings revision is supported by the current admission resolver: it captures current registry settings, which subsequent member resolution compares against the immutable admission receipt.
- The existing oracle checks distinct roots/members, brackets one writer-running observation with math-running observations, checks successful independent artifacts and settled calls, and verifies selected artifact IDs in the final writer context receipt. No new scenarios were proposed.
- Activation inspects the retained process command before signalling it, avoids force-kill, waits for exit, checks store handles, preserves a fresh backup and retains candidate receipts. No process, service, database, or provider action was performed by this reviewer.
- Only read-only source/artifact inspection ran (`git diff --staged --stat`, `git diff --stat`, `cat`, `sed`, `rg`, `nl`, `ls`, workspace metadata, and Python JSON reads restricted to non-secret configuration fields). Observed source reads establish the call paths above; they do not establish executable behavior.
- No tests, lint, format, typecheck, compiler probe, build, source-manifest validation, readiness call, BDD command, or external gate ran. Compilation, exact source-manifest matching, live process/store state, effective persisted embedding settings, JWT validity, concurrency behavior and final article acceptance remain unverified by this reviewer.
- The only file written is this review. No source edits, guards, tests, fixtures, or unrelated changes were added. No fixes were attempted.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 0 | pass |
| MEDIUM | 1 | info |
| LOW | 0 | pass |

Verdict: APPROVE (PASS) — no blocking source finding; correct the packet's embedding-effect description before relying on its zero-call assertion.
