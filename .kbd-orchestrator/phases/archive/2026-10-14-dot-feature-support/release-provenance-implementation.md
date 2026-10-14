# Release provenance implementation handoff

Date: 2026-10-11. Phase: `dot-feature-support`, Execute. Change: `dot-01-release-provenance`, task 1. Producer: native `gpt-6.1-sol`, high reasoning.

## Applied prerequisite and current hold

The operator approved the exact main pins and explicitly authorized editing `versions.toml`. The applied change sets Liter to version `2.1.1`, commit `a6047386cc9fae4258b4a8577f6a016022095586`, and the skill-resource commit to `f4bef4556a83ac22bf204bf69a01ddf0f9e007f2`. Both memory policy keys and the skill historical base key remain unchanged. Policy comments record adoption of the current source baseline; they do not assert behavioral migration qualification.

The parent subsequently held new packaging/provenance production implementation until the credential slice completes integration. The one-line Docker `--locked` change had been made immediately before that steering and was reverted before the parent clarified it could remain. **Current Dockerfile has no worker diff and still lacks `--locked`.** No production provenance script was created. No task marker or generated KBD state was changed.

Observed checks:

- `pnpm github-actions-policy:validate` before Docker edit: exit 0, `GitHub Actions policy validation passed (deployment workflows only; Pages publisher: docs.yml).`
- `git diff --check -- versions.toml Dockerfile`: exit 0, no output.
- `pnpm github-actions-policy:validate` after Docker edit: exit 0, same output.
- `pnpm github-actions-policy:validate` after reverting only the worker's Docker edit: exit 0, same output.
- `git diff --stat -- versions.toml Dockerfile`: only `versions.toml`, 7 insertions and 8 deletions.

The original `release-pin-reconciliation.{md,json}` remains the dated pre-decision audit. Its original `versions.toml unchanged` check describes that audit, not this later authorized policy edit. Root owns the canonical operator decision record.

## Narrow implementation once released

Reuse the existing local release pipeline and existing evidence-file digests. Do not add a certification framework or new signing/publication path.

1. Add `--locked` to the existing production Cargo build in `Dockerfile` (line 226 at the inspected root). Preserve frozen pnpm commands, entity build order, schema exceptions and the deployment-only Actions policy. Run the policy validator locally before and after the edit; stop on a failure.
2. Add one concise proposed public CLI, `scripts/generate-release-input-provenance.mjs`, to inspect approved source identities and emit one source-bound JSON companion receipt. This path is a proposal, not an existing command. It accepts output location plus an optional repository root for real Git/filesystem integration tests. Use Node standard libraries and existing Git commands; no dependency adoption or lockfile regeneration is required.
3. Reuse `scripts/prepare-release-evidence-local.sh`'s clean-checkout and recursive-submodule preconditions. Invoke the new CLI before producing offline archives or signing. The existing `generate-release-manifest.mjs` already hashes every evidence file into `evidence.files`, and `validate-release-manifest.mjs` verifies those digests; a companion `release-input-provenance.json` can use that mechanism without changing manifest schema or signature roots.
4. Add targeted public-CLI integration tests, using real temporary Git repositories/submodules and filesystem inputs rather than mocked Git output. Proposed file: `scripts/test-release-input-provenance.mjs`. The fixture verifies policy mismatch rejection, mismatched submodule checkout rejection, version/lock disagreement rejection and a valid memory export with a COMMIT identity distinct from its TREE identity. This is supporting negative-control evidence, not a substitute for the actual clean source/build gate.
5. Update the existing `docs/release-verification.md` with actual implemented commands and evidence limits after the CLI exists. Keep product verification local. No workflow changes, signing, source archive generation, network authentication, dependency installations or builds are authorized during this hold.

## Proposed receipt contract

The CLI should fail before writing a success receipt when approved immutable input identity disagrees with source. That guard traces to the observed policy drift and the OpenSpec requirement that unresolved drift prevent certification.

Record root commit/tree and hash of `versions.toml`; exact Liter policy/gitlink/checkout, crate version and Cargo.lock version; exact skill policy/gitlink/checkout and `skills` tree; memory upstream commit as recorded in policy and `vendor/git/README.md`, exported root tree, adapted manifest hash and exported file hashes. Memory comparison must never equate an upstream COMMIT with the root TREE. The existing reconciliation independently verified every exported source file against the policy upstream commit; that historical proof remains linked rather than requiring upstream network access in release packaging.

Record hashes for Cargo.lock, root/frontend/entity-management pnpm locks, relevant manifests, provider catalog, Dockerfile, .dockerignore and compiled schema inputs. Enumerate the lockfile for each actual pnpm workspace; a root lock hash alone does not describe frontend or entity-management resolution. Record only source inputs, never complete environment contents, ignored credentials or operator-local files.

Do not claim the companion receipt is an executed runtime skill receipt. The actual host already logs `skills.pack.resolved` with `source`, `root`, `version` and `skill_count` in `src/server.rs:1058`. At the real packaging gate retain that log and record the content identity of the selected root plus extra/verified roots. Docker explicitly selects `/opt/uar/skills/builtin` through `UAR_BUILTIN_SKILLS_DIR`; this is different from repository-cwd embedded selection. No invented management endpoint is needed. A source-only receipt cannot prove which override a separate launcher selected.

## Deferred actual production gate

Root serializes all Cargo/build writers after the credential boundary. A clean exact-pin source is required before acceptance; today's dirty multi-worker checkout cannot certify a clean release. No commit or push was made by this worker.

At the approved completed-change boundary, use the implemented provenance CLI, the three existing frozen installation commands from Docker/local paths, and the actual locked Docker production build for the selected Linux architecture. Record command, exit status, source identity and produced image identifier; acceptance for both Linux architectures requires each actual build, not extrapolation from one. Preserve schema inclusion and source/lock hashes in the receipt. The offline archive path remains existing `scripts/package-offline-source.sh`, whose production `cargo vendor --quiet --locked` stages only tracked source; the existing offline-build spec requires a subsequent unpacked `cargo build --locked --offline` with empty caches and networking disabled. Neither operation ran here. Full release publication, signing and deployment health remain change 17 scope.

The uncomfortable thing: adopting current pins removes policy disagreement but does not prove the broad dependency migration, offline closure, compiler compatibility or actual runtime resource identity. These are acceptance claims only after the corresponding real gate succeeds.
