# Verification — `rewrite-readme-and-docs`

Date: 2026-08-18

Scope: customer Markdown and Docusaurus 3.10.2 on Node.js 26.5.0, plus UAR
`server-full` OpenAPI code on macOS. Results transfer to no other profile,
platform, site generator, or deployment.

| requirement | assertion observed | negative control observed | command | result |
|---|---|---|---|---|
| Customer documentation describes the whole product accurately. | README and site now cover architecture, Flint service boundaries, SDKs, skills, deployment, and security; Mermaid support is configured; the documentation truth gate and production site build pass. | A normal temporary docs page linking to a nonexistent target made Docusaurus exit 1 with `Docusaurus found broken links`; removing it restored exit 0. | Exact commands and output are in `evidence/positive-verification.md` and `evidence/negative-controls.md`. | Documentation checks pass for the stated Node/Docusaurus scope. |
| Published API metadata matches the runtime. | OpenAPI version equals `CARGO_PKG_VERSION`; representative chat, runs, providers, skills (including both `/reload` and `/refresh`), knowledge, auth, and realtime paths exist. | The focused test asserts both distinct mounted skills refresh paths and their customer route peers. | `RUSTC_WRAPPER= cargo test --locked -p universal-agent-runtime --no-default-features --features server-full spec_uses_package_version_and_documents_customer_routes --lib` | Exit 0; 1 passed, 0 failed. |
| Repository root contains no captured test output. | All five reviewed scratch artifacts are absent by exact path. | The exact-path guard exits nonzero if any target exists. | Command and output are in `evidence/positive-verification.md`. | Exit 0; no target exists. |
| Change-level gates pass. | Tier 0, docs-site checks, strict OpenSpec, scoped diff checking, and artifact-refiner review passed. | The critic first blocked three product-truth defects and the judge blocked missing checkpoint provenance; both independently passed after the exact corrections. | Commands and actual output are in `evidence/artifact-refiner-validation.md`. | Exit 0; artifact-refiner converged with 4/4 constraints and 9/9 checkpoint references present. |
