## Why

Changes 9–11 (`sdk-rust-1.0`, `sdk-python-1.0`, `sdk-typescript-1.0`) each
shipped a 1.0 SDK with 6 runnable examples and doc tooling scaffolding.
Order 6's Change 12 (`sdk-examples-cookbook-rustdoc`) is the follow-on
"cookbook" pass: the plan's done-condition names "12 runnable
`cargo run --example`s spanning the runtime," hosted rustdoc / typedoc /
sphinx, and a `tools/validate-examples.sh` CI smoke test. Without this
change, the SDKs have examples and doc *source*, but nothing verifies the
examples still compile against the SDK surface as it evolves, and the
Rust cookbook itself is thinner than the plan's "12 examples" target.

## What Changes

- **Audited first** (see "Plan correction" below): Changes 9–11 already
  delivered 6 examples per SDK (18 total across Rust/Python/TypeScript)
  and doc tooling (`cargo doc` via rustdoc comments, `sdks/typescript`'s
  `typedoc.json` + `docs` script + `typescript-sdk-docs.yml` GitHub Pages
  workflow, `sdks/python/docs/` Sphinx + Furo config). This change does
  **not** redo that work.
- **`sdks/rust/examples/`**: added 6 new examples to bring the Rust
  cookbook to the plan's literal "12 runnable `cargo run --example`s"
  target — `embeddings.rs`, `knowledge_base_crud.rs`,
  `document_ingest.rs`, `run_checkpoints_resume.rs`,
  `list_knowledge_bases.rs`, and `error_handling.rs`. Together with the
  existing 6, every public `ClientApi` surface (`chat`, `runs`,
  `knowledge`, `ingest`, `tools`, `embeddings`) now has at least one
  example. `error_handling.rs` is deliberately self-contained (targets an
  unreachable port, asserts on the `miette::Diagnostic` it gets back) so
  it can actually execute — not just compile — without a live server.
- **`tools/validate-examples.sh`**: new script that builds/typechecks
  every example across all three SDKs (Rust via `cargo build --examples
  --locked`, Python via `py_compile`, TypeScript via the existing
  `tsconfig.examples.json`-driven `npm run typecheck`), runs the one
  self-contained Rust example end-to-end, and supports an opt-in
  `VALIDATE_EXAMPLES_LIVE=1` mode to execute every example against a real
  running UAR server (nightly/staging use, not CI-on-every-PR).
- **`.github/workflows/ci.yml`**: new `sdk-examples` job that installs
  the Rust/Python/Node toolchains for all three SDK directories and runs
  `tools/validate-examples.sh` on every push/PR — the first CI coverage
  for `sdks/rust` and `sdks/python` at all (previously only
  `sdks/typescript` had any CI wiring, via `typescript-sdk-docs.yml`).
- **Verified, not re-built**: `cargo doc --no-deps -p
  universal-agent-runtime-sdk` (zero warnings), `sphinx-build -b html`
  for `sdks/python/docs` (build succeeded), and `npm run docs` /
  `typedoc` for `sdks/typescript` (0 errors) — see the change's task list
  for exact commands.
- **`sdks/rust/Cargo.lock`**: regenerated. `cargo build --locked` on
  `sdks/rust` was already broken on `main` before this change (verified
  via `git stash` against a clean checkout) — the `embedded`-feature path
  dependency on the root `universal-agent-runtime` crate pulls that
  crate's full dependency graph into the SDK's independent lockfile, and
  root-crate dependabot bumps merged after the SDK lock was last
  generated (e.g. `aes-gcm` 0.10.3 → 0.11.0) left it unable to satisfy
  `--locked`. Regenerating was necessary for the new `sdk-examples` CI
  job (which uses `--locked`, matching this repo's existing Rust CI
  convention) to pass at all; it is a lockfile-only change with no
  `Cargo.toml` edits.

## Plan correction: scope was 6 examples richer than assumed, doc tooling already exists

The plan's Change 12 done-condition reads as if examples and doc tooling
were still to be built from scratch. An audit of `sdks/{rust,python,
typescript}/` at the start of this change found:

- **Already done (Changes 9–11):** 6 examples per SDK (18 total), rustdoc
  comments across `sdks/rust/src/lib.rs` and friends, a working
  `sdks/typescript/typedoc.json` + `docs/api` output + GitHub Pages
  publish workflow, and a working `sdks/python/docs/conf.py` (Sphinx +
  Furo) that builds clean HTML today.
- **Genuine gap #1 — Rust example count:** the plan's literal "12
  runnable `cargo run --example`s" only makes sense as a Rust-specific
  count (`cargo run --example` is Rust-specific tooling), and Rust only
  had 6. This change adds the other 6, closing that gap for real
  coverage (not padding — each new example exercises a previously
  uncovered `ClientApi` method group: embeddings, full KB CRUD, generic
  ingest, checkpoint/resume, KB listing, and the typed error model).
- **Genuine gap #2 — no CI wiring:** neither `sdks/rust` nor
  `sdks/python` had *any* CI job before this change (grep against
  `.github/workflows/*.yml` for `sdks/rust` / `sdks/python` before this
  change returns nothing). `tools/validate-examples.sh` plus the new
  `sdk-examples` CI job close that gap.
- **Not done here — deferred, out of scope:** a `cargo publish` /
  release-plz workflow that would push `sdks/rust` to crates.io (which
  is what actually triggers a docs.rs build — docs.rs itself has no
  local-verification path and only builds after a crates.io publish).
  `release-plz.toml` exists at the repo root but no
  `.github/workflows/release-plz.yml` (or equivalent `cargo publish`
  step) exists yet for `sdks/rust`, `sdks/python` (PyPI), or
  `sdks/typescript` (npm — note `sdks/typescript` publishing to npm is
  separate from its already-working GitHub Pages typedoc publish). Wiring
  actual package-registry publishing is release infrastructure that
  belongs with the SLSA/provenance work in Order 5, not this
  documentation/examples change, and this change's own instructions are
  explicit that "docs.rs publish itself is a release-time, not
  implementation-time, event." What *is* implementation-time and is
  verified here: `cargo doc --no-deps` produces clean rustdoc output
  locally, which is the actual signal that a future docs.rs build (once
  publishing exists) will succeed.
- **Also observed, not fixed here:** `.github/workflows/deploy-docs.yml`
  (Docusaurus `website/`) and `.github/workflows/typescript-sdk-docs.yml`
  (typedoc) both deploy to the same `environment: github-pages`, which is
  a single Pages site per repo — this predates this change (introduced
  by Change 11) and is a latent deployment conflict, not something this
  change touches. Flagged separately rather than silently expanded into
  this change's scope.

## Impact

- Affected capability: `sdk-cookbook` (new).
- Affected files: `sdks/rust/examples/*.rs` (6 new), `tools/validate-
  examples.sh` (new), `.github/workflows/ci.yml` (new `sdk-examples`
  job). No SDK public API changes; no breaking changes.
