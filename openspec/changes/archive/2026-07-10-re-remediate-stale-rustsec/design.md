## Context
Assessment §2. Verified states: quinn/hickory orphaned-or-feature-gated;
lopdf/quick-xml all via kreuzberg v4.9.8; upstream main renamed to xberg
1.0-rc (too heavy to adopt mid-cycle).

## Goals / Non-Goals
**Goals:** zero stale suppressions; GitHub alerts cleared; honest remainder.
**Non-Goals:** adopting xberg 1.0-rc; fixing rsa Marvin (unfixable); opendal
quick-xml (upstream latest still 0.39 — disclosed).

## Decisions
- D1: fork-and-bump kreuzberg v4.9.9 (smallest verified diff) over adopting
  the renamed 1.0-rc line; upstream PR filed so the fork is temporary.
- D2: REMOVE sandbox-microsandbox rather than keep disclosing it — the
  feature never compiled, was excluded from all CI, and was the sole path
  pinning vulnerable hickory-proto. Wasmtime/remote runners unchanged.
- D3: html-to-markdown-rs constrained to 3.5.x in the fork (3.6+ made a
  semver-breaking type change kreuzberg 4.9.x doesn't compile against).
- D4: remaining ignores (3) each carry the blocking condition inline.

## Risks / Trade-offs
- [Fork drift] → upstream PR filed (xberg-io/xberg#1241); Cargo.toml comment
  says to repoint on the next upstream release containing the bumps.
- [Anyone using execution_mode=microvm] → falls through to wasmtime/remote
  selection; settings enum no longer offers it. Pre-1.0, no released users.

## Migration Plan
Reverting = repoint kreuzberg at v4.9.8 tag + restore the feature commit.

## Open Questions
(none)
