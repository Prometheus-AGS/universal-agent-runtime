# Analysis: independent workflows, shared truth

## Governing question
Can a configured user complete each designed workflow reliably on the claimed platform? A successful HTTP status, build or source review cannot answer this alone. Three successful repetitions establish observed repeatability on those runs, not a statistical availability guarantee.

## Chosen architecture
One versioned coverage registry is the source of scenario definitions, prerequisites, applicability, repetitions and limits. One small runner selects cells by workflow/platform/backend and calls production entry points. A cell is a scenario on one actual platform, backend and source/artifact identity. Shared setup is cached only where identities and owned state permit reuse. Each cell acquires only its named prerequisites. Setup errors become BLOCKED dependent cells, not global suite aborts.

Separate prepare, exercise, cleanup and report as internal steps of the same admitted invocation. A basic prompt needs one usable artifact, one isolated supported store, configured real inference, ingress credentials and its own evidence directory. It does not need embeddings, another platform, every SDK or every backend. A2UI, team, KB and platform-specific fixtures are lazy additions.

Retain protected-host boundaries, genuine inference provenance, current source/artifact/config identity, durable state, provider reservations and redaction. Do not infer zero cost from subscriptions or refund an unknown embedding request. Reuse a minimal existing spend-reservation mechanism rather than invent another generic proxy, signing protocol, database or receipt framework. Detailed terminal-receipt tests are supporting checks; a cancelled or unsettled request retains its reservation. If resource ownership, spend safety or contamination is unknown, block the affected resource domain; stop all provider dispatch if the shared budget itself is untrustworthy.

One append-only run log contains scenario/platform/backend/source/artifact/config identities, prerequisites, attempt, input/expected predicate, redacted observed response/stream references, duration, usage/reservation, result/reason, cleanup and any correction link. Terminal PASS requires every oracle and cleanup; PASS of one cell cannot fill another cell. Mixed-source summaries report each cohort separately.

## Platform sequencing

The operator has selected complete platform sweeps in the order macOS, Linux, iOS. Workflow independence applies within the current platform and to honest dependency blocking, not permission to interleave later-platform cases. Finish all applicable current-platform dispositions and cleanup before advancing; prioritize shared fixes so later platforms exercise corrected artifacts. A completed sweep with failures is not certification. Windows remains visible outside this selected sweep; Android deferred. Later shared fixes require affected earlier-platform regression evidence or an explicit unverified designation.

## Failure policy
A product failure is FAIL; missing executor/credential/fixture is BLOCKED; unattempted is NOT_RUN; Android is DEFERRED; NOT_APPLICABLE needs a capability/spec reason. Continue independent cells. Keep original failures even if a correction passes. At most one targeted correction and one same-item rerun; choose quick fix or backlog after seeing the failure. A quick fix is at most15active minutes and within already approved scope. It excludes dependency upgrades, architecture changes and weakened assertions. No background wait for agents to supply correction receipts. A source change rebuilds affected artifacts and records the new cohort; it never relabels old passes as new-source acceptance.

## Policy redesign is explicit
The current AGENTS.md and adopted A22 still apply. They are not silently bypassed by activating this child. The proposed replacement has one preparation/implementation boundary and one explicit execution approval with named side effects, resources, supported selectors and bounded one-fix/one-rerun authority. The operator must explicitly adopt the child-specific gate policy before it replaces per-edit inventory/signature loops. Keep canonical KBD signed-contract compatibility; derive any required envelope/manifest mechanically from the single registry/run plan instead of manually duplicating it. No credentials or platform facts are invented.

## Limits and inheritance
All parent call/time/repair history and UNKNOWNs carry forward. Account032 recorded3909calls/2232.207652466667conservative active minutes at2026-10-26T05:52:09.628595Z; later work accrues. Original ceilings4000calls/2400active minutes/28repairs remain; repair20 consumed, fixture21 uncharged. No new idle exclusion or child reset. Independent total embedding ceilings remain USD10Cohere/USD10OpenAI, with prior reservations deducted. New execution volume must fit the actual remainder or obtain an explicit cumulative-limit amendment; planning is not that amendment.

## Rejected alternatives
A giant new harness repeats the defect. A thin wrapper around platform_suite still requires all platforms/stores. Mock-only smoke tests cannot establish configured inference. Dropping scenarios to reduce the denominator obscures missing behavior. Removing ownership/spend checks would replace preparation failure with uncontrolled side effects.
