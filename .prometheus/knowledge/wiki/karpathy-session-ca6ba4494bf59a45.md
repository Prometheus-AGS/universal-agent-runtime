---
type: SessionRecord
id: karpathy-session-ca6ba4494bf59a45
title: Karpathy session ca6ba4494bf5
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-11T06:25:25.291360+00:00
created_at: 2026-08-11T06:25:25.291360+00:00
updated_at: 2026-08-11T06:25:25.291360+00:00
revision: 0
---

## Delta

Independently verified.

```
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 62 filtered out; finished in 264.16s
```

**29/29 pass.** exit 0. Run on my own checkout of `main` at `38d41a42`, with the pinned command — not read from Codex's artifacts.

The four earlier "1 passed / 90 filtered" lines were `harness::tests::process_server_helper` self-tests running individually. I misread the first of them as the matrix result and said so; that was wrong, and this is the corrected number.

## Where the phase stands

| | |
|---|---|
| Cases | **29/29 pass**, 264s |
| Capabilities | **27/27 covered**, none missing |
| Evidence | 1×L4, 14×L3, 4×L2, 1 shape-only, 3 absent, 6 excluded |
| Tasks | 44/44 across all three changes |
| Merged | `38d41a42` (PR #240) |

**What you can now say, at full strength:** *On `server-full`, against a recorded in-process stub, in a harness-created database — 27 of 27 capabilities measured, 21 passing at L2 or above, one surviving a cold process restart, six structurally unreachable with reasons named.* Reproducible by anyone with one command.

That's the first trustworthy statement about what this runtime does since v1.0.0 shipped a month ago.

## Two things I'd still put in front of you

**The CI requirement is broader than your decision.** The merged spec says *"GitHub Actions MUST NOT run the matrix or other unit, integration, conformance, lint, format, or routine development checks."* You said defer until the code works — temporal. That's categorical, and permanent. It also means the existing `CI`, `Coverage`, `BDD Chat Scenario Suite`, and `Cookbook examples` workflows should be *retired* rather than fixed. Worth confirming which you intend.

**Six exclusions, where the reviewed spec sanctioned one.** Codex's reasoning is sound — UAR genuinely has no `frf-did`/`frf-wallet` dependency, and there's no cross-tenant surface to test C-21 against. But it's a scope change that never went through the critic, and the exclusions are load-bearing in the final table.

The measurement is done and it holds up. What it measures is still narrow — no semantics, no real provider, one profile — which is exactly what `goals.md` says it is.

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-11T06:25:13.802731Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
