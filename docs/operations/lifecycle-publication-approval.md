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
