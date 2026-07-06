# Assessment — `prometheus-package-integration`

**Date:** 2026-05-26
**Tool:** claude-code (`/kbd-assess`)
**Prior phase:** `runtime-provider-protocol-hardening` (still in `assessment_ready`; this phase is being inserted ahead of it per direct user request — confirm sequencing before planning execution)
**Project source of truth:** `.kbd-orchestrator/`
**Memory mirror:** `surreal_memory` MCP at `/mcp/memory` (non-authoritative)

---

## 1. Phase goals

Three coupled goals, all stemming from the user's session on 2026-05-26:

1. **Fix Knowledge Bases doc-count display** — Admin → Knowledge shows `0 documents` for the `default` KB even though the DB has an indexed document. The bug is a missing field in the backend response, not a UI bug.
2. **Embed `prometheus-entity-management` as the canonical React state layer** — add the repo as a git submodule under `frontend/packages/prometheus-entity-management/`, wire `configureEngine`, and migrate the SPA's entity-flavored UI (knowledge bases, documents, agents, providers, threads, settings) to consume its `useEntity` / `useEntityList` / `useEntityCRUD` hooks instead of bespoke fetchers + ad-hoc stores.
3. **Embed `prometheus-skill-system` as built-in, non-deletable UAR skills** — add the repo as a submodule under the Rust workspace, discover every `SKILL.md` it contains at startup, register them in `SkillService` as system-owned (un-deletable) skills, and make the production container ship them plus a writable volume for derivative artifacts.

---

## 2. Current state (what exists today)

### 2.1 Repo / workspace topology

- UAR is a Cargo workspace already (`Cargo.toml` lines 1-3) with two members: `.` (the runtime crate) and `tools/uar-jwt-proxy`.
- Frontend lives at `frontend/` and is built by `build.rs` via `bun run build`. Output is copied to `static/` and (for installed binaries) `~/.uar/static/`.
- Frontend is **not** a pnpm/yarn workspace today — single `package.json` at `frontend/package.json`. The user explicitly wants entity-management to live in a `packages/` subdir, which implies introducing workspaces.
- The two target sibling repos are already present at:
  - `/Users/gqadonis/Projects/prometheus/prometheus-entity-management/`
  - `/Users/gqadonis/Projects/prometheus/prometheus-skill-system/`
  Both are git repos, suitable for use as submodules with `git@github.com:Prometheus-AGS/…` remotes.

### 2.2 Knowledge-base doc count

