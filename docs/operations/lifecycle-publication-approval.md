# Source publication while verification is closed

Operator authorization on 2026-10-20 covers scoped commits, push, a PR and a
Docker deployment image build when appropriate. Verification remains CLOSED.
The existing UAR branch is `codex/uar-1-1-1-implementation`; its existing draft
PR is https://github.com/Prometheus-AGS/universal-agent-runtime/pull/418.

## Concrete hook conflict

UAR's installed `.git/hooks/pre-commit` invokes Lefthook. Checked-in
`lefthook.yml` runs `pnpm github-actions-policy:validate`, and stages touching
frontend TypeScript also run frontend lint and typecheck. Its commit-message
hook invokes Commitlint. These are verification commands; committing normally
would run them indirectly while the gate is closed.

`AGENTS.md` says “Never skip, disable, weaken, or route around that validator.”
The operator has also kept verification closed. No hook has been executed or
disabled for this continuation. Source implementation continues independently.
Once the complete scoped source is ready, the lead must obtain an explicit
operator resolution of this conflict before making the UAR source commit.
An authorized source-only hook exception would not change the stored hook or
validator, certify policy compliance, or authorize any later verification.

The Boss currently has `core.hooksPath=/dev/null`; this was observed rather than
changed by this task. No promise is made that either repository passed checks.

## Deployment path

The UAR existing `deploy.yml` supports `workflow_dispatch` on a pushed ref.
The non-sidecar path builds native linux/amd64 and linux/arm64 Docker images,
publishes their digests, assembles a manifest, and publishes a long-SHA tag.
Its Dockerfile runs Vite build and Cargo build, not product tests or routine
lint/typechecks. This is the operator-requested image delivery path; it does
not open local verification or certify image behavior. No workflow or existing
workflow-invoked script is edited by this continuation.

After an actual scoped commit and push, the exact workflow command uses the
existing branch:

```text
/opt/homebrew/bin/gh workflow run deploy.yml --repo Prometheus-AGS/universal-agent-runtime --ref codex/uar-1-1-1-implementation
```

The published image must be reported with the observed immutable digest and
run URL. Dispatch success alone establishes only that a build was requested.
Do not label a queued, running or failed build as an available usable image.
The draft PR must retain the verification-closed status and source limitations.

## Boss candidate path

The new standalone `scripts/build-boss-candidate.mjs` prepares a native Apple
Silicon candidate from exact committed Git objects in an isolated directory.
It is not run while verification is closed. The Boss candidate pin cannot be
finalized before that UAR source commit actually exists. Its new explicit
candidate-record mode validates immutable commit/tree, platform, features,
protocol and artifact/log digests without consuming unrelated dirty worktree
files. Legacy local and public release integrity checks remain separate.

No prior release or other platform's binary is relabeled to this source. A
fresh native candidate build, customer package and Windows evidence still need
their own later exact-command approval.
