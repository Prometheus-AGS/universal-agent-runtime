# repair15 — HTTP absolute deadline correction

Phase: `uar-post-lifecycle-verification-readiness`; waypoint inspected at revision 518,
task 2.9, implementation 144/168, stage IN_PROGRESS. Verification remains CLOSED.
This handoff does not override the canonical phase state or authorize a gate.

## Delivered source

- `scripts/acceptance/core.py`: added the standard-library `threading` import and
  changed only `Context.request_bytes`. One scoped POSIX `ITIMER_REAL`/`SIGALRM`
  timer interrupts request opening, headers, and body collection using the minimum
  of the existing context remainder and request allowance. It is never rearmed
  per chunk. The previous signal handler is restored in nested `finally` cleanup
  and the owned timer is disarmed on every exit from the armed scope.
- Body collection uses `read1` in chunks of at most 65,536 bytes, with the original
  response limit plus one overflow byte as its total capture cap. Each iteration
  checks the context and absolute request deadline. HTTP error responses remain
  readable responses through the existing `HTTPError` branch and context manager.
- Failed transport receipts now include elapsed time, captured prefix byte count,
  and its SHA-256 digest. Partial body content is withheld, including potentially
  incomplete private reasoning or credentials; the receipt still passes through
  `Evidence.write` redaction. Captured bytes count toward existing output usage.
- The transport exception path calls `remaining()` after retaining evidence, so
  exhausted global wall allowance propagates `Blocked`; request-only expiry keeps
  the existing chained `Failed` transport semantics. In-read `Blocked` also retains
  the prefix identity and propagates unchanged.
- `.agent-team/uar-core/handoffs/http-deadline-correction.md`: this scoped handoff.
  No fixtures, private inputs, manifests, limits, callers, or other implementation
  files were changed by this writer. No resource allowance was added.

## Source basis and limits

The observed CODE warning is the owned chunked SSE heartbeat scenario in
`target/uar-diagnostic-sweep-001/reviews/whole-phase-code-review.md`: the old
aggregate `read(limit + 1)` could remain blocked while heartbeats avoided socket
inactivity expiry. Incremental reads retain prefixes; the signal interruption
covers a blocked chunk header, response header, or opening operation as well.

Inspected `user_outcomes.py`'s `__main__ -> main -> execute` path: contexts and case
requests run synchronously in the serial root process. `PacketProcesses` creates
its context synchronously and owns the budget forwarding subprocess; the
`budget_proxy.py` server and timer threads are inside that separate subprocess.
No request worker or background thread was introduced.

Inspected installed Python 3.10 stdlib source without importing or executing it:
`http/client.py:643` (`read1`) and `:684` (`_read1_chunked`),
`urllib/request.py:1307` (`do_open`, including close-on-exception),
`urllib/error.py:35`, `urllib/response.py:18`, and `tempfile.py:473`
(HTTPError response delegation and close), `signal.py` (signal wrappers and
pthread mask), `test/test_signal.py:698` (read only: getitimer/setitimer call
signatures and tuple shape), and `threading.py:1423,1567` (thread identity APIs).
These are source evidence for APIs, not proof of the frozen fixture interpreter.
Python patterns were consulted for scoped cleanup, explicit exception handling,
and retained exception chaining. Root instructions, waypoint, and gotchas were
read; the absent `.claude/rules/python.md` was not invented or replaced.

Timer authority is process-global. The implementation explicitly blocks before
network I/O on unsupported platforms/APIs, non-main-thread execution, an inherited
blocked SIGALRM, or an existing active interval timer (either tuple component).
The active timer is inspected without replacing it. These guards trace to the
real timer-ownership boundary; no unbounded fallback or background worker exists.
This is intentionally a Mac/Linux serial-main-thread path, not a portable
Windows transport redesign. Independent concurrent code installing an alarm in
the same process is outside this serial ownership contract.

## Verification and remaining work

Only source inspection and authorized implementation/documentation writing ran.
No imports, Python execution, tests, format/lint/type checks, compiler probes,
builds, diff checks, service/provider calls, inventory scripts, or review commands
were run. No verification tier acceptance is claimed. Runtime interruption,
actual frozen-interpreter compatibility, timer restoration, heartbeat cleanup,
and error-response behavior remain unverified under CLOSED.

No unresolved design choice blocks the scoped implementation. The lead owns the
renewed exact-source whole-phase inventory and the single fresh CODE review;
this writer did not launch another review/cohort or alter their authority.
Any runtime reproducer requires its own exact operator authorization.
