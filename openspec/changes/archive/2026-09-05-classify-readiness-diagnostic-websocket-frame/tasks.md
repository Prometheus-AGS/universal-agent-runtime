## 1. Continuity and observer preparation

- [x] 1.1 Reconfirm the inherited source, executable, process/start-time, listener, nonsecret target, authentication-form, configuration-digest, and evidence-hash baseline; create the dependency-free child-local header observer with the 10-byte framing-prefix, 64-KiB evidence, privacy, deadline, and stop constraints from the design; verify the observer parses syntactically and the continuity receipt contains only allowlisted fields, or record the exact prerequisite failure and mark the operational task skipped.

## 2. Single authorized observation

- [x] 2.1 After explicit Execute authorization and only if Task 1.1 passes, run the observer once: make one WebSocket upgrade and one root-signin RPC, capture only derived handshake/header metadata, destroy the socket without intentionally reading or parsing payload, and verify `evidence/frame-observation.json` reports the exact one-or-zero attempt and zero query/readiness/payload-retention/credential-retention counters; stop without retry on any failure.

## 3. Offline classification

- [x] 3.1 Produce `classification.md` that maps every observed field separately to RFC 6455 header-level constraints and the prior collector predicate, identifies indeterminate properties and historical-frame uncertainty, and gives only the smallest evidence-justified next action; verify each claim points to a retained field or cited protocol rule and does not implement a fix or resume readiness.

## 4. Phase-end acceptance

- [x] 4.1 At the child phase boundary, run synthetic observer/parser fixtures, JSON schema/size/privacy assertions, strict OpenSpec validation, and a fresh-context artifact-only critic over the spec, observer digest, evidence, and classification; resolve critical findings or retain a qualified/blocked outcome, deliver `verification.md` and `handoff-out.md`, verify no product/source/service/database mutation or product test suite occurred, and run `kbd-status` before entering Reflect separately.

Task order is strictly 1.1 → 2.1 → 3.1 → 4.1. A failed prerequisite yields explicit SKIPPED records for dependent operational work; it never permits a retry, fallback client, enlarged budget, or claim about the discarded historical frame. The phase-end synthetic checks are diagnostic-artifact tests, not product tests or GitHub Actions work.
