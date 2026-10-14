# Release pin reconciliation — operator decision artifact

Date: 2026-10-11. Phase: `dot-feature-support`, Execute. Change: `dot-01-release-provenance`, task 1 prerequisite only. Inspected root: `2cc9a1829167411e01a8198072f60892d70a233d`. Producer: native `gpt-6.1-sol`, high reasoning. This report does not approve a dependency migration or certify packaging.

## Decision requested

**Adopt the exact Liter 2.1.1 and skill-system gitlinks already selected by main as the release baseline, retaining the existing memory upstream commit, or restore the older policy gitlinks and reconcile their consuming lockfiles?**

Recommended: deliberately adopt the current main inputs for the subsequent production gate. Main, its path manifest, and Cargo.lock already agree on Liter 2.1.1. Restoring 1.18.2 introduces a downgrade into an active implementation baseline and requires fresh compatibility and lockfile reconciliation. Current main's published image evidence is useful baseline evidence, although it does not certify this change. This recommendation is conditional on operator acceptance of migration risk; it is not a claim that the broad Liter/skill-system migrations have been exhaustively reviewed.

The uncomfortable thing: the old-to-current Liter diff spans 4,555 files, including 70 files in the consumed `crates/liter-llm` tree; the skill-system diff spans 6,928 files, including 1,120 under `skills/`. Adopting main is a policy decision across substantive source changes, not a generated-record-only refresh. Full API/provider/skill compatibility and clean locked builds remain unverified.

## Exact policy versus source

| Input | Authoritative policy | Effective source at inspected root | Classification |
|---|---|---|---|
| Liter | `liter_llm = "1.18.2"`; commit `c5c6caac617eb931cd5009146a70831422ec236c` | Root `Cargo.toml:587` uses `vendor/git/liter-llm/crates/liter-llm`; gitlink and clean local HEAD `a6047386cc9fae4258b4a8577f6a016022095586`; crate manifest and Cargo.lock both `2.1.1` | Confirmed policy drift |
| Builtin skill resource | commit `ad5c82c6c16145637c589a3ddfa06e0f20d603e7` | Gitlink and clean local HEAD `f4bef4556a83ac22bf204bf69a01ddf0f9e007f2`; root excludes this subtree from the Cargo workspace; resource contents are copied/discovered | Confirmed resource-policy drift; runtime overrides are separate |
| Memory | upstream commit `432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8`, base `e5f48916048a8e26d3ace83c44746b8105f0696c` | Root `Cargo.toml:422` uses `vendor/git/surreal-memory-server/crates/surreal-memory`; Cargo.lock version `0.1.0`; root Git entry is ordinary **tree** `f9215a067a0e5dd3be0569ab687272c264748d5b` | Verified source export with explicit manifest adaptation; no demonstrated upstream-pin drift |

Both older gitlink commits are local Git objects and ancestors of the effective commits. Liter old commit is dated 2026-08-26 and effective commit 2026-10-02. Skill old commit is dated 2026-08-28 and effective commit 2026-10-10. Full diff counts are structural evidence, not an exhaustive behavioral audit; Git skipped exhaustive rename detection for the broad skill diff. The scoped skill count used `--no-renames` explicitly.

### Memory comparison: COMMIT is not TREE

`vendor/git/README.md` records the same policy upstream commit and describes four changed implementation files copied byte-for-byte with standalone manifest adaptations. The local upstream repository `/Users/gqadonis/Projects/prometheus/surreal-memory-server` contains the exact commit; comparison used `git show 432eaa1…:<path>`, not its newer working HEAD `1184156d9cf2fb576faf83284cb7339f42011ef5`.

All 20 root-tracked exported files were compared with their exact upstream paths. **19 are byte-identical: all 18 Rust source files and the crate README.** The sole difference is `crates/surreal-memory/Cargo.toml`. The complete file-level SHA-256 inventory is in the companion JSON. This is stronger than verifying only four changed files, but does not claim that deleted upstream files, tests, or workspace metadata were exported.

The manifest difference matches the documented export adaptation: explicit standalone dependency declarations replace workspace inheritance; `surrealdb` and `surrealdb-types` are exact `=3.3.0`; the optional `mempalace-core` dependency is omitted and `palace = []` retained. `hf-hub = "=0.5.0"` is present in both the export and that upstream commit, so it is not an additional manifest difference. The exported manifest SHA-256 is `7423a95a88e2a23dc12587d3864b79ae813238b34731858f95577409cabc42b3`; upstream manifest SHA-256 is `68b51b8143548c8c2660fd2778dedef1f598795bf1fd1a08bf8fe34b189c2736`.

