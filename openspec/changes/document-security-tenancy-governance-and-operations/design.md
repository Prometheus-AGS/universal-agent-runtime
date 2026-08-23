## Context

See `proposal.md` for motivation and `specs/dev-portal-2026/spec.md` for the
behavior contract. The current portal inventory reserves six operator routes
for this slice: authentication, credentials, approvals, runtime console, runs,
and cost. Current source also exposes narrower contracts for tenant-aware A2A,
Cedar governance, Prometheus/OTLP telemetry, multiplexed SSE, persistence,
backup, cancellation, and deadline-bound shutdown.

The uncomfortable constraint is that these controls do not form one universal
security boundary. `server-full` compiles Cedar; `minimal` uses the explicit
permit facade; embedded hosts supply their own transport and persistence.
Verified tenant identity currently partitions A2A task/context state, while
other resources use user, agent, session, or deployment scopes. Server policy
load errors fall back to permit-all. Cost budgets and several runtime-console
views are process/browser-session state. Public documentation must expose those
limits rather than normalize older, broader claims.

This is documentation-only. It owns security, tenancy, governance, and
operations guide roots plus a bounded manifest and validator. It does not own
runtime/React behavior, shared site navigation/theme, the frozen screen route
inventory, README reconciliation, dependency changes, raw history, or Pages
deployment. Full build and browser certification remain deferred until all
content slices are complete.

## Goals / Non-Goals

**Goals:**

- Establish an eleven-guide operating sequence covering authentication through
  recovery without collapsing distinct security or state boundaries.
- State exact fail-closed behavior alongside every known permissive fallback,
  profile exclusion, and process-local limitation.
- Connect packaged UI routes to current API/process authorities and observable
  success/failure states.
- Make unsafe or overbroad security claims fail deterministic local controls.
- Preserve concise compatibility entry points for existing top-level security,
  governance, backup, and troubleshooting pages.

**Non-Goals:**

- Fixing policy-load fallback, expanding tenant partitioning, changing API-key
  persistence, adding durable cost rollups, or changing shutdown behavior.
- Re-certifying runtime security, isolation, backup/restore, or operational
  resilience from documentation checks.
- Publishing raw `.prometheus`, KBD, private verification, credential, payload,
  or machine-local material.
- Running unit, integration, soak, production build, browser, accessibility,
  deployment, or public Pages gates in this content change.

## Decisions

### 1. Publish eleven guides in boundary order

Create these current-authority guides:

1. `website/docs/security/authentication.md`
2. `website/docs/security/credentials.md`
3. `website/docs/tenancy/overview.md`
4. `website/docs/governance/overview.md`
5. `website/docs/governance/approvals.md`
6. `website/docs/operations/runtime-console.md`
7. `website/docs/operations/runs.md`
8. `website/docs/operations/observability.md`
9. `website/docs/operations/realtime.md`
10. `website/docs/operations/cost.md`
11. `website/docs/operations/recovery-and-shutdown.md`

The sequence moves from who the caller is, through where identity is consumed,
to what policy and human decisions permit, then to how operators observe and
recover the process. The six inventory-backed routes remain unchanged.

Existing `website/docs/security.md`, `website/docs/governance/intro.md`,
`website/docs/backup-and-restore.md`, and `website/docs/troubleshooting.md`
become concise compatibility or index pages pointing to the new authorities;
they do not retain duplicate behavioral contracts.

**Alternative considered:** one security page and one operations page. Rejected
because profile, fail-closed, persistence, and UI/API boundaries would become
too easy to conflate and too hard to validate independently.

### 2. Add a separate operating-authority manifest

Add `docs/publication/security-operations.json` with exactly the eleven guide
IDs, files, routes, order, profiles, classified source records, current
authorities, required markers, and diagram/prose requirements. It supplements,
but never edits, `docs/publication/routes.json`: the route inventory proves
screen coverage; this manifest proves operating-contract depth and provenance.

Representative current authorities include `src/uar/security/`,
`src/uar/governance/`, `src/uar/runtime/manager.rs`,
`src/uar/telemetry/`, `src/uar/realtime/`, `src/uar/api/sse.rs`,
`src/server.rs`, matching frontend feature slices, and current canonical
OpenSpec for JWT hardening, tenant isolation, approvals, graceful shutdown, and
runtime event projection. Every path is resolved from the checkout before use.
`versions.toml` is absent and cannot be cited as inspected evidence.

**Alternative considered:** put source authorities in frontmatter only.
Rejected because a central ordered manifest is required to detect an omitted
guide or unclassified authority without trusting each document to inventory
itself.

### 3. Describe identity and isolation as a chain of proofs, not labels

The authentication guide explains the shared `TokenVerifier` boundary: HS256
when no JWKS URL is configured, RS256 when one is configured, exact issuer,
audience, `nbf`, `kid`, refresh, and failure behavior. RustCrypto is documented
as UAR's process-level `jsonwebtoken` provider; any earlier provider owner is a
structured conflict, not an identity comparison.

Tenant identity is documented only after successful verification. The tenancy
guide names the current A2A task/context partition and the separate user-scoped
credential, memory, and resource contracts. It explicitly says that the
presence of `tenant_id` does not prove blanket isolation for every table,
event, cache, or API.

