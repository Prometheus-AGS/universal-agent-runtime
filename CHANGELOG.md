# Changelog

All notable changes to the Universal Agent Runtime (UAR) are documented here.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Stability statement

`1.0.0` marks the first stable release. From this point:

- The public HTTP surface (`/api/chat/completion`, the OpenAI-compatible
  `/v1/chat/completions`, and the `/api/uar/*` runtime endpoints), the
  configuration surface (the `UAR_*__*` env convention and precedence), and the
  agent-descriptor/compiler contract follow semantic versioning — breaking
  changes require a major-version bump.
- Security fixes are backported to the `1.x` line per
  [SECURITY.md](SECURITY.md).
- Internal crate APIs are **not** covered by this guarantee and may change in
  minor releases.

## [Unreleased]

## [1.0.0] — 2026-07-11

First stable, externally-consumable release.

### Added
- **RAG / knowledge-base retrieval** end-to-end with real BGE-small embeddings
  (fastembed) — ingest → vector search → ranked retrieval, plus "chat with your
  documents".
- **Runtime Console** (`/admin`): all operational panels wired to live data —
  runs, steps, tool calls, approvals, provider health, AG-UI events, memory
  activity, artifacts, model routing, and A2UI surfaces.
- **Published documentation site** (Docusaurus → GitHub Pages): installation,
  full configuration reference, backup/restore runbook, upgrade guide,
  troubleshooting, and API reference.
- **Security & community policy**: manufacturer-posture `SECURITY.md`
  (GitHub private vulnerability reporting, CRA-aligned), `SUPPORT.md`, issue
  templates, and plain-language dual-license clarity.
- **Config surface hardening**: `--port`/`PORT` and `--jwt-required`/
  `JWT_REQUIRED` CLI flags now apply (previously silently dropped).

### Changed
- Config precedence documented and enforced: CLI args → `UAR_*__*` env →
  legacy env → config file → compiled defaults.
- CI: the BDD chat suite and the recorded-backend live-integration tier are now
  blocking gates (were advisory).

### Security
- Remediated the standing stale RUSTSEC advisories (quinn-proto, kreuzberg
  dependency tree, microsandbox removal); zero open Dependabot alerts.
- Added a Dependabot-alerts CI gate that fails on any undisclosed open alert.

[Unreleased]: https://github.com/Prometheus-AGS/universal-agent-runtime/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/Prometheus-AGS/universal-agent-runtime/releases/tag/v1.0.0
