## Why

The integration phase touches eight distinct surfaces (KB counts, live bus, skills model, submodules, WASM runtime, pnpm migration, entity-mgmt, Docker). Each change ships its own unit/integration tests, but we need a small set of **system-level** tests that lock the cross-cutting contracts so regressions in one slice surface immediately.

## What Changes

### Backend / system tests

- `tests/live_bus_latency.rs` — write a row, assert SSE event within 200 ms p95 across 1000 iterations.
- `tests/builtin_skill_delete_409.rs` — register a fixture Builtin skill, assert DELETE returns 409.
- `tests/kb_document_count.rs` — assert `document_count` matches actual row count for N ∈ {0,1,5}.
- `tests/binary_supervisor.rs` — stub TCP server, assert no spawn when port is occupied.

### Frontend tests

- Vitest + RTL: synthetic `EventSource` mock; two components reading the same entity re-render on a single SSE event.
- Storybook smoke for the Skills page (Built-in vs User affordance).

### Container tests

- GH Action: `docker build` against the multi-stage Dockerfile; smoke `docker run … bash -c "rustc --version && node --version && pnpm --version && bun --version && python3 --version && uv --version && go version && wasmtime --version"`.
- Optional: spin the container up with surreal/skill-system mounted; curl `/health` and `/api/uar/skills`.

### Docs

- Update `README.md` architecture diagram.
- Update `AGENTS.md`: clone with `--recurse-submodules`, pnpm-not-bun, builtin skills location.
- Update `CLAUDE.md`: realtime contract (SSE topics + entity-mgmt graph keys), skill model (`kind`, `origin`), supervisor model.
- New `docs/realtime.md`, `docs/frontend-realtime.md`, `docs/skill-authoring.md` (Manifest, Wasm, Native).

## Acceptance

- All new tests pass in CI.
- Docs reference the canonical paths (e.g. `~/.uar/`, `/opt/uar/`, `/var/lib/uar/`).
- An external contributor can clone, init submodules, build, and run end-to-end following only README.
