# UAR Compiler Agent

The UAR Compiler Agent transforms **UAR-AGENT-MD** Markdown specification documents into signed, runnable agent descriptors through an 8-stage PMPO pipeline.

## Overview

```
UAR-AGENT-MD document
        │
        ▼
  ┌─────────────┐
  │   Parser    │  pulldown-cmark → AgentDescriptorIR
  └──────┬──────┘
         │
         ▼
  ┌─────────────────────────────────────────────────────────┐
  │                   8-Stage Pipeline                      │
  │  01 Frontmatter → 02 UI → 03 MCP → 04 A2A Schemas      │
  │  05 Cedar       → 06 Actors → 07 PEP → 08 Emit         │
  └──────┬──────────────────────────────────────────────────┘
         │
         ▼
  Signed AgentDescriptor (JSON) + CompileReport
```

## Operating Modes

### Single-Shot (Batch)

Submit a complete UAR-AGENT-MD document and receive a compiled descriptor in one call.

**Native Skill:** `uar.compile`

```json
{
  "skill": "uar.compile",
  "input": {
    "content": "---\nname: my-agent\n...\n## Identity\n..."
  }
}
```

### Conversational (Multi-Turn)

Build a spec incrementally through a multi-turn session. The compiler tracks completeness and compiles automatically when all required sections are present.

**Session Tools:**
- `uar.session.update_section` — add or update a section
- `uar.session.check_completeness` — check which sections are missing
- `uar.session.compile` — trigger compilation when ready

## REST API

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/uar/specs` | Submit a spec for compilation |
| `GET`  | `/api/uar/specs` | List all specs |
| `GET`  | `/api/uar/specs/:id` | Get a spec |
| `GET`  | `/api/uar/specs/:id/reports` | Get compilation reports |
| `POST` | `/api/uar/sessions` | Create a conversational session |
| `GET`  | `/api/uar/sessions` | List sessions |
| `GET`  | `/api/uar/sessions/:id` | Get session status |
| `DELETE` | `/api/uar/sessions/:id` | Cancel a session |
| `POST` | `/api/uar/sessions/:id/compile` | Compile a session |

All endpoints require an `Authorization: Bearer <token>` header. See [API_KEYS.md](./API_KEYS.md).

## Pipeline Stages

| Stage | File | Purpose |
|-------|------|---------|
| 01 | `s01_frontmatter.rs` | Validates YAML frontmatter metadata and identity |
| 02 | `s02_a2ui.rs` | Validates UI component IDs, registers schemas |
| 03 | `s03_mcp.rs` | Validates MCP server configs and tool references |
| 04 | `s04_a2a_schemas.rs` | Validates A2A message schemas |
| 05 | `s05_cedar.rs` | Compiles Cedar policy statements |
| 06 | `s06_actor_endpoints.rs` | Registers A2A actor endpoints |
| 07 | `s07_pep.rs` | Derives Policy Enforcement Point bindings |
| 08 | `s08_emit.rs` | Emits signed `AgentDescriptor` JSON |

## Signing

Each compiled descriptor is signed with **Ed25519** using the `KeyProvider` trait:

```rust
pub trait KeyProvider: Send + Sync {
    fn signing_key(&self) -> &SigningKey;
    fn public_key_bytes(&self) -> [u8; 32];
}
```

The default `LocalKeyProvider` generates a fresh key at startup. For production, implement `KeyProvider` backed by a KMS.

## Persistence

Specs and reports are stored via the `SpecStorage` trait. Sessions are stored via `SessionStorage`. Both are backed by **SurrealDB** when configured, falling back to in-memory for development.

```toml
[persistence]
provider = "surreal"
database_url = "rocksdb://./data/uar.db"
```
