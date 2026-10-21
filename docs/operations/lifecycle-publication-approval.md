# Source publication while verification is closed

Operator decision, October20,2026 (America/Chicago): **Allow source-only commits;
keep verification closed.** This explicitly authorizes `LEFTHOOK=0` for the
scoped implementation commit and necessary documentation follow-ups, overriding
the repository policy-validator prohibition only for those commits. Stored
hook/validator configuration is unchanged. No policy compliance result is claimed.

## Actual source publication and delivery history

- Initial UAR implementation: `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`; published on the existing topic
  branch and draft PR418.
- First Docker delivery: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39523808909. Both native architecture jobs failed19
  compiler diagnostics. Manifest merge was skipped; no image was published.
- Repaired immutable UAR source: `6d50cb8a05fb2af33e0928dc8d7ac3ff0c5f3e46`, committed and pushed after all
  runtime/state/MCP repair handoffs. No standalone compiler or test ran.
- Boss renewed pin and proposed notes: `2888c6ff368fbc52ec51c7ff4a59c493c4487d84`, published in draft
  PR74. GitHub reports its signature valid/Verified; DCO signoff is retained.
- Both drafts remain open: https://github.com/Prometheus-AGS/universal-agent-runtime/pull/418
  and https://github.com/Prometheus-AGS/the-boss/pull/74.
- Replacement Docker delivery: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39528865081, confirmed at repaired source `6d50cb8a05fb2af33e0928dc8d7ac3ff0c5f3e46`. Both architecture jobs are active. Only the same explicitly approved workflow command ran. No image or digest is claimed yet.

The UAR source commit used exactly:

```sh
LEFTHOOK=0 git commit --file /tmp/uar-lifecycle-source-commit-message.txt
```

Normal pre-commit would execute policy validation and frontend lint/typecheck;
Commitlint runs in the message hook. Those checks were not run. The Boss's
pre-existing `core.hooksPath=/dev/null` setting was observed and preserved. No
personal credentials, runtime settings or unrelated skills gitlink was changed.

## Deployment exception and exact command

The operator requested Docker image delivery and then explicitly approved the
source-only publication path and requested Docker deployment build. The existing
unchanged deployment workflow was initially dispatched with:

```sh
/opt/homebrew/bin/gh workflow run deploy.yml --repo Prometheus-AGS/universal-agent-runtime --ref codex/uar-1-1-1-implementation
```

This is the narrow image-delivery exception. It does not open routine tests,
lint, formatting, typecheck, compiler probes, local/native builds, packaging,
policy validation, independent review or acceptance. The workflow builds native
linux/amd64 and linux/arm64 images, publishes digests and assembles a long-SHA
tag. Its Vite/Cargo builds are delivery steps, not product-test evidence. No
workflow or existing workflow-invoked script was edited for this continuation.
Metadata observations may record job progress and the actual published digest;
dispatch alone never establishes image availability.

## Boss candidate boundary

The standalone native producer is pinned to actual committed source. It exports
immutable root/gitlink objects and uses a fresh target directory; unrelated dirty
worktree files are excluded. Neither that producer nor local/public customer
packaging has run. The exact deferred command packet remains separate.
Public and other-platform release payloads are not relabeled. The proposed
bilingual Boss design note stays proposed because acceptance is unexecuted.

Verification remains CLOSED. No silent source inventory, task transition,
signature status or successful deployment build can authorize routine verification.


## Startup Send repair publication — 2026-10-21

Replacement run39528865081 failed its ARM64 build with two diagnostics from one startup tracing await; AMD64 remains active at this boundary. No multi-architecture image was published. Source-only follow-up `32cb3bcfc9691212ec0b0359ce22474968bc6850` moves awaited startup logging values before tracing macros and preserves retained ownership. Boss signed follow-up `a609869079bf6d51af196944965e6f86ee6c0758` pins that exact source and refreshes the proposed bilingual note. GitHub reports the signature valid/Verified. Writer handoffs are collected; next same-workflow delivery is pending inventory renewal. No routine verification ran; CLOSED.


## Third authorized delivery invocation

Renewed source inventory at waypoint3175 preceded run https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39532243053. GitHub confirms its immutable head `32cb3bcfc9691212ec0b0359ce22474968bc6850`. Boss pin `a609869079bf6d51af196944965e6f86ee6c0758` matches. Actual image/digest outcome remains pending; no extra preflight or routine gate ran. CLOSED.


## Completed requested image delivery

Run https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39532243053 completed success at immutable production source `32cb3bcfc9691212ec0b0359ce22474968bc6850`. ARM64 and AMD64 jobs plus manifest job119019008132 succeeded. The workflow inspection reported combined `sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8` with linux/arm64 and linux/amd64, published under tag `sha-32cb3bcfc9691212ec0b0359ce22474968bc6850`. No separate local inspection/pull/smoke test ran. Both draft PRs remain draft; native Boss candidate construction, public/customer packaging, all routine verification and runtime acceptance remain CLOSED. The receipt and renewed inventory preserve every outstanding evidence obligation.


## Approved embedding repair publication — 2026-10-21

Completed9.6/9.7 source repairs and existing embedding PASS receipts published as d35234ba132988e5d64b1e19aea688c7fa978690 to existing PR418 branch. Prior operator source-only hook exception used for scoped commit; no stored hook settings or new verification. Preserved unrelated seed-tool clone/newline edits outside commit. Passing saved worktree identity remains3189/plan82 digest3902617d5158f3556f5206c69513022fc3783f95c193638764f0fcca951d878a, not clean-checkout proof for published commit. Image/Boss pin remain32cb3bcf. Budgeted Pending shutdown API still awaits operator decision; no source work remains in current embedding repair cohort.

Both scoped commit and push exited0. The existing branch advanced aea2f0f34..d35234ba1. No new image or candidate was built; source publication does not reopen verification. See `lifecycle-approved-embedding-repair-publication-2026-10-21.md` for exact commands, scope and evidence limits.


## Budgeted Pending shutdown source publication — 2026-10-21

Approved budgeted shutdown API9.8 source-complete and published as0f926ed6deee26c353278640e3d7c1f19982bc55 in existing draft PR418. Exact saved-source boundary3196/plan86 digest251e9c10ee2a67d3f18164aeca6710a3136d46d2516e383da85ebd8a495ed0a6. Runtime/embedded interfaces, UUID retained owner, explicit Pending inventory, completion recheck and docs/spec handoffs complete. No tests or verification ran; no new gates, deps or wire changes. Earlier embedding pass predates this edit. Image/Boss pin unchanged32cb3bcf; uncooperative work/executor lifetime and missing execution evidence remain limits.

Scoped source-only commit and push exited0; branch advanced15f2fa38c..0f926ed6d. See `lifecycle-budget-publication-2026-10-21.md`. No stored hook setting changed and no verification is claimed.
