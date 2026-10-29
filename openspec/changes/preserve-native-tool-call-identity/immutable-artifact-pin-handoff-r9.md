# Immutable replacement artifact and consumer pin handoff

Preparation only, 2026-10-25. No publication, pin adoption, runtime acceptance or
merge is authorized by this document. The source preparation plan is
`backend-submission-preparation-r9-plan.md`. Preserve r8 evidence against its
original tree; successor source has no inherited runtime acceptance.

## Deliverable state

| Item | State |
| --- | --- |
| Historical r8 source tree | `f1deb3ccb29e75003c46abd8e545429e0c8512d7` |
| Historical r8 identity selection | 10 passed, none ignored |
| Historical r8 customer build | exit0 |
| Historical r8 local extended gate | exit2, `extended_passed_remaining_release_gates`; not release acceptance |
| Retained protocol target | r7,11/11; not scheduled for rerun |
| Successor backend gates and submission checks | Await complete settings, source review/freeze and exact instruction |
| Corrected UAR source commit/push | Pending; staged source is not a public commit |
| New release version/reference | Pending operator publication decision |
| New immutable index/platform/provenance/run | Pending actual authorized publication |
| Kubo replacement pin and model continuation | Pending actual delivery and consumer approval |
| WebUI PR13 | Draft; no merge readiness claim |

## Publication source contract

The existing `.github/workflows/deploy.yml` builds native linux/amd64 and
linux/arm64, then merges platform digests. It triggers on main, `v*` tags or
workflow_dispatch; the sidecar and old1.2.0-alias branches have separate behavior.
No workflow or Dockerfile edit is part of this scoped repair.

Dockerfile's actual command is a locked release build with
`--features server-full,postgres-backend`; it does not pass
`--no-default-features`. Current Cargo default is minimal, and server-full
already includes minimal. This source closure explains the feature relation;
it does not prove identical artifacts, platform behavior or packaging.

The package remains version1.3.0. A branch dispatch with an immutable SHA tag
could produce a candidate, but Kubo's `mcp/uar/requests/CONFORMANCE.md` requires
a published tagged release and acceptance against that artifact. A branch
candidate cannot satisfy that gate. Select a genuinely new release reference
and resolve its package/release-version policy before authorizing publication.
If that decision requires source changes, reopen the source boundary, integrate
them and request verification against the new identity. Do not repoint existing
1.3.0/v1.3.0 tags or silently weaken Kubo's published-artifact requirement.

The publication approval must name the exact corrected source commit, full tree,
unused ref, workflow/ref/inputs, expected two native platforms and the registry
destination. Merely pushing a matching `v*` tag or main can start publication;
those pushes need the same deliberate approval. Keep secrets in supported
workflow secret channels. Product tests remain local.

## Required actual artifact delivery

The publisher must deliver all of the following, retaining failures separately:

- Signed-off conventional source commit, full tree, parent, patch and complete
  path inventory, plus unchanged Cargo.lock/toolchain/Dockerfile/workflow hashes
  and every submodule gitlink. Bind the final source review to those exact bytes.
- Build workflow/run/attempt/job identities and exact source checkout/ref.
  Distinguish native platform builds from the index merge and local debug build.
- Raw canonical OCI index bytes and their actual SHA256; actual platform
  manifests/configs and attestations, with linux/amd64 and linux/arm64 mappings.
  Record provenance source/revision and signature verification limits without
  treating unsigned same-publisher statements as independent authentication.
- The release/tag/package identities, the immutable digest pull reference and
  exact acceptance receipts against that artifact. No copied local binary hash
  substitutes for the OCI index. No fictional success/date/digest field.
- Private runtime inputs and an owned startup/cleanup procedure using environment
  secrets, authenticated durable storage, writable ingest directory and explicit
  disabled/unconfigured third-party services. Public configuration has no tokens
  or credential-bearing URLs. Failed/partial cleanup remains a blocker.

The predecessor remains
`ghcr.io/prometheus-ags/universal-agent-runtime@sha256:f67a36f1a6fb886379c6947f605a88945a94481c8b3c4e1d998b9b481cea0697`,
source `774469746fee4de99be41dc1cd231857067e4487`, publishing run40091743841.
Its pre-release verification-pending status and historical failure evidence are
preserved. Do not rewrite the old pin report as evidence for a new artifact.

## Consumer owner and exact integration sequence

Consumer root is
`/Users/gqadonis/.codex/worktrees/agentic-ui-frontend/kubo`; Desktop sibling is
`/Users/gqadonis/.codex/worktrees/agentic-ui-frontend/ipfs-desktop`. The frontend
publication checkout is `ipfs-webui-issue10-publication` in that same parent.
Kubo architect owns contract/pin/OpenSpec integration; the UI lane owns export
and cumulative presentation evidence; Desktop inputs retain their own owners.

Known replacement source paths are `mcp/contract/agentic/uar-preflight.json`
(`pinnedFrom.artifact` and report), `mcp/contract/uar_test.go` expected artifact/
report, a new accurate `mcp/uar/` pin report, and
`mcp/uar/requests/CONFORMANCE.md` with new release facts and unresolved acceptance.
Preserve `mcp/uar/v1.3.0-pin.md` as history. Before edits, expand the full exact-path
delivery inventory to every artifact/lease/source-guard binding discovered from
the actual delivered index, with recorded preimages. Never stage wildcard paths.

Guarded integration must reconcile the new report and every changed binding;
the historical fixed115-path procedure cannot silently accept an added path.
Do not copy foreign files by hand. Stop writers and post a new combined freeze
and inventory hash on the boundary board before any consumer generation or
verification. Normal WebUI export/Desktop assembly and any implementation drift
require a renewed consumer boundary and exact post-generation approval.

Runtime ownership must be freshly delivered. The previous Liter owner record
SHA256 `3e7945ff7e389bbca4faa4c3fea66773228cf1bae6c569a4627cf785728bec86`
is historical; its child3707/supervisor3693 are not current authority. No blind
kill, lease adoption, fallback host or weaker validator is permitted. Bind the
real configured provider/model and new immutable artifact to fresh compatibility
receipts. Required native success must actually resume successfully; a refused
missing-ID negative does not establish native success.

Only after that integration and exact approval may Kubo run its selected backend,
principal, boundary-lease, secret-handling and real native/model gates, then the
affected cumulative WU/SU pair with owned cleanup and final independent product/
security reviews. Preserve existing static/node/contrast and broader-packet
distinctions. Update PR13's description with actual final composition and results
before requesting draft-to-ready transition; no merge is included.

## Submission holds

UAR's workspace format check is `cargo fmt --all -- --check`, spanning more than
this patch. It is not yet authorized. No automatic workspace formatting repair
is authorized. The whole-phase server-full test suite remains broader pending
certification; this scoped backend family cannot be relabeled whole-phase green.
The observed vendored unused-import warning and Darwin compact-unwind warning
remain unresolved. No vendor/pin/profile suppression is included.

CONTRIBUTING requires DCO sign-off and Conventional Commits. A future exact
`git commit -s` request must include the pre-commit
`pnpm github-actions-policy:validate` hook and commit-msg
`pnpm exec commitlint --edit <actual-message-file>`; frontend hook globs apply
only if corresponding frontend source is staged. Do not bypass hooks. No commit,
push, format check, validator, build or publishing dispatch ran during preparation.

The uncomfortable limit is that a locally passing source repair is not a
published release, consumer compatibility result or ready-to-merge composition.
