# Whole-phase CODE diagnostic review

Verdict: FAIL SOURCEONLY. One WARNING in the acceptance runner. This is a bounded source review, not runtime certification or a claim that the affected scenario has occurred.

## Identity and isolation

- Review target: supplied `current-source-after-RUST-correction.diff`, listed untracked sources, and current inspected source; declared identity `sha256:e79f323b0f9de281afc2e13dc78406fe729e96972419c4398ca08698b66fa2f2`. No identity probe was executed.
- Phase: `uar-post-lifecycle-verification-readiness`; waypoint revision 518, task 2.9, stage IN_PROGRESS, implementation 144/168. This review does not reconcile or override that canonical status.
- Native fresh-context artifact-only reviewer; same-family fallback, weaker than cross-model adversarial review. Exact producer and judge model IDs are unknown to this reviewer. No full API adversarial-review claim. Producer conversation, implementation handoffs and other reviewer verdicts were not used.
- Authorization: A15 CODE source inspection/report writing only. No tests, builds, compiler probes, imports, lint, formatting, diff checks, providers, services, or external gates were executed. No production source was edited.

## WARNING — [scripts/acceptance/core.py:241] HTTP body collection does not enforce the absolute packet deadline

Freshly inspected artifact:

```python
# scripts/acceptance/core.py:234-242
try:
    reply = self.opener.open(request, timeout=min(
        self.remaining(), self.fixture["limits"]["request_seconds"]))
except urllib.error.HTTPError as error:
    reply = error
with reply:
    limit = self.fixture["limits"]["response_bytes"]
    body = reply.read(limit + 1)
    response = Response(reply.code, {k.lower(): v for k, v in reply.headers.items()}, body)
```

The body-read line is exact; indentation above is normalized to the quoted block.

Failure scenario: an owned HTTP endpoint opens a chunked SSE response and keeps delivering small heartbeat chunks before each socket timeout, without closing or reaching the byte allowance. The synchronous `read(limit + 1)` can remain inside the body read after the invocation's absolute wall deadline. `remaining()` is called before opening the request, not during body collection. Cleanup and first-failure evidence cannot run until control returns. The byte limit bounds bytes rather than elapsed time.

Surrounding scope inspected: full `Context.request_bytes`, `Context.remaining`, `user_outcomes.execute`, `MountedClient.native`/`chat`, and `PacketProcesses.start`/`alive`/`stop`. Mounted streaming paths call this same request reader. Packet liveness is checked between cases; it is not a concurrent interruption of this read. The forwarding process has its own deadline, but the HTTP reader also serves owned runtime endpoints and cannot depend on every endpoint terminating when that forwarding process expires. The local Python 3.10 stdlib `http/client.py` was inspected: `HTTPResponse.read(amt)` delegates chunked bodies to `_read_chunked`, whose loop collects chunks until the requested amount or stream termination without an absolute deadline. That installed source is explanatory evidence, not proof of the frozen fixture interpreter version.

Minimum correction: enforce one absolute deadline across request opening and body collection, with bounded incremental reads and an interruption mechanism that can terminate a stalled read at that deadline; preserve partial redacted failure evidence and existing byte limits. Recomputing `remaining()` only after the same blocking aggregate read would not correct it.

Falsifier / future authorized reproducer: a controlled owned chunked endpoint drips a short valid SSE heartbeat more frequently than `request_seconds`, stays below `response_bytes`, and continues past `wall_seconds`. Demonstrate that the actual frozen runner interrupts, retains failure evidence and performs owned cleanup within its declared wall/cleanup allowances. This reproducer was not run. A proven whole-run watchdog that interrupts this request would falsify the finding; none appears in the inspected runner or packet owner.

This violates the product-validation-evidence requirement to stop on exhausted allowance. It is a concrete acceptance-control defect, not a claim of incorrect inference behavior.

## Coverage and limits

Inspected the acceptance core/runner, mounted protocol adapters, interface and browser cases, installed browser client, Python/TypeScript acceptance examples, TypeScript client changes, AG-UI adapter/projection changes, selection announcement, chat hydration/stream continuation changes, A2UI rendering/action domain/transport/hooks/receipt persistence, and graph bootstrap. Consulted the product-validation-evidence OpenSpec delta, project rules, team tool policy/repository map, uar-project-review, review-verification-protocol, Vercel React Best Practices/Composition Patterns and entity-graph optimization guidance. Entity review applied typed domain-hook access from components, domain-owned sequencing, transport-owned I/O, narrow status subscriptions, and durable action receipts. No additional confident code defect was established in those inspected paths.

The cumulative diff is much broader than this six-minute review. Rust runtime/persistence, vendor code, every Python observer/case, all transport/auth paths and every OpenSpec delta were not exhaustively reviewed. No absence-of-security-defects claim is made. Python SDK DONE handling was traced into its existing SDK parser, which consumes the marker internally; the example's unreachable marker branch alone was not treated as a product defect.

Actual receipts read: S5 attempt/final; S6/S7 attempt 1; S8/S9 attempts 1/2; S10 attempt/final; S14-S21 attempt 1. S6/S7 passed at `28137dc...`; S8 passed after correction at `c6963d...`; S9 still failed after correction; S5 remained blocked after one correction. S10 final records timeout, no produced image, and acknowledged build cancellation. These identities precede the reviewed declared `e79f323...` source. S14-S21 validate spec artifacts at the earlier identity; they do not establish compiled behavior. The latest auxiliary receipt correction remains uncompiled. Runtime cases, main boundary and Issue 426 remain blocked by fixture availability according to the supplied scope; no runtime PASS or feature failure is inferred from their absence.

No verification command was run and no tier acceptance was earned. Only this report was added; no guard or unrequested implementation was added. Proposed correction remains unimplemented and unverified. Coordinator-owned sycophancy detection screens remain pending; this report does not claim machine screening.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | none established |
| WARNING | 1 | source defect; correction pending |
| SUGGESTION | 0 | none |

Verdict: FAIL SOURCEONLY — resolve or falsify the acceptance deadline finding. Runtime acceptance remains BLOCKED and unverified independently of this source verdict.
