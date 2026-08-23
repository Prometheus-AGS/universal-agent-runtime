## 1. Establish the security and operations authority contract

- [x] 1.1 Add `docs/publication/security-operations.json` with exactly the eleven planned guides, fixed order, stable routes, profiles, classified records, existing authorities, required boundary markers, and diagram requirements; verify the manifest parses and every referenced source path exists.
- [x] 1.2 Add the bounded security/operations validator and isolated control script, register their local package commands, and compose the validator into the final publication entrypoint; syntax-check the scripts but defer executing controls until all guides and compatibility pages are complete.

## 2. Document authentication and credentials

- [x] 2.1 Add `security/authentication` covering JWT-required and anonymous modes, HS256 and RS256/JWKS selection, registered-claim checks, cache/rotation behavior, API keys, probe exceptions, RustCrypto provider ownership, observable failures, and profile limits; verify no unverified claim is described as identity.
- [x] 2.2 Add `security/credentials` covering the packaged credentials UI/API, write-only plaintext, masked metadata, AES-256-GCM storage, session/agent/user/system/env resolution, service-disabled fallback, deletion/rotation, persistence limits, and profile boundaries; verify it contains no usable secret or private-key material.

## 3. Document tenancy, governance, and approvals

- [x] 3.1 Add `tenancy/overview` with the verified-tenant construction boundary, current A2A task/context partition, separate user/session/agent/deployment scopes, cross-tenant denial behavior, and explicit non-universal limits; verify blanket isolation language is absent.
- [x] 3.2 Add `governance/overview` with server-full Cedar behavior, minimal/embedded exclusions, policy directory and reload behavior, empty-policy denial, current permissive load-error fallback, HTTP `X-Agent-Id` scope, tool/skill decision boundaries, and audit limits; verify it does not claim universal fail-closed enforcement.
- [x] 3.3 Add `governance/approvals` with effective-run deny, Cedar deny, allow, risk/Ask, emitted events, packaged dialog and approvals page, single-use response, cancellation, rejection, channel-close, and five-minute timeout semantics; verify approval is never described as overriding denial.

## 4. Document live runtime operations

- [x] 4.1 Add `operations/runtime-console` and `operations/runs` covering packaged UI/API entry points, live entity sources, trace and status inspection, cancellation, terminal outcomes, approval interaction, reload behavior, and process/browser-state limits; verify UI projection is not described as durable runtime authority.
- [x] 4.2 Add `operations/observability` covering liveness/readiness, Prometheus metrics, structured log formats and filters, optional OTLP export, provider/model versus UAR-owned signals, scrape/export failures, and profile limits; verify logs and metrics are not described as an immutable audit ledger.
- [x] 4.3 Add `operations/realtime` covering run streams, AG-UI replay boundaries, the multiplexed `/api/live` EventSource, topic demultiplexing, reconnect/backoff, live-query ownership, offline/embedded behavior, and reload/replay limits; verify reconnect is not described as durable replay.

## 5. Document cost, shutdown, and recovery

- [x] 5.1 Add `operations/cost` covering opt-in usage/cost estimation, provider/model labels, in-process budget scopes and threshold events, the session-scoped dashboard, missing pricing/usage behavior, and provider-billing authority; verify estimates are not represented as invoices or durable spend history.
- [x] 5.2 Add `operations/recovery-and-shutdown` covering SIGINT/SIGTERM, root run cancellation, listener drain, registered cleanup, configured deadline and outcome markers, persistence-provider ownership, cold backup/restore, functional read-back, and embedded-host responsibility; verify HTTP cancellation alone is not described as full process shutdown.
- [x] 5.3 Add category metadata, dependency-order cross-links, and concise compatibility/index pages for `security.md`, `governance/intro`, `backup-and-restore`, and `troubleshooting`; verify all local links resolve while shared global navigation and `docs/publication/routes.json` remain unchanged.

## 6. Reconcile truth and publication safety

- [x] 6.1 Audit every present-tense security and operations statement against current source and canonical OpenSpec, qualify conflicts with older narrative material, and verify profile, permissive fallback, state-owner, retention, and durability limits are explicit while `versions.toml` is not cited as inspected authority.
- [x] 6.2 Audit all public content for raw `.prometheus` or KBD material, machine-local paths, credentials, private keys, raw event/session payloads, unsafe command examples, and unsupported certification claims; verify public pages contain only reviewed current-source synthesis.

## 7. Verify only after security and operations content is complete

- [x] 7.1 Run isolated security/operations controls and observe missing guide, unclassified authority, unsafe credential, unverified tenant identity, blanket isolation, universal fail-closed governance, approval override/timeout, durable realtime, authoritative billing, missing recovery/deadline, missing profile/state-owner, and unsafe private-history controls fail before the complete source fixture passes.
- [x] 7.2 Run the Docusaurus TypeScript check and bounded architecture, brand, product-workflow, security/operations, and composed publication control suites locally, recording commands and output without running the phase-level production build or browser/accessibility certification.
- [x] 7.3 Run `openspec validate document-security-tenancy-governance-and-operations --strict` and the artifact-refiner content gate; correct findings until the OpenSpec bundle, refiner schemas, referenced evidence, and finalized active/history state pass.
- [x] 7.4 Audit tracked and untracked diffs for runtime, React application, provider/security behavior, dependency, vendored, shared route/navigation, README, non-documentation lockfile, raw `.prometheus`, and deployment-workflow changes; remove anything outside this change's documentation/process surface.
- [x] 7.5 Write row-form `verification.md` with requirement, command, observed result, limit, source SHA, and documentation profile, explicitly deferring build, visual, accessibility, deployment, runtime-security, backup-restore, and cross-profile claims.
- [x] 7.6 Transition the registered KBD change through the canonical runtime, refresh the human handoff, verify the exact next command names `document-apis-sdks-tools-and-deployment`, and commit the complete change independently without pushing the partial phase.
