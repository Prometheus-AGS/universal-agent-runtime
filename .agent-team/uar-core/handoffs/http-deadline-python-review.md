# HTTP deadline correction review — A15repair15

Verdict: **PASS SOURCEONLY for the scoped HTTP deadline correction.** This is an implementation correction review, not formal CODErerun, whole-phase certification, runtime acceptance, or permission to execute a gate. Operator verification lock remains CLOSED.

Phase read from the authoritative waypoint: `uar-post-lifecycle-verification-readiness`, task 2.9, implementation stage IN_PROGRESS.

## Exact inspected artifact

`scripts/acceptance/core.py:246–258`:

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
```

`scripts/acceptance/core.py:268`: `chunk = reply.read1(min(65536, limit + 1 - len(prefix)))`.

`scripts/acceptance/core.py:277–281` cancels the owned interval timer in a finally block and restores the saved SIGALRM handler in its nested finally block.

## Source assessment

- Read `Context.request_bytes`, its `request` wrapper, `remaining`, `Evidence.redact`/`write`, and `public_body`. Read synchronous `user_outcomes.execute` and its exception/cleanup paths. Located request callers across acceptance cases. The actual runner calls each case synchronously and gives each Context the packet's `global_started` value (`user_outcomes.py:249–270`).
- The one-shot timer is armed before `opener.open`, remains armed during status/header and body collection, and is never renewed by body progress. Continuous heartbeat delivery and header drip therefore have an interrupt mechanism independent of inactivity timeout. Explicit monotonic checks before each read and after collection also prevent a successful late response.
- The saved prefix never exceeds `response_bytes + 1`. Each request to `read1` is at most 65536 bytes and no larger than the remaining prefix allowance. Limit overflow follows the existing response receipt and Failed predicate path.
- `HTTPError` remains a response (`core.py:261–263`); its status, headers and body are preserved. On deadline/transport errors the prefix byte count is added to `output_bytes`, while evidence retains only partial length/digest and redacted error text. `remaining()` after the transport receipt preserves global-wall Blocked versus request-only Failed classification (`core.py:288–296`). A Blocked raised during collection also retains partial length/digest and byte accounting.
- Unsupported platforms/threads, blocked SIGALRM, and existing active ITIMER_REAL timers refuse network entry (`core.py:237–245`). Existing active timers are left untouched. The supported path starts no worker thread, cancels its timer and restores the prior handler on return or exception. No concrete signal-state leak was found in this standalone serial runner scope.
- Installed primary stdlib source inspected directly, without importing/executing it: Python 3.9 `http/client.py:626–686` (`HTTPResponse.read1`, `_read1_chunked`), `urllib/request.py:1343–1372` (request/header error closure), and `urllib/error.py` plus `urllib/response.py` (HTTPError response wrapper). The chunked read1 implementation may perform protocol reads before a payload read; the timer encloses those reads too. These installed files explain the API shape but do not establish the frozen fixture interpreter identity.

No actionable CRITICAL/HIGH correction finding in the assigned scope. No source code was modified. Only this assigned handoff was written; no other author's edits were reverted. No author handoffs or generation history were used.

## Limits and command record

All commands were source inspection (`git diff -- '*.py'`, `cat`, `sed`, `nl`, `rg`, `head`) or this authorized handoff write. No tests, lint, formatting, typecheck, import, compiler probe, build, service/provider command, or helper execution ran. Static analysis was intentionally not run because it is verification under the CLOSED lock and the assignment explicitly forbids it. One source search returned status 2 because alternate stdlib paths were absent; the installed Python 3.9 sources were subsequently read successfully.

Unverified: actual timer delivery latency, frozen macOS/Linux interpreter behavior, runtime resource cleanup, controlled heartbeat/header-drip reproducer, and device/customer outcomes. Signal-handler setup/teardown scheduling edges and arbitrary embedding environments are not certified by source inspection. The supported scope is the selected standalone synchronous main-thread fixture host; this verdict does not broaden it.
