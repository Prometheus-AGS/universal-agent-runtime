# Source publication while verification is closed

Operator decision, October20,2026 (America/Chicago): **Allow source-only commits;
keep verification closed.** This explicitly authorizes `LEFTHOOK=0` for the
scoped implementation commit and necessary documentation follow-ups, overriding
the repository policy-validator prohibition only for those commits. Stored
hook/validator configuration is unchanged. No policy compliance result is claimed.

## Actual delivery

- UAR implementation source commit: `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`, pushed to
  `codex/uar-1-1-1-implementation`.
- UAR draft PR: https://github.com/Prometheus-AGS/universal-agent-runtime/pull/418.
- Boss draft PR: https://github.com/Prometheus-AGS/the-boss/pull/74 on
  `codex/uar-lifecycle-payload`. Its schema2 local candidate pin names the exact
  UAR source commit above. Published Boss commit
  `7e7ff9f696e7f32f69c223a9aaa49b815134a23b` is signed and DCO-signed off;
  GitHub reports its signature verified with reason `valid`. The final commit
  uses the already registered Boss release key and account noreply identity
  through per-command options. No account key or stored identity setting changed.
  Signature status is recorded separately from product acceptance.
- Requested deployment-image workflow: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39523808909, confirmed running at source
  `ad557b0d00134c003bf0e87d248d1aef2e9ce4e5`. Both native architecture jobs are active. **No image digest or
  available image is claimed until publication completes.**

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
unchanged deployment workflow was dispatched with:

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
