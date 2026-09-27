# Design

## Authority envelope

`PreparedToolInvocation` is the immutable command envelope. Its random `invocationId` is the execution identity. The resource and payload revisions cover the resolved descriptor, concrete mounted resource, and canonical validated arguments. The envelope also carries the complete UAR-owned execution lease (ID, task, attempt, epoch, holder, expiry, active state) and exact tool-call budget reservation (reservation ID, budget ID, revision, amount, unit, expiry, active state). Arguments remain available for dispatch but are redacted from Debug and never written into admission evidence. UAR's Cedar revision is an expected/observed revision for stale-wait detection; Flint Gate resolves its own authoritative policy state.

Lease and budget-reservation expirations are always finite. UAR uses the run's configured `budgets.timeout_seconds` deadline when present; otherwise it applies the existing 300-second tool-approval window as the explicit effect-authority TTL. Direct authenticated calls and the explicit constrained-local adapter use that same 300-second bound.

The paired host prepares and resolves admission as today. After local governance or human approval returns, UAR calls the host's claim-time revalidation operation with the complete immutable binding set. A changed host epoch, revoked grant, stale lease, changed payload, exhausted budget, or changed policy rejects the claim. UAR then re-evaluates current local Cedar policy and governance posture before persisting claim intent. Only after persistence succeeds may dispatch start.

## Ownership

- The Boss owns host authentication, approval presentation, durable issuer-scoped decisions, and host-side policy/grant facts.
- UAR owns descriptor resolution, argument validation, exact effect identity, claim serialization, dispatch, and terminal/uncertain state.
- Cedar and the effective UAR run policy compose restrictively. Human approval does not override a current Cedar deny or a stale authority envelope.
- Direct HTTP execution is an ordinary UAR invocation whose owner and Cedar principal are derived from the middleware-verified subject. Caller-controlled headers cannot select another agent identity. It is not a second execution path.
- Authenticated actor collaboration reuses the root run's exact approval gate. A required descriptor or host `ask` disposition remains pending until that gate returns its real decision; neither is converted into approval by the actor adapter.

## Governed and constrained-local profiles

A server with active governance must start only with a readable nonempty valid Cedar policy set. Missing or invalid policy is a startup error. Governance Off remains a distinct constrained-local posture established by the runtime authority: loopback ingress, no delegated actor, no paired host, and a root owner. The admission result records `governance_bypassed`, and runtime status/logging already exposes that posture.

Embedded callers must inject an explicit admission port and policy posture before exposing tool-capable execution. The convenience standalone runtime remains internal to constrained local roots; it cannot authorize delegated actors or paired-host effects.

## Compatibility

The sidecar contract remains `the-boss.uar.sidecar/1`. The private admission URL family remains `/uar/admission/v1`; claim-time revalidation is an additive `POST /uar/admission/v1/claim` operation and prepared-invocation fields are additive JSON fields. The Boss maps that private claim to Flint Gate protocol `afc.governed-effect/1`: initial evaluation uses `POST /authority/effects/evaluate`, and claim-time revalidation uses `POST /authority/effects/revalidate` with the issuer, challenge ID, and exact request. Gate decisions bind the request SHA-256 and authority references; UAR requires the paired-host authority revision to remain unchanged. Old paired hosts fail closed because UAR cannot complete claim revalidation.

## Failure semantics

Revalidation failure occurs before claim intent and therefore proves non-dispatch. Failure after claim persistence remains outcome-unknown until a terminal receipt or reconciliation. No retry reuses an invocation identity.
