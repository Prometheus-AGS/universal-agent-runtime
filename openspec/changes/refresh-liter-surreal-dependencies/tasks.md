## 1. Workflow and Pin Baseline

- [x] 1.1 Validate the OpenSpec proposal, design, and deltas with `openspec validate refresh-liter-surreal-dependencies --strict` before dependency edits
- [x] 1.2 Record the approved Liter, Surreal Memory, SurrealDB SDK/server, and OCI image pins with rationale in `versions.toml`, then verify the file contains no floating dependency value
- [x] 1.3 Re-resolve the three upstream remotes immediately before use, review drift beyond the planning heads, and record the accepted commit set

## 2. Leaf and Nested Repositories

- [x] 2.1 Fast-forward Surreal Memory, pin `surrealdb` and `surrealdb-types` to `=3.2.4`, regenerate its lockfile, and pass focused and full checks
- [x] 2.2 Commit and push the Surreal Memory exact-pin change without force, then verify the commit is reachable from remote `main`
- [x] 2.3 Advance `prometheus-skill-system` Liter and Surreal Memory tool gitlinks to the verified commits and pass both target builds
- [x] 2.4 Commit and push the nested pointer update without force, then verify the parent and both leaf commits are remotely reachable

## 3. UAR Runtime Inputs

- [x] 3.1 Advance the UAR Liter gitlink, regenerate the provider catalog twice, and verify identical output digests
- [x] 3.2 Refresh the curated Surreal Memory manifest and four changed implementation files, specify the adopted durable-runtime behavior, preserve standalone adaptations, and pass the leaf runtime plus snapshot-focused checks
- [x] 3.3 Update vendor provenance and dependency documentation, advance the `prometheus-skill-system` gitlink, and verify recursive submodule reachability
- [x] 3.4 Regenerate `Cargo.lock` and prove Liter resolves to 1.18.2 and every SurrealDB crate resolves to 3.2.4
- [x] 3.5 Preserve Liter's owned response stream through UAR normalization so post-tool completions are not buffered behind the stream-start timeout, then pass the focused incremental-stream regression

## 4. Security and Deployment Inputs

- [x] 4.1 Pin and regenerate the authoritative root and frontend pnpm lockfiles and website npm lockfile, then prove vulnerable `nanoid`, `dompurify`, and `js-yaml` versions are absent
- [x] 4.2 Add website Dependabot coverage plus local website audit and affected-image-format enforcement, then verify the security receipt is clean and the gate rejects a disposable affected-format fixture
- [x] 4.3 Pin every Compose, Kustomize, Helm, and OpenTofu SurrealDB image to the exact 3.2.4 tag and OCI digest, then assert all rendered outputs contain only that reference
- [x] 4.4 Document the official v2-to-v3 export/import procedure and complete a disposable representative-data rehearsal without changing production state

## 5. Local Verification and Packaging

- [x] 5.1 Complete frozen installs, zero-unaccepted-finding audits, dependency graph assertions, and required Rust Tier 0 and focused Tier 1 tests
- [x] 5.2 Complete Rust Tier 2, frontend typecheck/lint/tests/build, and Docusaurus typecheck/lint/security/build with observed zero-failure output
- [x] 5.3 Complete Compose, Kustomize, Helm, and OpenTofu render/validate/plan checks and prove exact SurrealDB image references
- [x] 5.4 Package the offline source archive and complete a locked offline build from its extracted contents
- [x] 5.5 Complete the Tier 3 release builds for Liter, Surreal Memory, and UAR and record source artifact hashes and signing state
- [x] 5.6 Restrict offline packaging to tracked repository and recursive-submodule inputs so ignored operator credentials and private deployment variables cannot enter the archive

## 6. Local Deployment

- [x] 6.1 Capture rollback copies, hashes, executable identities, and current loaded state for UAR, Surreal Memory, and SurrealDB before replacement
- [x] 6.2 Install Liter and both Surreal Memory binary copies, verify installed hash equality and signing, and prove Liter CLI/MCP initialization
- [ ] 6.3 Install UAR through the macOS upgrade path and verify source/installed hashes and signing
- [ ] 6.4 Restart SurrealDB, Surreal Memory, and UAR in dependency order and capture SurrealDB authenticated query, Surreal Memory write/restart/read, UAR health/API/static asset, listener, PID, executable, and clean-log evidence

## 7. Commit, Advisory Closure, and Archive

- [ ] 7.1 Append decisions, commands, hashes, deployment evidence, alert dispositions, and the session summary to tracked `.prometheus` history
- [ ] 7.2 Obtain an artifact-only adversarial review of the implementation and correct any blocking finding before commit
- [ ] 7.3 Commit the verified UAR implementation, deploy from that exact commit, and push `main` without force
- [ ] 7.4 Poll the eight patched Dependabot alert IDs, dismiss only #210 and #211 with the approved bounded rationale, and verify the authoritative open-alert response is empty
- [ ] 7.5 Mark tasks complete, archive the OpenSpec change, validate canonical specs, commit and push the archive/evidence update
- [ ] 7.6 Verify root `main` equals `origin/main`, recursive submodules are clean and reachable, only the root worktree and root local `main` branch remain, and no task-created temporary resource remains
