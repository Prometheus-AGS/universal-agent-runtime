# Verification

Date: 2026-08-21
Scope: `universal-agent-runtime` `server-full`; local macOS arm64 and local Linux arm64 container only

| Requirement | Command or evidence | Observed result | Limit |
|---|---|---|---|
| Replacement transports survive filtered and merged registry view boundaries | Focused exact test in `evidence/tier-zero-and-focused-tests.md` | `1 passed, 0 failed`; a later independently filtered call and a pre-existing merged view both used the same replacement process | In-process fixture; packaged boundary is covered separately below |
| Upsert preserves slot identity and authoritative reconnect configuration for existing views | Focused exact test in `evidence/tier-zero-and-focused-tests.md` | `1 passed, 0 failed`; after an A→B upsert, an old filtered view crashed B, reconnected, and the next call remained on B; A received no post-upsert call | Private registry behavior only; no public API change |
| Crash fails closed and is not replayed | `evidence/mcp-crash.sse`, `evidence/mcp-process-trace.jsonl`, and `evidence/mcp-process-boundary-validation.txt` | One failed tool-result event; one crash execution in PID 58390; next echo used PID 58463 | Local installed preflight for source `f0298d76`; not the parent three-hour soak |
| Timeout fails closed and is not replayed | `evidence/mcp-timeout.sse`, `evidence/mcp-process-trace.jsonl`, and `evidence/mcp-process-boundary-validation.txt` | One failed tool-result event; one timeout execution in PID 58463; 30 seconds observed against 30 configured; next echo used PID 58743 | Local installed preflight for source `f0298d76`; not the parent three-hour soak |
| Reconnect recovery preserves authorization | Focused filtered/merged test and six negative controls in `evidence/tier-zero-and-focused-tests.md` | Excluded server/tool views remained empty; all six fail-closed controls were rejected | Authorization was verified at registry-policy boundaries; no claim for unrelated policy systems |
| Candidate is the immutable child source | `evidence/installed-preflight-results.json` and `evidence/installed-artifact-preflight.md` | Source SHA `f0298d76ea3c39853020c8a33e13f136c07a1806`; candidate tag `operational-resilience-f0298d76ea3c`; outcome `passed` | Frontend build regenerated derived `static/` output from that source; no input manifest, lockfile, source, or submodule pin changed |
| Tier 0 and focused Tier 1 | Exact commands in `evidence/tier-zero-and-focused-tests.md` | `cargo check` exit 0; package Clippy exit 0; two focused registry tests pass | Three pre-existing package warnings remain outside the child edit; no zero-warning repository claim |
| Local operational preflight | `DOCKER_CONTEXT=orbstack UAR_SOAK_DURATION_SECONDS=60 scripts/certify-operational-resilience-local.sh preflight` | `LOCAL_OPERATIONAL_RESILIENCE_PASS`; focused operational suite `5 passed, 0 failed`; installed candidate certification passed | Local only; no GitHub Actions; not GA certification |
| GitHub Actions deployment-only policy | `pnpm github-actions-policy:validate` | Passed; only `deploy.yml`, `docs.yml`, and `typescript-sdk-docs.yml` accepted | Proves workflow policy shape, not deployment success |

The uncomfortable result retained by this artifact is the original failure:
the pre-child installed trace stopped after `echo, crash`, later calls never
reached a replacement process, and the supposed timeout returned in about 0.2
seconds. The corrected candidate is accepted only because the retained raw
stream and process evidence now demonstrate the inverse at the installed
boundary.
