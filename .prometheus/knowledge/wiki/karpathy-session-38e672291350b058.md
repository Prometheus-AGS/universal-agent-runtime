---
type: SessionRecord
id: karpathy-session-38e672291350b058
title: Karpathy session 38e672291350
tags:
- karpathy
- session-learning
sources:
- session:444a77f7-71ee-4a60-8ce1-1b19da7d22ab
timestamp: 2026-08-09T09:45:34.912519+00:00
created_at: 2026-08-09T09:45:34.912519+00:00
updated_at: 2026-08-09T09:45:34.912519+00:00
revision: 0
---

## Delta

`git check-ignore` returned nothing — the `.bak` is **not ignored** and would be committed by a directory-level add. That's a real defect worth flagging. Everything gathered; here are the results.

## Task 1 — resolution 3 applied

**Correction accepted on both points.** All five phase-independent rules are already in `.claude/rules/rust.md` — nothing added, nothing duplicated into `AGENTS.md`:

| Rule | Line |
|---|---|
| Zero warnings | 22 |
| Never `cargo clean` | 24 |
| Batch fixes | 29 |
| CI as asynchronous evidence | 31 |
| Linux/macOS Stable, Windows Experimental | 54 |

And you're right that I under-read the SBOM evidence. **Absence of the artifact is positive evidence of absent work**, not an unrecorded state — I treated it as merely consistent with PENDING when it actually settles that one item. The RESOLVED entry now says so explicitly, and carries the sentence you asked for unsoftened: *"A 1.0.0 release without supply-chain artifacts is a gap in exactly the thing being sold."*

`grep -n OPEN` returns nothing. The KBD ledger was not touched.

## Task 2 — proposed `versions.toml` (paste this; I did not edit it)

```toml
# Authoritative architecture decisions and dependency pins.
# Agents must not contradict this file, and must not edit it — .claude/settings.json
# denies Edit(versions.toml). Change it deliberately, by hand.

[meta]
created = "2026-08-09"

[pins]
rust-toolchain = "nightly-2026-07-18"   # rust-toolchain.toml; floating nightly deliberately avoided
rust-edition   = "2024"
tokio          = "1.50.0"
axum           = "0.8"                  # resolves 0.8.9
surrealdb      = "=3.2.4"               # exact — schema-coupled
rmcp           = "=2.2.0"               # exact — MCP wire compatibility
wasmtime       = "47"                   # must move in lockstep with wasmtime-wasi
wasmtime-wasi  = "47"                   # skew breaks Linker<T> unification
cedar-policy   = "4.9.1"
ractor         = "0.16.3"
liter-llm      = "vendor/git/liter-llm" # path dep, not a registry version
react          = "^19.2.8"
typescript     = "^5.9.3"
vite           = "^8.2.0"
pnpm           = "11.15.0"              # packageManager field, root and frontend

[decisions]
feature-profile     = "server-full is the checkpoint feature set and the BossFang sidecar profile"
ui-authority        = "React 19 is the authoritative first-party UI; HTMX/Web Component material is historical"
llm-path            = "All LLM access goes through liter-llm; no direct provider SDKs"
worktree-root       = "Git worktrees live under ~/.claude/worktrees/, never inside the repo tree"
wasmtime-lockstep   = "wasmtime and wasmtime-wasi share a major; a skew broke server-full on 2026-08-08"
release-1-0-0-gap   = "v1.0.0 published 2026-07-11 with 4 certification changes still PENDING; supply-chain artifacts absent. See .prometheus/decisions.md."
```

I kept it to pins that are enforced or that have already caused a failure. The three `=` exact pins and the wasmtime lockstep note are load-bearing; the rest are the versions actually resolved today.

## Task 3 — `model-fleet.md` filled

Four rows, all routed through liter-llm. Every "Scaffold needed" is `unknown`, every "Measured" is `no`, profile stays `mixed`. I added a line stating plainly that nothing has been measured and that `mixed` is a default, not a result.

## Task 4

**(a) I could not run `/doctor` as specified.** The CLI `claude doctor` reports installation health only — its own closing line says the full checkup requires `/doctor` inside an interactive session, which I can't launch from here. Verbatim output ends with `No installation issues found.` and **never prints a skill-listing budget line**.

So I measured directly instead:

```
15 skills · 1,513 description chars total · ~378 tokens
skillListingBudgetFraction: 0.02  →  ~4,000 tokens at a 200k window
```

Roughly **10× headroom; no drops expected**. Worth noting: 3 of the 15 (`iterative-evolver`, `kbd-process-orchestrator`, `pmpo-skill-creator`) h

## Root Cause

No explicit root-cause section was captured; preserve this as a session record, not an inferred diagnosis.

## Corrective Actions

Review and promote only reusable findings.

## Session Metadata

- Harness: claude-code
- Session: 444a77f7-71ee-4a60-8ce1-1b19da7d22ab
- Captured: 2026-08-09T09:45:28.770377Z
- Project: /Users/gqadonis/Projects/prometheus/universal-agent-runtime

## Changed Paths

- No changed paths detected.
