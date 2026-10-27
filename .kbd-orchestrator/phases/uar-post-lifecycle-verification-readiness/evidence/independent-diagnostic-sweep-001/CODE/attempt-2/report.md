# CODE corrective source review — attempt 2

## Artifact under judgment

Freshly read `scripts/acceptance/core.py:246-268`:

```python
allowance = min(self.remaining(), self.fixture["limits"]["request_seconds"])
deadline = time.monotonic() + allowance
prefix = bytearray()
limit = self.fixture["limits"]["response_bytes"]

def interrupt_request(signum, frame):
    raise TimeoutError("absolute HTTP deadline exhausted")

previous_handler = signal.signal(signal.SIGALRM, interrupt_request)
try:
    try:
        # Never rearm per chunk: every blocking stage shares this wall.
        signal.setitimer(signal.ITIMER_REAL, allowance)
        try:
            reply = self.opener.open(request, timeout=allowance)
        except urllib.error.HTTPError as error:
            reply = error
        with reply:
            while len(prefix) <= limit:
                self.remaining()
                if time.monotonic() >= deadline:
                    raise TimeoutError("absolute HTTP deadline exhausted")
                chunk = reply.read1(min(65536, limit + 1 - len(prefix)))
```

Indentation is normalized; identifiers and statements above are from the inspected source. This is the replacement for the prior aggregate `reply.read(limit + 1)` finding.

Verdict: **PASS SOURCEONLY**, limited to correction of the original HTTP absolute-deadline finding and its direct propagation paths. Zero actionable correction defects established. This is not whole-phase approval, runtime acceptance, or release certification.

## Identity and isolation

Supplied source identity: `sha256:5c78708d57b1949d96002628a001ca6e52a299dc8530f3a9c75ea1d1681ad5eb`, from `target/uar-diagnostic-sweep-001/reviews/source-review-inputs-after-CODE-correction.json`. No source-identity calculation or verification command was executed. Actual current source was read directly. The requested staged and unstaged Git diffs for the two scoped Python files returned no hunks; the input lists them as untracked. The supplied cumulative diff search did not provide this correction, so the verdict relies on actual source and the independent original finding, not a claimed diff identity verification.

Native fresh-context artifact-only reviewer; same-family fallback, weaker than cross-model review. Exact producer and judge model IDs are unknown to this reviewer. No author conversation, generation history, implementation handoffs, or preferred verdict was consumed. The original independent finding was an explicit re-review input. This is the single corrective review, not a renewed whole-phase review.

The authoritative waypoint read in this turn identifies `uar-post-lifecycle-verification-readiness`, revision 518, task 2.9, implementation 144/168, stage IN_PROGRESS. The operator verification lock remains CLOSED. All commands were source/artifact inspection or writing this report. No test, import, compiler probe, build, lint, formatting, diff check, helper, service, provider, or external gate ran.

## Original finding disposition

**Resolved at source level.** The original trigger was a small heartbeat/drip HTTP response keeping an aggregate body read alive beyond the absolute allowance. The correction arms a single `ITIMER_REAL` before `opener.open` and leaves it armed through headers and incremental body collection (`core.py:254-276`). It does not refresh the timer when a chunk arrives. The alarm handler raises `TimeoutError`; the loop also checks the monotonic deadline and global allowance. This adds the interruption mechanism missing from the earlier reader rather than relying solely on socket inactivity.

The correction explicitly refuses unsupported platforms or non-main-thread execution (`core.py:237-241`), a blocked SIGALRM (`242-243`), and an existing active interval timer (`244-245`). The inspected standalone runner calls `main()` directly (`user_outcomes.py`, final entrypoint) and executes backend/case work serially (`217-320`); the correction introduces no worker thread. This disposition is scoped to the supplied Mac/Linux standalone main-thread runner configuration.

The timer is disabled and the saved signal handler restored in nested `finally` blocks (`core.py:277-281`):

```python
finally:
    try:
        signal.setitimer(signal.ITIMER_REAL, 0)
    finally:
        signal.signal(signal.SIGALRM, previous_handler)
```

HTTPError continues through the bounded reader as a response (`260-263`). Response collection preserves the limit-plus-one overflow detection and bounded chunk size (`264-271`, `301`). On interrupted transport or global exhaustion, collected prefix bytes are charged to `output_bytes`; receipts retain byte count, digest, elapsed time and redacted error information (`282-296`). No opaque partial body is copied into the failure receipt. A global allowance failure remains Blocked, including after an alarm wrapped by urllib; a request-only transport failure remains Failed (`293-296`).

`user_outcomes.execute` catches Blocked/Failed, records the case outcome and first failure, stops the serial case loop, and enters owned cleanup (`277-318`). `Context.cleanup` has its separately bounded cleanup allowance (`core.py:494` onward). Those source paths address the original inability to return control for failure evidence and cleanup; they do not prove that cleanup succeeds on a real frozen runner.

## Coverage and evidence limits

Read in full: `Context.request_bytes`, `Context.remaining`, `Context.cleanup`, `Evidence` redaction/write/seal, and `user_outcomes.execute`; also inspected fixture limit admission and the standalone entrypoint. Consulted the product-validation-evidence exhausted-allowance requirement, team tool policy/repository map, uar-project-review, and review-verification-protocol. Rust edition/API checks are inapplicable to this Python-only corrective scope. No unused-code claim is made.

The uncomfortable limitation is that signal-driven interruption and cleanup have not been exercised here. Latest code remains uncompiled/unexecuted under the supplied evidence scope. Actual frozen-interpreter behavior, heartbeat interruption timing, open/header interruption, retained receipts and cleanup completion remain unverified. A future separately authorized real-run reproducer should exercise the original drip trigger and both request-only/global exhaustion; it was not run in this review. No claim is made about uninspected broader source or absence of all security defects. Runtime 97 cases, main acceptance, and Issue 426 remain BLOCKED according to the supplied scope; this source verdict does not change them.

Only this report was written. No production changes, guards, or unrequested implementation were added. Existing new guards trace to the original deadline failure and ownership of the process-global timer. Coordinator-owned sycophancy screens are pending; this report does not claim machine screening or cross-model certification.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | none established |
| HIGH | 0 | none established |
| MEDIUM | 0 | none established |
| LOW | 0 | none established |

Verdict: **PASS SOURCEONLY — APPROVE the bounded correction at source level.** Original deadline finding resolved in inspected code. Runtime acceptance and whole-phase completion remain unverified and BLOCKED independently of this verdict.
