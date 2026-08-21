# Immutable installed-artifact preflight

Date: 2026-08-21
Profile: local macOS arm64 archive plus local Linux arm64 `server-full` container
Source commit: `f0298d76ea3c39853020c8a33e13f136c07a1806`

## Candidate construction and execution

Commands:

```bash
git worktree add --detach \
  /Users/gqadonis/.claude/worktrees/uar-release-candidate-f0298d76 \
  f0298d76ea3c39853020c8a33e13f136c07a1806
git -C /Users/gqadonis/.claude/worktrees/uar-release-candidate-f0298d76 \
  submodule update --init --recursive
cd /Users/gqadonis/.claude/worktrees/uar-release-candidate-f0298d76
DOCKER_CONTEXT=orbstack UAR_SOAK_DURATION_SECONDS=60 \
  scripts/certify-operational-resilience-local.sh preflight
```

Observed exit: `0`

Observed result tail:

```text
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

universal-agent-runtime 1.0.0
Candidate artifact certification passed; evidence: target/resilience-certification/installed-runtime/results.json
Candidate certification evidence passed for operational-resilience-f0298d76ea3c (f0298d76ea3c39853020c8a33e13f136c07a1806).
LOCAL_OPERATIONAL_RESILIENCE_PASS source_sha=f0298d76ea3c39853020c8a33e13f136c07a1806 mode=preflight duration=60 results=target/resilience-certification
```

The fresh worktree remained at the named commit. The local frontend build changed
only derived files below `static/`; no committed source input, manifest, lockfile,
or submodule pin changed during certification.

## MCP process-boundary result

Observed validation output:

```text
MCP_PROCESS_BOUNDARY_EVIDENCE_PASS crash_failure_events=1 timeout_failure_events=1 crash_calls=1 timeout_calls=1 reconnects=2
```

Observed process sequence:

```jsonl
{"pid": 58390, "request_id": 2, "mode": "echo"}
{"pid": 58390, "request_id": 3, "mode": "crash"}
{"pid": 58463, "request_id": 1, "mode": "echo"}
{"pid": 58463, "request_id": 2, "mode": "timeout"}
{"pid": 58743, "request_id": 1, "mode": "echo"}
```

The crash used the initial process and emitted exactly one failed normalized
tool-result event. The following independent echo used PID 58463. The timeout
then used PID 58463, lasted 30 seconds against the configured 30-second
boundary, and emitted exactly one failed normalized tool-result event. The
following independent echo used PID 58743. Neither failed operation was
replayed.

Persisted raw evidence:

- `mcp-crash.sse`
- `mcp-timeout.sse`
- `mcp-process-trace.jsonl`
- `mcp-process-boundary.json`
- `mcp-process-boundary-validation.txt`
- `installed-preflight-results.json`
- `lifecycle.json`
- `failure-recovery.jsonl`
- `parallel-load.json`
- `soak.json`
- `upgrade.json`

SHA-256 receipts:

```text
ddea79152623addeb0df85a6b0366c4d272af812f243cbb3a42588d604546de3  installed-preflight-results.json
491ecb309ed031e1f981d80f86893754e43387e9227f94c3646498c14bd1b547  lifecycle.json
75719769b897fa50b0bb26dd530d6dce374f05e05f686898c281ef33d628b0ff  failure-recovery.jsonl
3867c71a77be386df3a6d803a22153c41a33b17b1e9666df613f0f0a871549ec  mcp-process-boundary.json
cb425ffda2be2b59b50b129d853df14c85ec273da5af8584143537a651fdb0f9  mcp-process-trace.jsonl
6f4abcbfe9709048425165465b55213f546c9f3c0eeeab5021d8c5a238a1f2fe  mcp-process-boundary-validation.txt
372c9dfe0bb59173a79aaf82162b552ebdb758a0b3ddeddd4a455c209c1eca4d  mcp-crash.sse
f155719e0a4593efe34af0901bd76631034aa25404baeb32f7bfcaf6b6a75ae8  mcp-timeout.sse
1bfd70b7941f75d9dbb6933f0f96a247e53b6dd6b2fec024a32d88ea2ab16ae5  parallel-load.json
54893a9f70d83c7d14a4eef67242471c885b62ae5f0318ba26dc05539087b96d  soak.json
a7cfca5cedb7dd3e11a905fd0e7f10eebe0e9929edb019ae24f60f7dde840f8d  upgrade.json
```

## Limits

This was the child-required 60-second preflight, not the parent phase's
three-hour soak. It proves the installed macOS archive, local Linux container,
and MCP crash/timeout sequence for source `f0298d76`; it does not certify GA or
any non-`server-full` runtime profile.
