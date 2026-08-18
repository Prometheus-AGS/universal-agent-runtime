## Context

The shipped surface is the 20-screen inventory in
`docs/product-surface-inventory.md`: `/threads`, `/about`, and the 18 admin
sections selected by `frontend/src/pages/admin-page.tsx`. The repository
already has a `playwright-bdd` runner under `tests/bdd/`, a deterministic stub
LLM, an ephemeral SurrealKV backend, Cucumber JSON output, Playwright video
capture, and a local `bdd-video-proof` bundle path. This change extends that
system; it does not introduce another browser runner.

The validation environment is local and deterministic. It exercises the real
Vite frontend, Axum server, embedded database, API routes, SSE stream, PGlite
browser database, and stubbed LLM boundary. Evidence is scoped to that profile
and does not claim external-provider availability or production deployment
health.

## Goals / Non-Goals

**Goals:**

- Give each of the 20 screens an explicit purpose, primary-function assertion,
  Cucumber scenario, Playwright video, and result row.
- Prove chat agent selection, deterministic orchestrator/default-agent answers,
  skill activation, RAG citation rendering, and memory-level evidence.
- Prove the offline banner, PGlite reload persistence, and SSE resynchronization
  in the browser.
- Bind JWT and two-user isolation evidence to verified subjects while keeping
  the general screen suite deterministic.
- Mint a local certification bundle containing Cucumber JSON, MP4 videos,
  screenshots where requested, and SHA-256 metadata.

**Non-Goals:**

- No UI redesign or adjacent cleanup. Product behavior changes are limited to
  the smallest repair for a primary-function failure observed by this suite.
- No external LLM, IPFS, cloud deployment, or cross-browser certification.
- No claim that a route-load assertion alone proves a mutating screen.
- No attempt to hide an unsupported screen or convert an experimental surface
  into a stable one through test wording.

## Decisions

### 1. Extend the existing `tests/bdd` Playwright-BDD runner

Add Gherkin features and thin TypeScript steps to the existing runner. Keep
`video: 'on'`, one worker, the stub LLM, and the fresh SurrealKV path. This
preserves the already-observed startup and build behavior and avoids a new
dependency. A separate direct Cucumber runner was rejected because it would
duplicate browser lifecycle, video, reporting, and server orchestration.

### 2. One recorded scenario per screen

The validation matrix names each route, purpose, primary action, observable
result, scenario title, and video artifact. Mutating screens operate on unique
BDD fixtures in the ephemeral database and clean up where the UI exposes that
action. Read-only screens must observe API-backed or replayed state; merely
finding a heading is insufficient. `/admin/a2ui-testing` is validated in the
Vite development profile because the route is intentionally development-only.

A single scenario outline was rejected because a failure would obscure which
screen lacks evidence and many reporters would attach one video to the whole
outline execution rather than an independently reviewable purpose/function
record.

### 3. Split general-screen and security evidence without weakening either

The general screen/chat suite keeps `jwt_required=false` while still using a
configured signing secret; this matches the existing BDD harness and lets the
browser exercise every screen without inventing a login layer that the product
does not have. JWT-required and two-user isolation assertions use Playwright's
browser-owned request context against a dedicated server profile and verified
tokens, with a visible browser evidence page for the recorded result. Anonymous
and cross-user requests are the required failing controls. Existing Rust live
integration coverage may corroborate those boundaries but cannot replace the
browser-run evidence required here.

### 4. Reuse deterministic fixtures for agent, skill, RAG, memory, and events

The stub LLM fixtures provide exact expected answers. The suite creates agents,
skills, and knowledge bases through real APIs and drives the visible chat UI.
Runtime SSE assertions inject or trigger known events through existing test
hooks and verify the live surface changes without reload. Local-first assertions
create a thread/message, reload the page, and observe the same PGlite-backed
state; offline mode must expose the shipped offline banner.

### 5. Bind the certification bundle to a clean implementation commit

BDD source and runner changes are committed before the certification run. The
bundle is then minted against that clean Git SHA and committed as evidence in a
second commit for this OpenSpec change. This avoids falsely binding video to a
parent commit that did not contain the executed scenarios. The final report
records both the tested source SHA and the evidence commit.

## Risks / Trade-offs

- **Twenty independent videos increase repository size** → keep each scenario
  focused on one primary function and remux WebM to MP4 without re-encoding.
- **A shared server can leak fixtures between scenarios** → retain one worker,
  unique names/IDs, explicit cleanup, and a fresh embedded store per run.
- **A video can look successful while the assertion is weak** → the report
  links the video to the exact Cucumber scenario and records the observed state
  transition, not only the route or heading.
- **Security evidence can become theatrical if the UI cannot authenticate** →
  use the browser-owned request context with verified JWTs and pair every
  allow assertion with anonymous/cross-user denial; do not claim a login UI.
- **The development-only A2UI test screen is not a production screen** → label
  it developer-preview evidence and do not transfer its result to production.
- **The existing product may fail a primary-function scenario** → stop on the
  observed defect and report it before expanding scope; once the operator has
  authorized plan correction, record the deviation and repair the smallest
  bounded product surface rather than relax the scenario or check the task.

## Migration Plan

1. Add the 20-row validation matrix and BDD scenarios/steps using existing
   fixtures and selectors.
2. Dry-run generation to prove every step is bound, then run the focused suite
   against the ephemeral local profile.
3. Commit the executed BDD source so the evidence SHA is immutable.
4. Mint and validate the local `product-screens` certification bundle.
5. Record per-screen results and limits, pass independent artifact review, sync
   the capability spec, archive the OpenSpec change, and transition KBD.

Rollback removes the added BDD scenarios, step helpers, matrix, and bundle; no
runtime migration or customer data is involved.

## Open Questions

None. Any screen whose primary function cannot be exercised through the current
product surface is an observed blocker, not permission to substitute a route
smoke check.
