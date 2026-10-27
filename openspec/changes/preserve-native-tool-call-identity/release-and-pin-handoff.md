# Replacement artifact and Kubo pin preparation

This is a prepared delivery procedure, not a published image, validated release, execution grant or usable replacement pin. The user authorized this source correction and replacement-pin preparation. Verification remains CLOSED.

## Immutable predecessor

| Input | Existing value |
| --- | --- |
| UAR source | `774469746fee4de99be41dc1cd231857067e4487` |
| OCI repository | `ghcr.io/prometheus-ags/universal-agent-runtime` |
| Canonical index | `sha256:f67a36f1a6fb886379c6947f605a88945a94481c8b3c4e1d998b9b481cea0697` |
| Existing tags | `1.3.0` and `v1.3.0` |
| Existing release | Pre-release, verification pending |
| Isolated correction | `codex/uar-native-tool-identity-pr13` in this worktree |
| Consumer | Kubo/frontend integration readiness, Execute/FIR-9; WebUI PR #13 remains draft |

The source manifest must identify every corrected file and its exact predecessor. No new commit, release tag, index, platform manifest, attestation or successful run exists yet. Keep those fields pending rather than supplying a locally invented digest.

## Source and command boundary

1. Complete production code, all regression source, spec deltas and documentation. Stop every writer and capture the full owned source manifest and mechanically applicable patch, including new files. Account for the exact tracked submodule commits and unchanged Cargo/toolchain/catalog inputs. Do not modify or copy dirty source from the operator's UAR checkout.
2. Reconcile the applicable UAR canonical phase ledger before any verification. Its retained waypoint references the unfinished function-first certification phase; counters alone do not distinguish pending verification from missing production work. This task cannot claim that broader phase complete or rewrite projections to open a gate.
3. Present the implementation inventory, unresolved dependencies and exact named proposed commands to the operator. UAR AGENTS.md requires explicit execution instruction after that presentation. Existing Kubo execution grants, this task packet and a source-complete declaration do not open it.
4. Run authorized local production-path integration checks serially. The existing targets are `tool_call_protocol`, `tool_admission_integration`, `approval_recovery` and `agent_threads`; the existing profile is `--locked --no-default-features --features server-full`. Typed adapter and normalized-driver regression fixtures are supplemental source-path evidence. An omitted optional storage backend remains unverified, not certified by another backend's pass. Exact final commands must be selected from the final changed-target inventory before requesting authorization.
5. Preserve real configured-model Kubo native approval/resume, denial and direct client-tool verification as required final oracles. A fail-closed missing-ID result is a safety success for a negative case and a failure for an acceptance case requiring successful native continuation.

## Publication preparation

The existing deployment-only workflow is `.github/workflows/deploy.yml`, titled `Build and Publish Images`. Its `ghcr-build` jobs build native `linux/amd64` and `linux/arm64` images, publish per-platform manifests by digest and pass those digests to `ghcr-merge`. `Dockerfile` invokes the locked Rust release build for `server-full,postgres-backend`; product tests must stay local. No workflow, build script, dependency or toolchain edit is part of this correction.

After authorized local verification and review, publication must use the corrected immutable source commit and an unused release reference. Preserve the old `1.3.0` and `v1.3.0` references; do not overwrite them or call a modified local image the selected old artifact. Record the actual publishing command, source commit, run identity, raw canonical OCI index bytes/hash, platform manifests and provenance, and their limitations. Credentials remain in supported secret channels, never argv, URLs, logs, source manifests or public configuration. No deployment workflow or registry operation is authorized by this document.

## Consumer integration package

Once an actual replacement artifact is available, the complete Kubo source batch must contain:

- The genuine image/tag/source/index facts, preserved old pin history and an accurate new pin report; update only the necessary contract pin fields and source references.
- Exact expected artifact values in relevant test/lease fixtures and source guards, keeping validators and pass predicates strict. Do not change the model, provider endpoint, capabilities or verification proof as an artifact workaround.
- The actual runtime-owner chain and a new immutable Liter predecessor input. The last delivered idle owner is `/Users/gqadonis/.codex/worktrees/agentic-ui-frontend/kubo/evidence/agentic-ui-frontend/pr13-readiness/codex-liter-owner-20261024T162508Z/owner.json`, SHA256 `3e7945ff7e389bbca4faa4c3fea66773228cf1bae6c569a4627cf785728bec86`, child `3707`, supervisor `3693`. Those are historical ownership facts; live PID/argv and expiry must be checked only at the authorized replacement/cleanup boundary. Never relabel its r11 freeze as a new one.
- A full exact-path owner source manifest/inventory, newly reconciled preimages for any additions, complete operational attestation and stopped-writer declaration. The existing fixed115-path integration helper cannot silently accept a new pin-report file; author an explicit new reconciliation if that scope grows.
- Guarded integration and a new combined source freeze/board signal before normal generation. A fresh post-generation grant must precede actual backend receipts, compatibility and cumulative WU/SU. Preserve every earlier failure and distinguish generation, safety negatives, native runtime acceptance, independent reviews and publication.

The uncomfortable limit is that this procedure cannot manufacture a corrected immutable artifact or prove that the real provider emitted an authentic ID. Until the actual replacement delivery and real native continuation succeed, compatibility/runtime inputs, PR readiness and whole-phase certification remain pending.
