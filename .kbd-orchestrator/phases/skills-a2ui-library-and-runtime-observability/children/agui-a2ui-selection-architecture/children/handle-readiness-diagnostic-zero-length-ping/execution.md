# EXECUTION: handle-readiness-diagnostic-zero-length-ping

Stage: Execute. Backend: OpenSpec through KBD apply, self-executed by Codex.
Model class: frontier; current GPT-6 session. Source: plan.md and plan handoff.
The operator's subsequent `continue` authorizes the planned bounded attempt.
Four tasks run sequentially; KBD owns task boundaries and status follows each.

Files: collector.mjs, collector.test.mjs, preparation.md, evidence/*.json,
classification.md, verification.md, handoff-out.md and child review receipts;
OpenSpec tasks and generated KBD state record completion.

The collector combines the prior collector's bounded JSON/ID parsing with the
accepted observer's connection, continuity and framing helpers. Prior artifacts
remain immutable. Source review enumerates cleanup paths before observation;
all behavioral fixtures run at Task 4, the child phase boundary.

The two Plan review corrections are carried forward: the 15-second budget comes
from the canonical header-observation requirement and is strengthened to cover
the full exchange; Task 1 checks cleanup structure, Task 4 checks behavior.

Sign-in is the final operation. Any prerequisite failure records a skipped
attempt. A consumed attempt cannot be repeated even after offline corrections.

## Progress

Task 1 started. No collector observation yet. Runtime behavior remains unverified.

## Task 2.1 observation

Command: `node collector.mjs --collect` (current child directory).
Exit 2: `state=stopped`, `stopReason=unsupported_websocket_frame`.
One upgrade and one sign-in request; one Pong write completed; two frame headers
observed; zero sign-in responses, payload bytes read/retained, queries, readiness
requests or retries. Both headers are final, RSV-clear, unmasked, minimally
encoded zero-length Ping. The second Ping is the explicitly unsupported boundary.
Active elapsed: 15,853.493 ms; exchange elapsed: 6,845.883 ms.
Continuity comparisons passed. HTTP upgrade 101, json subprotocol, no extensions.
Receipt size: 3,906 bytes; SHA-256:
`969d730640fb8c8e1ddede26bdc3b1236b22b3719d9e138fa67ec80011a515b8`.
The exclusive receipt remains reserved; the collector was invoked exactly once.

## Task 4.1 acceptance

Diagnostic suite passed 39/39 at the child phase boundary. Strict OpenSpec and
Tier 0 syntax/scoped diff checks passed. The artifact critic's initial three
findings were corrected offline and cleared; the exact collection source was
preserved before those changes. REST diff review passed with zero critical,
two warnings and three suggestions, all retained in the handoff. Strict
anti-theater screen passed (score 0.0). See verification.md for actual commands,
outputs, hashes and limitations. Exact runtime model identity is unverified;
the initial model label above should not be treated as verified provenance.
