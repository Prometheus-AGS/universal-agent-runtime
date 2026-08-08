# C-15 protected-path closeout

Run date: 2026-08-08

The exact combined protected-status, unstaged-diff, and staged-diff stream recomputed to:

```text
07e74ad94dc137e9574e411bc99d6f0fcd631879c5a0e52a1b87ca999cf43dc4
```

This matches `protected-path-baseline.txt`. C-15 did not modify `.gitmodules`, the
Prometheus skill-system submodule, either operator-owned license deletion, or the four
protected Rust API files.

The exact recomputation command and per-path status/unstaged/staged hashes are retained in
`protected-path-manifest.json`. The manifest preserves the inherited dirty state instead
of claiming these paths were clean.
