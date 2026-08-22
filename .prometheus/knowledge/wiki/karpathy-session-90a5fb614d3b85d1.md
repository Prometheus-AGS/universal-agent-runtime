---
type: SessionRecord
id: karpathy-session-90a5fb614d3b85d1
title: Karpathy session 90a5fb614d3b
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-09T16:30:29.005508+00:00
created_at: 2026-08-09T16:30:29.005508+00:00
updated_at: 2026-08-09T16:30:29.005508+00:00
revision: 0
---

## Delta

Reviewed. **Critic: CLAIM OVERSTATED. Judge after corrections: ACCURATE.** Here's the answer to your question.

## Your recollection is half right, and the half that's wrong is the important half

**You did compile for WASM.** `rust-toolchain.toml` pins `wasm32-unknown-unknown`, `wasm32-wasip1`, `wasm32-wasip2` alongside the iOS and Android targets. A `wasm-runtime` feature exists and works — I fixed its wasmtime skew yesterday.

**But it does not solve skills on mobile, for two independent reasons.**

**1. `embedded-mobile` doesn't enable it.**
```
embedded-mobile = ["host-persistence"]
wasm-runtime    = ["dep:wasmtime", "dep:wasmtime-wasi"]
```
Separate features. Nothing transitively pulls one into the other.

**2. Even enabling it wouldn't fix this — and this is the part the critic named a category error.** `wasm-runtime` adds the wasmtime *engine*. It bundles no `.wasm` skill modules and calls no registration path. **Runtime capability is not content availability.** Turning it on gives mobile the ability to execute WASM skills; it does not give it any skills to execute.

The actual mechanism is simpler and unrelated to WASM: `register_builtins` is called **only** from `src/server.rs` (lines 447, 510). `src/embedded.rs` never calls it. That's the whole of GAP-05.

## Where I was wrong, and where the spec is wrong

**My error:** I said "skills are empty on embedded/mobile." The critic pushed back and was right. `src/embedded.rs:360-371` constructs a `SkillService` with a `DatabaseStorageProvider`, and `initialize()` loads whatever that provider returns. So a device with prior writes boots with a **populated** registry.

The accurate statement: **built-ins never register on mobile.** Persisted skills do load. Empty overall only on a fresh device.

**The spec has the same overstatement, and worse.** GAP-05 says *"Embedded/mobile boots with an empty skill registry... this capability is at 0%."* The judge's ruling:

> *"'0%' conflates builtin availability with capability. The skill capability — registry, providers, persistence, listing, execution plumbing — demonstrably functions... The '124 correct routes, one capability at 0%' framing is rhetorically punchy but technically wrong."*

That framing is load-bearing in the spec — it's cited as *"the live proof"* of the method's central argument. It overstates what the code shows.

## What this changes

**GAP-05 is smaller than advertised.** Not "skills don't work on mobile" but "built-ins don't auto-register on mobile." The fix is plausibly calling `register_builtins` from `embedded.rs` — not a WASM architecture problem.

**The judge's recommendation:** amend `SPECIFICATION.md` GAP-05 to read *"empty of built-in skills"* and *"built-in skill capability at 0%"* — otherwise a future reviewer flags it again.

I'd hold that edit until you say so. The spec is the measuring stick for the phase Codex is executing, and changing it mid-measurement is exactly what the execution contract lists as a stop condition. It's a correction worth making, but deliberately and probably after the conformance run — not now.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-09T16:17:28.479360Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