Do not replace `surreal_memory_server_commit` with `f9215a…`: that value names a filtered root export tree, not an upstream commit. Keep the existing upstream policy identity and record the export tree and adaptation separately in release provenance.

## Runtime skill selection inspected on this machine

`src/uar/runtime/skills/pack_detection.rs` selects: `UAR_BUILTIN_SKILLS_DIR`, then `PROMETHEUS_SKILL_SYSTEM_DIR` or `../prometheus-skill-system` with a parseable plugin manifest, then installed-cache candidates, then embedded `crates/prometheus-skill-system/skills`.

For the inspected shell environment and repository cwd:

- `UAR_BUILTIN_SKILLS_DIR`, `PROMETHEUS_SKILL_SYSTEM_DIR`, `UAR_EXTRA_BUILTIN_SKILL_DIRS`, and `UAR_SKILLS_WASM_BUILTIN_DIR` are absent.
- `/Users/gqadonis/Projects/prometheus/prometheus-skill-system` does not exist.
- The runtime's actual installed-search roots are `/Users/gqadonis/.claude/plugins/cache`, `/Users/gqadonis/.claude-code/skills`, and `/Users/gqadonis/.config/uar/skills`. Only the first exists; none contains a matching `prometheus-skill-pack` candidate with `skills/` at the exact two-level scan depth used by this code. The Codex plugin cache is not a search root in this resolver.
- The repository `.env` exists but declares none of those four path keys. No unrelated environment values or secrets were emitted.
- Therefore source-evaluated primary selection is `embedded_submodule`, absolute path `/Users/gqadonis/Projects/prometheus/universal-agent-runtime/crates/prometheus-skill-system/skills`, at gitlink `f4bef4556a83ac22bf204bf69a01ddf0f9e007f2`. Its parent `.claude-plugin/plugin.json` does not exist, so the resolver's declared `version` is **null**, not inferred from package metadata.

This is filesystem/environment/source evidence, not an executed runtime receipt. `src/main.rs` and `src/bin/uar-sidecar.rs` call `dotenv`; an explicit selected environment file, launcher environment, alternate cwd, verified skill roots, or user skills can affect a real launch. `builtin_loader.rs` appends extra/verified roots; WASM uses its separate override. A release gate must record `(source, absolute root, declared version, immutable content identity)` from the actual launch and record extra roots separately. Do not assume the packaged gitlink proves every skill loaded at runtime.

## Landed fixes and current packaging inputs

The original operator report in `input-review.md` refers to root `3327bf99e48097479313c8a54e8e58d1de9e9ccb` and failed image run `38101260912`, checked 2026-10-11 13:47–13:48 UTC. Preserve it as dated evidence. It does not describe this later root unchanged.

At inspected main, commit `d4e819d1` updated both root and frontend pnpm lockfiles and restored the frontend A2UI/Zod 3 boundary. Commit `54e96c33` added workflow schema context and the explicit entity-graph-react build. The analysis records image run `38154340213` succeeding for both Linux architectures with index digest `sha256:e955789aa6aea70853d4e19ce95b5c5774750d232472c048bd4763c96e5513f5`; that is retained publication-log evidence, not a new registry/health recheck.

| Path | Actual inspected behavior | Action after decision |
|---|---|---|
| Root and `frontend/package.json` | `packageManager: pnpm@11.15.0`; frontend first-party `zod: 3.25.76` agrees with policy and lock importer | Preserve |
| `Dockerfile:206–212` | Frontend and entity-management installs use `pnpm install --frozen-lockfile`; builds core, React, then management before frontend | Preserve; no repeat of historical repair |
| `.dockerignore` | Retains draft-2 schemas and `!docs/agents/collaboration/workflow-execution/1.0.0/`; tracked extension schema exists, blob `8a7097502b86b9fdac3a829e036a8ebed465638d` | Preserve; source inspection is not a Docker-context execution test |
| `Dockerfile:226` | `CARGO_NET_GIT_FETCH_WITH_CLI=true cargo +"${RUST_TOOLCHAIN}" build --release --features "server-full,postgres-backend" --bin universal-agent-runtime` | Add `--locked` after policy decision; this is the observed gap |
| `.github/workflows/deploy.yml:211` | `cargo build --release --locked --no-default-features --features <platform features> --bin uar-sidecar --target <matrix target>` | Already locked; no workflow edit required for this gap |
| `scripts/certify-operational-resilience-local.sh:79–89` | Root/frontend frozen installs, frontend build, then `cargo build --locked --release --no-default-features --bin universal-agent-runtime --features server-full` | Existing callable local gate, not run in prerequisite |
| `scripts/package-offline-source.sh:24` | Tracks submodule files in export; `cargo vendor --quiet --locked vendor/crates` | Existing callable source-closure path, not run in prerequisite |
| `scripts/package-boss-sidecar.mjs` | Packages actual built target binary and emits file hashes, source and features; no compiler or test invocation | Record produced payload when future gate runs |

