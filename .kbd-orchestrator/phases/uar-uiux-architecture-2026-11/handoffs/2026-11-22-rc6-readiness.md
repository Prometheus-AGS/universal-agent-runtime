# 1.4.0-rc.6 readiness (2026-11-22, 04:45 CST): NOT READY

Assessment only, made by reading source, GitHub runs and worktrees. Nothing was built or published.

## Blockers

1. **S1 round 3**: being written, uncommitted, unverified. Last boundary (c8eb9e23): 1 PASS, 2 FAIL, 10 BLOCKED.
2. **#539**: written in `~/.claude/worktrees/uar-539-supervised-launch` (0ce227af plus uncommitted work).
   No commit, PR or valid test run. The only run was 11 passed, 29 failed, before later fixes. It was
   handed to Codex by the operator; no edit since 04:13. Tasks 4.1-4.3 are open.
3. **amd64 image build broken on main since #528 (0ce227af)**: `ERR_PNPM_OUTDATED_LOCKFILE`,
   `frontend/vendor/a2ui-web-core/package.json` wants `@preact/signals-core ^1.14.5`, lockfile has ^1.14.2
   (run 44610600467). cb19619e built on amd64.
4. **arm64 image build broken since at least c7e63291 (2026-11-17)**: unlocked
   `cargo install cargo-wasi 0.1.28` in `scripts/docker/install-wasm-tools.sh` resolves a `filetime` that
   does not compile on nightly-2026-07-18 (`fs_set_times`). The manifest job needs both architectures,
   so no `main` image has been published since 2026-11-14 (#505). A `v*` tag today would publish no tagged image.
5. **Integration**: the six S1 commits, round 3, #539 and main have never been built together. Scratch
   three-way merge of the overlapping files: `src/config.rs`, `src/server.rs` and `stub_llm.rs` clean;
   `.prometheus/decisions.md` conflicts (append-only log, keep both). Verification of either branch alone
   does not certify the combination, and the Repair rule invalidates S1 evidence when `src/**` changes.

## Release work after the fixes

- Version bump: `Cargo.toml`, lockfiles, `sdks/rust/Cargo.toml`, README, `website/docs/intro.md`, release notes.
- rc.3 to rc.5 were excluded from the CI tag build and published as locally qualified Linux amd64 images
  with evidence (rc.5: native chat, AG-UI/A2UI, security audit on the exact digest, signed tag). rc.6
  needs the same, with the tag exclusion in `deploy.yml` (three places), or an explicit decision otherwise.
- The Boss sidecars (win32-x64, darwin-arm64, darwin-x64) are separate tags and builds. #539 moves the
  sidecar source, so all three need rebuilding.
- `GET /api/uar/version` reports `git_commit: null` in containers until the Docker build passes `UAR_GIT_SHA`.
- Push, PR, merge, tag and publication each need operator authorization.

## Not included unless the operator adds them

- #539's optional "every tool off" mode (operator chose a follow-up issue) and a new capabilities
  endpoint (operator decided against; `/api/uar/capabilities` exists and needs credentials).
- Open issues #524, #520, #519, #388; seven Dependabot PRs (#532-#538).
- Two dirty worktrees, `uar-rc5-surreal-401-pr13` (32 files) and `uar-131-surreal-401-pr13` (36 files):
  uncommitted SurrealDB 4.0.1 work that differs from main. Intent unknown; do not delete.
- The rest of the UI/UX phase (about 305 tasks).

## Recommended order (awaiting the operator)

1. Finish and review round 3; commit locally. Do not start its 4.5-6 hour boundary yet.
2. Take #539 back from Codex, run its verification batch on its branch, commit locally.
3. Fix both image builds and the Docker commit stamp.
4. Build one local candidate: main + S1 + #539 + those fixes + the rc.6 version. Amend the S1 contract
   for that candidate (owner-signed), get the critic's confirmation, run the boundary once.
5. Qualify the exact amd64 image as for rc.5, then ask for push, PR, merge, tag and publication.

The alternative is to run the S1 boundary now and integrate later, which costs a second boundary run.
