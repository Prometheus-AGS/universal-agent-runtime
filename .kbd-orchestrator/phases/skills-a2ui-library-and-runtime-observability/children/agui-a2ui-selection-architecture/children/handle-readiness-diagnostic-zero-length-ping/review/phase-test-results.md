# Diagnostic phase-boundary checks

Command: `node --test collector.test.mjs` in this child (invoked by absolute path).
Observed exit code: 0.

```text
tests 36
pass 36
fail 0
cancelled 0
skipped 0
todo 0
duration_ms 2624.70525
```

Passing cases: direct Text; buffered Ping+Text and exact six-byte masked Pong;
sign-in error; repeated Ping; nonempty Ping; unexpected Pong; Close; continuation;
fragmented Text/Ping; Binary; reserved opcode; masked server frame; RSV-marked
frame; nonminimal 16/64-bit lengths; invalid 64-bit high bit; oversized/empty
Text; wrong RPC ID; missing/ambiguous result; malformed JSON; invalid UTF-8;
stalled Pong; Pong callback failure; buffered response after delayed sign-in;
stalled sign-in without frame reads; shared deadline across writes; expired read;
expired Pong write; synchronous write exception; SIGINT/SIGTERM cleanup; receipt
exclusivity/size; preserved real receipt digest, bounds, cardinality and key allowlist.

`node --check collector.mjs`, `node --check collector.test.mjs` and scoped
`git diff --check` exited 0. `openspec validate
handle-readiness-diagnostic-zero-length-ping --strict` returned:
`Change 'handle-readiness-diagnostic-zero-length-ping' is valid`.

These are synthetic stream fixtures. They do not prove remote Pong processing,
real sign-in success, precise host timer scheduling, immutable-string erasure,
or database-readiness recovery. Real evidence was preserved without a repeat.
