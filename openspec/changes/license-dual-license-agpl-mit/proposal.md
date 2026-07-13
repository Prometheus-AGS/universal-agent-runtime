## Why

UAR the runtime is currently AGPL-3.0-only (`Cargo.toml:
license = "AGPL-3.0-only"`) and the SDKs inherit that license. The
operator's 2026-07-13 release-readiness assessment flagged this as
**the single largest commercial-adoption blocker**: every named
2026 competitor (LangGraph, LangChain, CrewAI, Microsoft Agent
Framework, OpenAI Agents SDK, LlamaIndex, Haystack, agentgateway,
Markus) ships SDKs under a permissive license. The 2-of-2 AGPL-3.0
OSS agent runtimes (UAR, Markus) both explicitly dual-license; UAR
currently does not.

## What Changes

- **Runtime server** stays **AGPL-3.0 + commercial** (preserves the
  copyleft moat; SaaS deployers get an explicit exit).
- **SDKs (Python, Rust, TypeScript)** become **MIT** (matches 8 of
  8 named competitor SDKs; maximum enterprise adoption).
- **Documentation** licensed under **CC-BY-4.0**.
- **Trademark policy** already in `TRADEMARKS.md` — cross-linked.
- Open-letter process for the SDK contributors (the runtime keeps
  AGPL-3.0 for past contributions; only the SDK relicensing requires
  consent).
- `LICENSE-COMMERCIAL.md` rewritten with named pricing bands and a
  contact path.
- `CONTRIBUTING.md` updated with the CLA-lite forward-going clause
  ("by contributing, you agree to dual-license under AGPL-3.0 + the
  commercial terms").

## Capabilities

### New Capabilities

- `dual-license-policy`: the explicit dual-licensing framework for
  the UAR runtime + SDKs + documentation.

## Impact

- **Legal:** every past SDK contributor must consent to MIT
  relicensing, or their contributions are removed from the SDK
  source. Estimated SDK source size: ~15 files total; ~10 known
  contributors.
- **Operational:** the `CONTRIBUTING.md` change affects every
  future PR. The `LICENSE-COMMERCIAL.md` rewrite requires the
  operator to publish commercial pricing.
- **Marketing:** README + docs site license section must be
  updated; product-support-matrix.json `license` field per bundle
  must be updated.
- **CI:** a new `tools/license-check.sh` validates the license
  files are present and match the declared licenses in
  `Cargo.toml` / `pyproject.toml` / `package.json`.

## Out of scope

- Relicensing the **runtime server** to MIT. Out of scope: the
  AGPL-3.0 + commercial dual-license is the right answer for the
  runtime. Changing to MIT is a separate, larger decision.
- Adding a **CLA bot** (Contributor License Agreement signing flow).
  Out of scope: the open-letter + CLA-lite forward-going clause
  is the right answer for the SDKs. Adding a CLA bot is a
  6-month project of its own and tracked as a post-GA hardening
  task.
- **Renaming the project.** Not part of this change.
