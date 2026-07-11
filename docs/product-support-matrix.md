# Product Support Matrix

This is the release contract skeleton. Until the referenced certification change passes, rows remain Preview or Experimental regardless of whether code is present.

## First-party interfaces

| Interface | Status | Contract |
|---|---|---|
| React web application | Preview | Primary interface; certification in the active production-hardening phase |
| React administrative console | Preview | Same application and architecture contract |
| Tauri desktop shell | Preview | React frontend plus local sidecar/runtime; platform artifacts pending |
| Mobile | Experimental | No GA packaging/platform certification yet |
| HTMX/Web Components application | Not a primary product interface | Historical/research material only unless separately scoped |

## Protocols

| Protocol | Status | Release condition |
|---|---|---|
| MCP client/server | Preview | Stable transport/health/tool certification and support statement |
| AG-UI | Preview | Versioned mapping and golden/live conformance fixtures |
| A2UI v0.9.1 | Planned GA profile | Validated shared React renderer and round-trip certification |
| A2UI v1.0 candidate | Experimental | No GA promise while upstream status is candidate |
| A2A | Preview | Declared transport/profile and integration evidence |

## Persistence

| Deployment | Runtime store | Client store | Current status |
|---|---|---|---|
| Web/server default | Embedded SurrealDB | PGlite thread/message cache | Preview; authority/conflict contract pending |
| Server with Postgres | PostgreSQL/pgvector where configured | PGlite thread/message cache | Preview; feature matrix pending |
| Desktop | Embedded runtime store | PGlite in webview | Preview; platform recovery tests pending |
| Mobile | Undeclared | Local client storage | Experimental |

## Providers and models

The embedded catalog describes breadth; it is not a certification list. Provider Tier 1/2/3 capability evidence and last-verified dates will be published by `publish-capability-support-matrix`. Until then, the unqualified provider count is a catalog statement only.

## Security defaults

| Control | Current documented status | GA evidence needed |
|---|---|---|
| JWT authentication | Required by default | startup/API tests |
| Rate limiting | Enabled by default | enforcement tests |
| Prompt-injection screening | Detect-only by default | detection/block-mode tests |
| PII/secret screening | Detect-only by default | detection/block-mode tests |
| Cedar default policy | Permit-all baseline unless configured | exact policy/default documentation |
| Tool approval | Risk/policy driven | `Allow`, `RequireApproval`, hard `Deny` certification |
| Secret log redaction | Enabled | regression tests across config/error paths |

## Release bundles

The stable Cargo bundle matrix is not yet certified. `modularize-release-capabilities` will define and test `minimal`, `server-full`, and `desktop-full`; current feature presence does not imply GA support.
