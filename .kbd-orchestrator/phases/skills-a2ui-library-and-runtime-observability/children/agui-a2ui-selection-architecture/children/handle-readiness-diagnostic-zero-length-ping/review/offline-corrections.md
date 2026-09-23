# Offline acceptance corrections

The independent artifact-only critic identified two P2 verification gaps and
one P3 cleanup discrepancy: the receipt test called a writer not used by
collection; live incomplete reads and stalled Text after Pong were not tested;
partial handshake bytes lacked finally-based clearing.

Corrections are confined to this child's collector and synthetic fixtures.
The actual descriptor writer now has size, replacement, truncation and exclusive
reservation coverage. Three read-wait fixtures cover the original deadline,
terminal cleanup and counters. Partial handshake bytes are cleared in finally.

Collection was not repeated. The observation remains immutable. The exact
collection-time source is preserved as `collector-at-observation.mjs`, SHA-256
`9c7e4385048ccf5ce64d62fb4e55823f11ca29740ef42292c575792fb8cafca2`.
The corrected offline collector SHA-256 is
`792a659aff4dd3b10a95b4b358300c21e0e30e43d7d1a9fae58fa01b21e7b3d5`.
The receipt SHA-256 remains
`969d730640fb8c8e1ddede26bdc3b1236b22b3719d9e138fa67ec80011a515b8`.

Observed phase-boundary command: `node --test collector.test.mjs`.
Output: tests 39; pass 39; fail 0; cancelled 0; skipped 0; todo 0;
duration_ms 1647.739167. Exit 0.
Both `node --check collector.mjs` and `node --check collector.test.mjs`
exited 0 with no output. Product tests and another collection were not run.

Uncomfortable limitation: these offline corrections were not present in the
sole live attempt. That attempt ended on a second empty Ping before any Text
response; it proves neither authentication completion nor readiness recovery.
