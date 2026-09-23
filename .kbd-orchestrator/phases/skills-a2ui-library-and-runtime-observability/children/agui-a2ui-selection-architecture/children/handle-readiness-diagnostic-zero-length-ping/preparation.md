# Collector preparation — Task 1.1

The new collector derives connection, continuity, masking and frame-prefix code
from the accepted observer, and bounded UTF-8/JSON/RPC-ID parsing from the prior
restricted collector. Neither historical script is edited. The accepted observer,
frame observation and continuity digests were recomputed and match verification.md.
The new script verifies those three digests again before its sole connection.

Tier 0: `node --check collector.mjs` and scoped `git diff --check` exited 0
without output. No behavioral suite has run. Collection-time source digest:
`9c7e4385048ccf5ce64d62fb4e55823f11ca29740ef42292c575792fb8cafca2`.

## Source review of terminal paths

| Path | Cleanup and observation limit |
|---|---|
| Continuity/configuration failure | No socket opened; reserved receipt records skipped; local configuration byte buffers are cleared. |
| Sign-in request size, expired deadline, synchronous write error, callback error, close, timeout or success | sendSignin finally clears payload, mask, header and frame; writeFrame finally clears its frame; credential fields are released. |
| Pong write error, close, deadline or success | sendPong finally clears frame and key; writeFrame finally clears the frame. |
| Partial header/extended length | readFrameMetadata finally clears every acquired prefix buffer. |
| Unsupported/oversized frame | No payload read; runCollect finally destroys and drains queued socket buffers. |
| Invalid UTF-8, JSON, RPC ID or result; sign-in success/error | exchange finally clears raw bytes and releases parsed message reference. |
| SIGINT/SIGTERM | Bound handler marks interrupt and destroys current socket; normal finally clears credentials and queued buffers, writes sanitized receipt. |
| Receipt write failure | Socket cleanup precedes final receipt; FD closes in finally; exclusive reservation remains, preventing another attempt. |

JavaScript immutable strings and runtime-owned copies cannot be securely erased.
The collector releases references and emits no such values; it does not claim
cryptographic erasure of process memory. Mutable owned buffers have explicit
clearing paths. Behavioral cleanup verification remains Task 4.

The 15-second budget is verified at canonical spec lines 92–108. It now spans
the full exchange. readExact checks the deadline before reading even buffered
bytes, and exchange awaits sign-in write settlement before inspecting a frame.
The state transition consumes at most two frames and sends at most one Pong.

No separate --prepare run is needed: fresh passive continuity checks run inside
the exclusive --collect invocation before opening a socket. This prevents a
stale prepare receipt from authorizing collection. An unsafe gate consumes only
the local receipt and yields an explicit skipped attempt.