**Alternative considered:** describe UAR as multi-tenant because several
subsystems carry tenant or user identifiers. Rejected because identifiers are
not equivalent to an end-to-end isolation proof.

### 4. Separate authorization, denial, and approval

The governance guide documents Cedar only for `server-full`, the permissive
capability-disabled facade elsewhere, empty-policy deny behavior, server policy
directory loading, and the current permit-all fallback after a load error.
It must not claim UAR is universally fail closed by default.

The approval guide treats three outcomes separately:

- Cedar/effective run policy denial: terminal for the tool call;
- allow: immediate execution;
- risk/Ask: emit approval-required and wait up to five minutes.

Rejection, timeout, channel closure, and cancellation do not execute the tool.
A human approval cannot override Cedar denial.

**Alternative considered:** describe every denied or risky tool as a human
approval. Rejected because it misstates the terminal denial event and creates
an unsafe expectation that an operator can override policy.

### 5. Use explicit state-ownership tables in all operations guides

Each operations guide identifies whether a signal is:

- process health/readiness;
- structured log or optional exported trace;
- Prometheus process metric;
- broadcast/replay run event;
- multiplexed entity live update;
- browser entity-graph/PGlite projection;
- durable database record;
- process-local cost/budget aggregate;
- external provider billing authority.

The realtime guide documents one shared `/api/live` EventSource, topic
demultiplexing, exponential reconnect capped at 30 seconds, and the fact that
reconnect is transport recovery rather than proof of durable event replay.

**Alternative considered:** call all operator-visible information telemetry.
Rejected because it erases the loss/reload/recovery behavior an operator needs.

### 6. Make recovery procedural but not self-certifying

The recovery guide consolidates persistence ownership, backup/restore, run
cancellation, signal handling, listener drain, registered cleanup, and deadline
outcomes. It links to exact provider-specific commands where current docs and
source support them, but requires cold boundaries and functional read-back.
Archive creation alone is not restore evidence.

The guide states that an in-process HTTP cancellation token stops listeners but
does not independently prove every runtime-owned resource exited; full shutdown
uses the signal path. Embedded hosts own their lifecycle, persistence, and
offline recovery contract.

**Alternative considered:** preserve the old backup page as the authority.
Rejected because backup commands without shutdown and functional restore
semantics are operationally incomplete.

### 7. Compose bounded fail-closed controls after content completion

Add `scripts/validate-documentation-security-operations.mjs` and
`scripts/test-documentation-security-operations.mjs`, expose root local
commands, and compose the validator into the publication entrypoint. Validate
the manifest, exact order, paths/routes, source classification, authority
existence, frontmatter, headings/markers, profile vocabulary, local links,
diagram prose, compatibility pages, and forbidden public material.

Observe isolated failures for at least:

- a missing guide;
- an unclassified or absent authority;
- an unsafe credential or private-key example;
- an unverified tenant string represented as trusted identity;
- a blanket tenant-isolation claim;
- universal fail-closed Cedar wording or omitted permissive fallback;
- approval described as overriding denial or missing timeout behavior;
- realtime represented as durable history;
- estimated cost represented as billing authority;
- missing graceful-deadline or restore-readback limits;
- a missing profile or state-owner limit;
- an unsafe private-history excerpt.

Execute controls only after all guides and compatibility pages are complete.
The final production build/browser gate remains in the phase's last change.

**Alternative considered:** rely on prose review and Docusaurus broken links.
Rejected because neither produces observed evidence that dangerous security
wording fails closed.

## Risks / Trade-offs

- **[Risk] Older specs or docs describe broader Cedar/approval behavior than
  current source.** → Use current implementation for present tense, cite
  canonical specs as intent, and state any disagreement rather than merging the
  claims.
- **[Risk] Security examples accidentally become usable secret patterns.** →
  Use placeholders only, reject private-key markers and credential-shaped
  material, and keep raw evidence outside public sources.
- **[Risk] Readers infer A2A partitioning protects every subsystem.** → Require
  a subsystem scope table and an observed negative control for blanket claims.
- **[Risk] Process-local metrics and projections are mistaken for audit logs.**
  → Name the owner and retention boundary on every guide and reject durable-live
  wording.
- **[Risk] Existing top-level pages keep contradicting the new guides.** → Make
  them short compatibility/index pages and validate their authority links.
- **[Trade-off] Marker validation cannot prove runtime security.** → Pair it
  with current-source audit and explicitly limit verification to documentation.

## Migration Plan

1. Add and parse the eleven-guide authority manifest; resolve every source and
   classified record from the checkout.
2. Add validator/control composition and local package commands without running
   the controls before content completion.
3. Write authentication, credentials, tenancy, governance, and approvals.
4. Write runtime console, runs, observability, realtime, cost, and recovery.
5. Convert legacy entry pages to concise indexes and add category metadata and
   cross-links without changing shared navigation or the route inventory.
6. Audit every present-tense security and operations statement against source,
   current OpenSpec, profiles, and state ownership.
7. Run the isolated controls, bounded TypeScript/source checks, strict OpenSpec,
   artifact-refiner content review, and permitted-surface audit; record row-form
   evidence.
8. Transition KBD, refresh the human handoff, and commit independently without
   pushing the partial phase.

Rollback is a revert of this documentation commit. No runtime state, public API,
database, dependency, policy, or deployed artifact changes in this unit.
