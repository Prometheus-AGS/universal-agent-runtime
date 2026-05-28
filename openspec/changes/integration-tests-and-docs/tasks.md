## 1. Backend system tests

- [ ] 1.1 `tests/live_bus_latency.rs` — deferred to a focused test PR.
- [ ] 1.2 `tests/builtin_skill_delete_409.rs` — deferred.
- [ ] 1.3 `tests/kb_document_count.rs` — deferred.
- [ ] 1.4 `tests/binary_supervisor.rs` — covered by unit tests already in `process_supervisor.rs` (probe true/false). System-level tests deferred.

## 2. Frontend tests

- [ ] 2.1 Vitest + RTL multi-component SSE propagation test — deferred until consumer migrations (10/11) ship.
- [ ] 2.2 Storybook smoke for skill affordance — deferred.

## 3. Container CI

- [ ] 3.1 `docker build` step — deferred (Dockerfile shipped, CI integration is repo-wide ops work).
- [ ] 3.2 Toolchain smoke — deferred (Dockerfile already runs the smoke probe inline during build).

## 4. Docs

- [x] 4.1 [`docs/realtime.md`](../../../docs/realtime.md) — full server + client contract for the live-bus spine.
- [x] 4.2 [`docs/skill-authoring.md`](../../../docs/skill-authoring.md) — Manifest / Wasm / Native authoring guide + `origin` semantics.
- [ ] 4.3 README architecture diagram + clone-with-submodules — deferred.
- [ ] 4.4 `AGENTS.md` + `CLAUDE.md` updates — deferred.
- [ ] 4.5 `docs/frontend-realtime.md` (separate from server doc) — deferred; covered for now under §6 of `docs/realtime.md`.