- API contract: `KnowledgeBaseResponse` at `src/uar/api/knowledge.rs:62-72` does **not** include any `document_count` / `documents` field.
- Persistence: SurrealDB stores `knowledge_documents` with `kb_id` foreign-style references; no `documents` array embedded in the KB row.
- Frontend: `frontend/src/admin/pages/knowledge-page.tsx:303` renders `{kb.document_count ?? 0} documents`. The same field is referenced in `entities/types.ts:111` (`document_count: number;`) and `admin/components/agent-editor.tsx:291,592-594`.
- Result: every KB list view shows `0 documents` regardless of actual count. The backend has no API today that returns this aggregate.
- Note: the per-document chunk-count *update* path was also previously broken (see `src/uar/persistence/providers/surreal.rs:495` — `WHERE id = $id` with bare-UUID bind didn't match SurrealDB RecordIds; fixed earlier this session by switching to `type::thing(...)`). The doc-count bug at the KB list level is a separate, independent issue.

### 2.3 Existing skill subsystem in UAR

- `src/uar/runtime/skills/service.rs` already implements `SkillService` with two storage providers wired at startup: `Local Skills` (filesystem at `./skills`) and `Database Skills` (SurrealDB).
- Logs from the running instance: `FilesystemStorageProvider 'Local Skills': loaded 0 skills from "skills"` — the filesystem source is hooked up but finds nothing because the `skills/` dir is empty.
- Skill model in UAR (`src/uar/runtime/skills/`): WASM-skill-flavored (the sidebar even labels Skills as "WASM skill registry"), with matching algorithms `Keyword | Embedding | Llm | Hybrid | LocalEmbedding` (`src/uar/runtime/manager.rs:476`). Skills currently have `delete` permissions tied to the storage provider; nothing today distinguishes "system-shipped" vs "user-defined" skills.

### 2.4 Existing prometheus-entity-management (`/Users/gqadonis/Projects/prometheus/prometheus-entity-management`)

- Single tsup-built package: `@prometheus-ags/prometheus-entity-management`. Not a monorepo (one `package.json`).
- Public surface: graph store + query hooks built on Zustand 5 + Immer 11:
  - **Engine:** `configureEngine(EngineOptions)`, `fetchEntity`, `fetchList`, `startGarbageCollector`, `serializeKey`.
  - **Hooks:** `useEntity<TRaw, TEntity>`, `useEntityList`, `useEntityView`, `useEntityCRUD`.
  - **Schemas:** `registerEntityJsonSchema`, `buildEntityFieldsFromSchema`, `useSchemaEntityFields`.
  - **Local-first runtime:** `startLocalFirstGraph`, `hydrateGraphFromStorage`, `persistGraphToStorage`, `useGraphSyncStatus`.
  - **Adapters:** GraphQL, Prisma, realtime (Supabase / PGLite / ElectricSQL / Convex).
  - **UI primitives:** `EntityTable`, `EntityDetailSheet`, `EntityFormSheet`.
- Peer deps: `react@>=18`, `react-dom@>=18`, optional `@tanstack/react-table@>=8`. Runtime deps: `zustand@>=5`, `immer@>=11`, `clsx@>=2`, `tailwind-merge@>=3`, `lucide-react@>=1` — all already in the SPA except possibly immer 11. The host must supply fetchers + normalizers; the library is transport-agnostic.

### 2.5 Existing prometheus-skill-system (`/Users/gqadonis/Projects/prometheus/prometheus-skill-system`)

- **Not** a single Cargo workspace. It's a heterogeneous repo containing:
  - Markdown/YAML skill manifests under `skills/<domain>/<name>/SKILL.md` (this is the canonical skill format — YAML frontmatter + Markdown body, **not** WASM).
  - Tooling crates at `tools/prometheus-cli/` (workspace with `prometheus`, `prometheus-agents`, `prometheus-learn`, `prometheus-cedar`) and `tools/forge-rs/` (`forge` binary for Tera-template enrichment).
  - Submodules under `skills/imported/` for external skill packs.
- Skill discovery model: filesystem walk of `skills/`, parse `SKILL.md` frontmatter, build a TF-IDF / keyword index. There is no Rust crate to "use" — UAR must scan and parse the files.
- This is a **fundamental schema mismatch** with UAR's existing WASM-flavored `Skill` domain type. Reconciliation work is the largest unknown of this phase (see §4 gap G3).

### 2.6 Containerization today

- `Dockerfile` exists (referenced by `docker-compose.*.yaml` files). Has not been audited as part of this assessment; the integration plan needs to extend it to:
  - Vendor (or COPY) the contents of both submodules at build time.
  - Build the `forge` CLI from prometheus-skill-system if we want skill enrichment at runtime.
  - Declare a writable volume (e.g. `/var/lib/uar/skills-derived/`) for skill-generated binaries/artifacts.

---

## 3. Acceptance criteria (definition of done for this phase)

| # | Criterion | Verification |
|---|-----------|--------------|
| A1 | KB list (`GET /api/knowledge` + `/api/uar/knowledge-bases`) returns a `document_count: number` per KB. | `curl … /api/knowledge` shows non-zero on a KB with uploads |
| A2 | Admin → Knowledge page shows the correct doc count without UI changes (frontend already reads `document_count`). | Manual check + screenshot |
| A3 | `prometheus-entity-management` is a submodule at `frontend/packages/prometheus-entity-management/`. | `git submodule status` |
| A4 | `frontend/` is a workspace (pnpm or bun) with the submodule as a workspace member. SPA imports use `@prometheus-ags/prometheus-entity-management`. | `frontend/package.json` workspaces entry; bundle succeeds |
| A5 | `configureEngine(...)` is called once during SPA bootstrap with concrete fetchers wired to UAR's REST endpoints; the Admin Knowledge / Documents / Agents pages render via `useEntityList` / `useEntity`. | Visual + `useEntity` hooks visible in DevTools |
| A6 | `prometheus-skill-system` is a submodule at `submodules/prometheus-skill-system/` (Rust-workspace-adjacent, NOT a workspace member because it isn't a single crate). | `git submodule status` |
| A7 | At UAR startup, every `SKILL.md` under the submodule is loaded into `SkillService` with `origin = "system-builtin"` and `deletable = false`. Counts logged at startup. | `tracing` log line + `GET /api/.../skills` listing |
| A8 | DELETE on a system-builtin skill returns `409 Conflict` with a clear message; UI hides the delete button for such skills. | `curl -X DELETE` + UI snapshot |
| A9 | `Dockerfile` builds both submodules' content into the image: skill manifests vendored at `/opt/uar/skills/builtin/`; optional `forge` CLI present at `/usr/local/bin/forge`. | `docker build` + `docker run image ls /opt/uar/skills/builtin/` |
| A10 | A persistent volume mount point (`/var/lib/uar/skills-derived/`) is declared and writable for derivative artifacts skills may emit at runtime. | `docker-compose.prod.yaml` volume entry |
| A11 | New phase's `progress.json` reflects completion; `current-waypoint.json` updated to point at follow-on phase. | File diff |

---

## 4. Gap analysis

| ID | Gap | Severity | Notes |
|----|-----|----------|-------|
| G1 | `KnowledgeBaseResponse` is missing `document_count`; backend has no aggregate query. | **High** (breaks visible UI) | Add `COUNT()` query against `knowledge_documents WHERE kb_id = $kb_id` in `list_knowledge_bases`. Surreal can do this in a single query with `array::len((SELECT id FROM knowledge_documents WHERE kb_id = $parent.id))` per row, or N+1 with caching. Postgres provider has the same hole. |
| G2 | Frontend is single-package; embedding entity-mgmt as a submodule under `frontend/packages/` requires turning the frontend into a workspace (pnpm-workspaces are simplest given the submodule's pnpm tooling). | **High** | Decide tool (bun workspaces vs pnpm). Touch points: `frontend/package.json`, `pnpm-workspace.yaml` (new), Vite resolve aliases, `build.rs` env. |
| G3 | UAR's `Skill` domain model is WASM-flavored; the skill-system's `SKILL.md` model is Markdown+YAML. There is no 1:1 mapping. | **High** | Decide whether to (a) introduce a parallel `BuiltinSkill` type that the runtime treats as a context/prompt injector instead of a callable artifact, or (b) extend `Skill` with a `kind: Wasm | Manifest` discriminator. Option (a) is more contained. |
| G4 | No "non-deletable" flag exists on `Skill`. | Med | Add `origin: SkillOrigin { Builtin, User }` + storage-provider guard in the DELETE handler. Need a migration for any persisted skills. |
| G5 | `prometheus-skill-system` is not a Cargo workspace, so it can't be added as a workspace member. | Med | Add as plain submodule under `submodules/`; have UAR's build.rs (or a startup-time scan) walk its `skills/` dir. |
| G6 | The skill-system repo contains *its own* submodules (`skills/imported/`, `surreal-memory-server`, `liter-llm`). Recursive submodule init must be handled. | Med | `git submodule update --init --recursive` everywhere; document in CONTRIBUTING + Dockerfile. |
| G7 | Dockerfile not yet aware of either submodule. | Med | Add COPY steps + ensure `git submodule update --init --recursive` happens before `cargo build`. |
| G8 | No persistent volume strategy for skill-emitted artifacts. | Low | Pick a path (`/var/lib/uar/skills-derived/`), expose env var (`UAR_SKILLS_DERIVED_DIR`), declare volume in compose. |
| G9 | The existing default-assistant has `kb.enabled` only just flipped to `true` in this session; ensure newly added builtin skills are wired into the same default agent (`skills.prefer` or `tools.bundles`). | Med | Decide later — could be opt-in per agent. |
| G10 | Frontend Admin → Skills page (currently labeled "WASM skill registry") needs UI for "Built-in" badge + disabled delete + system filter. | Low | Frontend follow-up after backend `origin` field lands. |
| G11 | No tests cover KB doc-count or builtin-skill loading today. | Med | New integration tests required to lock the contract. |
| G12 | Frontend currently bypasses TanStack Query for many fetches; switching to entity-management means each replaced page also gains/loses cache semantics. | Med | Per-page migration plan needed; consider keeping fetch layer thin and just composing `useEntity` over the existing services. |

---

## 5. Sequencing recommendation

1. **G1** first (low-risk, immediately user-visible) — add `document_count` to KB list + detail responses, surreal + postgres.
2. **G4 + G3 + G5 + G7** as one logical bundle — introduces submodule, `BuiltinSkill` shim, loader, Dockerfile updates. This is the riskiest slice and should be its own OpenSpec change.
3. **G2** — turn frontend into a workspace and bring entity-management in as a member, with one trial migration page (Knowledge list) to validate ergonomics.
4. **Broader entity-mgmt rollout** — page-by-page migration of Admin views.
5. **G10** — Skills UI polish for "Built-in" affordance.

Tests / docs / Docker artifacts trail each slice.

---

## 6. Open questions for the user before planning

1. Should this phase **replace** `runtime-provider-protocol-hardening` as the current waypoint, or run alongside it? Both are flagged `assessment_ready`.
2. **Workspace tool for frontend:** `pnpm` (matches the submodule's tooling) or `bun` (matches the rest of UAR's frontend toolchain)?
3. **Builtin skill model:** introduce a parallel `BuiltinSkill` type (smaller blast radius) or extend `Skill` with a `kind` discriminator (more unified)?
4. **Submodule placement for skill-system:** `submodules/prometheus-skill-system/` (top-level) or under `crates/` (even though it isn't a crate, it would group cleanly with Rust work)?
5. Should the new built-in skills be **enabled** by default in the default-assistant, or merely *registered* and opt-in per agent?

Answers to (1)–(5) gate the planning step (`/kbd-plan`).

---

## 7. Progress signal

Completed kbd-assess — prometheus-package-integration
