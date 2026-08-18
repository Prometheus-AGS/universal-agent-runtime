# Positive verification — `emit-rag-retrieval-events`

## Provenance event and citation stream

Commands:

```bash
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::rag::citation_stream::tests -- --test-threads=1
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::api::sse::tests::maps_rag_citation_with_knowledge_base_and_document_provenance -- --exact --test-threads=1
```

Observed outputs, exit 0:

```text
running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 598 filtered out; finished in 0.00s

running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 605 filtered out; finished in 0.00s
```

The focused mapping assertion requires `knowledge_base_id`, `document_id`, and
`document_name` to survive normalization into the AG-UI custom event.

## Hardened retrieval pipeline and audit event

Command:

```bash
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::rag::pipeline::tests -- --test-threads=1
```

Observed output, exit 0:

```text
running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 600 filtered out; finished in 0.00s
```

The audit test captures tracing metadata and structured fields directly. It
requires the event name `rag.retrieval.decision`, `kb_id="kb-audit"`, and
`returned_count=1`; it does not infer the audit event from a formatted message.
The limit test supplies two substantive subqueries and requires the higher-scored
match to be the sole result when the global limit is one.

## SurrealKV status transition

Command:

```bash
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::persistence::providers::surreal::tests::document_status_reaches_indexed_on_embedded_surrealdb -- --exact --test-threads=1
```

Observed output, exit 0:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 605 filtered out; finished in 0.15s
```

## PostgreSQL status transition

The test ran against an isolated temporary PostgreSQL 17 cluster on
`127.0.0.1:55439`, database `uar_rag_test`, with pgvector 0.8.6. The cluster
was stopped after the assertion.

Command:

```bash
DATABASE_URL=postgres://127.0.0.1:55439/uar_rag_test \
  cargo test --quiet --locked -p universal-agent-runtime \
  --no-default-features --features server-full,postgres-backend \
  --test knowledge_base_integration test_document_lifecycle \
  -- --exact --test-threads=1
```

Observed output, exit 0:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.11s
```

## Browser BDD

The first run could not render because the existing linked entity package had
not been built. After building its already-installed core and React packages,
the scenario exposed two fixture defects: the helper used an immediate
`isVisible` probe instead of waiting for thread creation, and the hover assertion
matched both visible and screen-reader copies of the filename. The corrected
fixture waits for the composer and scopes the filename assertion to the hover
card. No production frontend file changed.

Commands:

```bash
pnpm exec bddgen test -c tests/bdd/playwright.config.ts
pnpm exec playwright test -c tests/bdd/playwright.config.ts \
  --grep "Retrieval-influenced response" --workers=1
```

Observed final output, exit 0:

```text
Running 1 test using 1 worker
✓  1 tests/bdd/.features-gen/features/chat-kb-retrieval.feature.spec.js:6:3 › Chat with a knowledge base enabled, retrieval influencing the response › Retrieval-influenced response (6.8s)
1 passed (17.8s)
```

## Change-level checks

Commands:

```text
cargo fmt --all -- --check
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
pnpm typecheck
pnpm lint
pnpm -C frontend build
openspec validate emit-rag-retrieval-events --strict
git diff --check -- openspec/changes/emit-rag-retrieval-events src/uar/api/sse.rs src/uar/domain/events.rs src/uar/persistence/providers/surreal.rs src/uar/rag/citation_stream.rs src/uar/rag/pipeline.rs src/uar/runtime/manager.rs tests/bdd/features/chat-kb-retrieval.feature tests/bdd/steps/rag-citation.steps.ts tests/bdd/support/world.ts
```

Observed results:

```text
cargo fmt: exit 0, no output
cargo check: exit 0; 3 known warnings
cargo clippy: exit 0; 571 warnings
pnpm typecheck: exit 0 (`tsc -b`)
pnpm lint: exit 0 (`eslint .`)
frontend build: exit 0; 8040 modules transformed; built in 1.66s
OpenSpec: Change 'emit-rag-retrieval-events' is valid
git diff --check: exit 0, no output
```

Candidate SHA-256 values:

```text
769174031654a569f3e51be7d3d52bc5c7bd8aa5f61f37ad198cf5e5cbc57845  src/uar/api/sse.rs
f36083381450286906c2c436116e912a6581e45da1ca25717b22bbe373ca29a3  src/uar/domain/events.rs
0de0cf2fb32c30e94e83b23ac75aae6b8a7ac7c4602f6cdaf5f13e942427eff9  src/uar/persistence/providers/surreal.rs
ca07c4bdd6a07f404f1c5d08725290335cd19f5ac7e2aa434b48439d64814eea  src/uar/rag/citation_stream.rs
b37878358165911230864811d25096563832bfbd887d17c10b42b8ec1d6da556  src/uar/rag/pipeline.rs
6463bc42b06cbd45c7d40d5ce169e0b301cd4a3c4c36776e3de7f9801ae30d0b  src/uar/runtime/manager.rs
5c70ee2023872ebef953c99acd74c6ef0ba07b26e7ed8cd34cbff1d70d525c7d  tests/bdd/features/chat-kb-retrieval.feature
36e93db2f5e463ca01eb140551e0795ad76dc58ca81912d3862f838e3f692ea4  tests/bdd/steps/rag-citation.steps.ts
a3eff4f5458b1347d36e1399587977b75738c3b7b544a8b80edba2c9b41307bd  tests/bdd/support/world.ts
```

Phase Tier 2 remains deferred until all active-phase changes are complete.

## Independent artifact review

Observed on the final candidate:

```text
artifact critic: PASS
independent artifact judge: PASS
```

Both reviewers confirmed the nine recorded source hashes, exact focused result
counts, provenance and hardened-pipeline behavior, SurrealKV and PostgreSQL
status proofs, browser evidence, strict OpenSpec validation, and artifact schema
consistency. They also confirmed that `.claude/settings.local.json`,
`pnpm-lock.yaml`, `.refiner/registry.json`, and unrelated KBD projection churn
must remain outside the commit.

Final artifact command:

```bash
crates/prometheus-skill-system/skills/imported/artifact-refiner/scripts/state-finalize.sh \
  emit-rag-retrieval-events
```

Observed output, exit 0:

```text
.refiner/history/emit-rag-retrieval-events/2026-08-18_19-09-11Z
```

The active and archived artifact manifest, constraints, and refinement-state
documents each passed their canonical JSON schema. Their file references exist,
and both states report `converged`, a final `terminate` decision, and 4/4
constraints satisfied.

Archive and canonical-spec commands:

```bash
openspec archive emit-rag-retrieval-events -y
openspec validate rag-provenance --type spec --strict --no-interactive
```

Observed output, exit 0:

```text
Change 'emit-rag-retrieval-events' archived as '2026-08-18-emit-rag-retrieval-events'.
Specification 'rag-provenance' is valid
```