No dependency installation, Cargo invocation, container/native build, product suite, workflow edit, commit or push ran here. Root reported a passing pre-edit `pnpm github-actions-policy:validate`; this worker did not run or independently certify it. The validator remains mandatory before and after a future Docker/workflow-invoked script edit. Product verification remains local and serialized at the completed slice boundary.

## Deliberate policy patch candidate — not applied

An operator may apply this minimal candidate **by hand to `versions.toml`** after accepting the current baseline. The file's standing instruction says agents must not edit it. No automated patch to that file was made.

```diff
--- a/versions.toml
+++ b/versions.toml
@@
-# Liter 1.18.2 is the provider/model routing baseline. The accepted remote head
-# adds only generated knowledge records after the 1.18.2 source release.
-liter_llm = "1.18.2"
-liter_llm_commit = "c5c6caac617eb931cd5009146a70831422ec236c"
+# Deliberately adopt the exact Liter source already selected by UAR main
+# 2cc9a182; migration and locked production-path acceptance remain required.
+liter_llm = "2.1.1"
+liter_llm_commit = "a6047386cc9fae4258b4a8577f6a016022095586"
@@
-# The Skill System accepted head already contains the reviewed Liter and
-# Surreal Memory pointer refresh; its final UAR pin is recorded after the
-# exact-pin pointer commit is created.
+# Deliberately adopt UAR main 2cc9a182's exact skill-resource gitlink.
+# Effective runtime overrides and migration acceptance are recorded separately.
 prometheus_skill_system_base_commit = "f0df1e0121d176c9467e5c71022737b845af6a20"
-prometheus_skill_system_commit = "ad5c82c6c16145637c589a3ddfa06e0f20d603e7"
+prometheus_skill_system_commit = "f4bef4556a83ac22bf204bf69a01ddf0f9e007f2"
```

Keep both memory keys unchanged. The old base commit key is historical baseline context, not a claim the entire new skill diff has been accepted; revising that baseline meaning needs a separate deliberate policy choice.

Alternative: retain `versions.toml` exactly; restore Liter gitlink `c5c6caac617eb931cd5009146a70831422ec236c` and skill gitlink `ad5c82c6c16145637c589a3ddfa06e0f20d603e7`. Then deliberately reconcile `Cargo.lock` to Liter 1.18.2 and any transitive differences, regenerate affected source/resource provenance, and run fresh compatibility/packaging acceptance. Do not keep a 2.1.1 lock record with a restored 1.18.2 path manifest; do not blindly regenerate every lockfile. The older revisions are locally available but rollback compatibility has not been tested. No rollback was performed.

## Allowed next work and exit status

Without editing protected policy: inspect and author release evidence contracts, record immutable source identities, preserve dated reports, and prepare the future packaging patch. Parent may continue independent changes whose prerequisites allow it. Do not apply dependent source/lock/build changes or mark task 1 complete before the operator resolution is recorded. After resolution, the minimal known Docker edit is adding `--locked`; dependency alignment depends on the chosen option. Actual clean source closure, frozen installs, Docker context, locked builds and runtime skill receipts are the completed-change production gate, not prerequisite evidence.

Changed files: this Markdown decision artifact and its companion JSON only. Unrequested additions: none. Guards added: none. Remaining unverified: behavioral migration compatibility; fresh transitive dependency closure; container/native packaging; actual launched skill provenance; registry publication/health at a new revision. These require the future approved production gate. Tier 0 artifact JSON parsing and whitespace checks are recorded in the JSON; no broader tier is claimed.
