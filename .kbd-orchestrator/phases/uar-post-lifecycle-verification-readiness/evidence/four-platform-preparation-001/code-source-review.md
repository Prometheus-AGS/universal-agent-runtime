# A17/P16.002 source review

Verdict: FAIL_SOURCE_ONLY

Source identity supplied for review: sha256:580f7196999aa00b78e211a9894424b0d8c4074acf07fbafb483d00592e8673f.
Active phase read from the waypoint: uar-post-lifecycle-verification-readiness.
This is an independent artifact-only source review. It is not runtime acceptance. Native same-family reviewer isolation is weaker than cross-model review.

## Required finding

[HIGH] iOS bypasses the required authorized same-item correction and rerun

File: scripts/acceptance/platform_suite.py:165–167

Exact source:

    result["platforms"]["ios"] = run_ios(config, fixtures["macos"], packet, evidence, shared)
    except Exception as error:
        result["platforms"]["ios"] = {"status": "BLOCKED" if isinstance(error, Blocked) else "FAIL", "error_type": type(error).__name__}

The corresponding invocation in scripts/acceptance/ios/runner.py:240 executes one seed/restore XCTest command with the default allow_failure=False. The command helper raises on a nonzero exit. The surrounding execute function has only a finally block at lines246–248, which removes the unchanged private input and lets the exception escape. The outer main catches it and exits1. There is no per-item correction request, correction-authority consumption, corrected artifact admission, or same-item rerun on this path. The suite catches the error once and proceeds to teardown.

Failure scenario: an admitted seed or restore invocation fails a product predicate while the packet still has remaining time and repair capacity. Even when actual host correction authority exists, this path never offers the required one targeted correction and one same-item rerun. Later independent repetitions are also abandoned. This violates the assigned failure-policy requirement and cannot be cured by the macOS/Linux failure_sweep controller: that controller returns before this separate run_ios call, and run_ios has no correction-authority parameter.

The existing timeout, private-file cleanup, and budget ownership checks constrain execution but do not implement the missing failure transition. The source does preserve first-failure behavior for this path; it lacks the explicitly requested collection policy.

Required correction: define iOS item identity and dependency/cleanup boundaries, preserve the initial failure receipt, admit at most one actual-authority correction and one rerun of that exact item under the same cumulative epoch and repair cap, then retain unresolved status and continue only independent work after confirmed owned cleanup. Do not add an automatic repair or repeat the whole existing output scope.

## Scope and evidence limits

Stopped at the first required blocker and notified the lead. No repairs were made. Not all38 changed files received complete review, so this report must not be interpreted as approval of unreviewed artifacts.

Inspected source with filesystem reads only:
- AGENTS.md; .kbd-orchestrator/current-waypoint.json; .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/four-platform-whole-phase-inventory.json.
- scripts/acceptance/prepare_supporting_inputs.py; prepare_platform_inputs.py; failure_sweep.py; platform_suite.py; linux_runtime.py; core.py; budget_proxy.py; compose_fixture.py; tools_fixture.py; user_outcomes.py. Some combined initial output was truncated; complete retained text or later bounded reads were used for the reported finding.
- scripts/acceptance/platform_suite.README.md; prepare_platform_inputs.README.md.
- scripts/acceptance/ios/runner.py; README.md; Tests/EmbeddedAcceptance.swift; rust/lib.rs.
- Review guidance: prometheus-rust-workspace/SKILL.md, project rust-best-practices/SKILL.md, and review-verification-protocol/SKILL.md. No Rust-specific verdict is issued.

Tools actually used: functions.exec for tool discovery/orchestration and text slicing; mcp__rust_mcp_filesystem.read_multiple_text_files; mcp__rust_mcp_filesystem.read_file_lines. The report destination was read first and returned "No such file or directory"; this report is a new file written with mcp__rust_mcp_filesystem.write_file. Collaboration send_message notified the lead of the first blocker. No shell/terminal commands, tests, syntax checks, builds, imports of reviewed modules, service/provider/network calls, or code/config edits ran.

Runtime remains UNVERIFIED for all platforms. The supplied source identity was read, not independently recomputed. Windows remains BLOCKED and Android DEFERRED. Original budget/cap/history preservation is not newly certified by this bounded source review.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 0 | info |
| LOW | 0 | note |

Verdict: WARNING — 1 HIGH requirement failure; FAIL_SOURCE_ONLY. No runtime PASS.
