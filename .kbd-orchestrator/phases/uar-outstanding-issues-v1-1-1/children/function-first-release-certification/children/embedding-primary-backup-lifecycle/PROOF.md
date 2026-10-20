# PROOF — embedding-primary-backup-lifecycle conditions 1–6

Date: 2026-10-19. Candidate of record: v4 sha256 `96bf486872c57d22071596be6d81af42d3dfe7dee801a40226e507caf4225154`
(preserved copy: `uar-r14/uar-v4-preserved.bin`, hash re-verified 15:40 UTC).
Claim classes per frozen amended EXIT-CONTRACT and the 15:50 UTC
operator-directed corrections.

## Condition 1 — inventory — PASS

`INVENTORY.md`: compiled catalog (fastembed/candle/openai/voyage/cohere,
mod.rs:109-124 + Cargo.toml:220-224 feature matrix), per-backend approved
identity + credential presence (names only, never values), settings schema
(`llm.embedding.backend|model` only), live persisted row
(`llm.embedding.model = bge-small-en-v1.5`, REST readback). Scope correction
recorded: RAG-fixture inventory does not generalise to the memory-service
subsystem (OpenAI/1536 personal config; Mistral arm service.rs:100-107;
unknown-provider rejection service.rs:138; #420 silent-fallback history;
fixture memory-service local BGE/384 boot observation).

## Condition 2 — primary/backup resolution + truthful status — PASS

`RESOLUTION.md`: single active slot (`SwappableEmbeddingBackend`), two
ordered writers (boot overlay src/server.rs:1006-1072, REST activation
settings.rs:893-932), no automatic embedding backup machinery
(`llm_failover.*` is chat-model only), activation map + honored-check
truthfulness (settings.rs:852-887), boot overlay refusal of unhonored drift
(observed 13:23:03). v5 adds instance-level truth: `/embedding/provenance`
status route + provenance fields in swap/boot logs.

## Condition 3 — pinned space identity: ingest/bind/restart — PARTIAL

- Ingest-time binding: PASS — probe KB `18906c43-80b9-4c4f-9550-1f555aad3481`
  bound `{fastembed, bge-small-en-v1.5, 384}` at ingest; KV readback
  13:22 UTC (P2 receipts, config-lifecycle-map §9).
- Restart restore: PASS — v4 restart legs P2/P4; binding and retrieval
  survive restart (search 0.6399 post-restart, P2 §9).
- Query-time enforcement: **UNVERIFIED** — guard code path present
  (manager.rs:4658-4704 `embedding_space_mismatch`) but the frozen leg
  requires a real second honored space; none exists in this deployment
  (INVENTORY). Per frozen contract: UNVERIFIED, never PASS-by-impossibility.

## Condition 4 — invalid config rejection without state loss — PASS

Fresh v4 evidence 15:26-15:32 UTC (this session, port 1907):
- Baseline retrieval: 3 hits, top 0.38963237.
- PUT `llm.embedding.model=all-MiniLM-L6-v2` (equal 384-dim class) →
  422 `unsupported_setting`, message names canonicalization and
  "nothing was persisted and the active backend is unchanged".
- Row after reject: effective == saved == `bge-small-en-v1.5`, revision
  UNCHANGED (`2026-10-19T13:26:16.061406000Z` before and after).
- Post-reject retrieval: 3 hits, top 0.38963237 — byte-identical outcome.
Plus P2 §9: bogus-backend 422; out-of-band bogus row refused at boot
overlay 13:23:03.

## Condition 5 — failover/recovery — NOT MET (failover), resilience probe pending

Claim class (operator-corrected 15:50 UTC): local BGE mirror A→B→A boots
prove **restart resilience with truthful provenance** of the RAG fastembed
instance pair only — NOT live provider failover, NOT memory-subsystem
failover, NOT condition-5 completion. v5 server-full build + scratch-port
1911 probe: pending build completion. Blocker for full condition 5: no
approved second provider/space reachable within limits (INVENTORY scope
correction; no Mistral call, no OpenAI route, no personal config change).

## Condition 6 — incompatible-space refusal without data loss — PASS

- Equal-dimension different-model refusal: the 15:26 UTC 422 above IS the
  equal-dim different-model case (all-MiniLM-L6-v2 is 384-dim) — refused
  pre-persist, row revision unchanged, retrieval intact. PASS.
- P2 §9: unhonored-model 422 with byte-identical row; documents intact
  across every rejected mutation leg.
- Query-side incompatibility refusal: shares condition 3's query-guard leg —
  UNVERIFIED (same frozen blocker).

## Preliminary verdict mapping (frozen exit contract)

Conditions 1, 2, 4, 6: PASS (evidence-bound). Condition 3: PARTIAL —
query-guard leg UNVERIFIED. Condition 5: NOT MET as failover (resilience
probe pending; blocker named). Child closes **NOT READY** — one frozen
blocker: no second honored embedding space/provider exists in this
deployment within the child's limits, so query-guard exercise and bounded
compatible failover remain unproved.

## Build ledger

- v5 attempt 1 (16:05 UTC): `cargo build --release --bin universal-agent-runtime`
  — default `minimal` features (NO local-models; fastembed.rs not compiled).
  Verdict: Finished release [optimized] in 26m 46s, zero errors — compile
  check of mod.rs/swappable.rs/settings.rs/server.rs only. DISCARDED for
  evidence; binary superseded by attempt 2.
- v5 attempt 2: `cargo build --release --features server-full --bin
  universal-agent-runtime` — sequential, started after attempt 1 exited.
  Verdict: Finished release [optimized] in 19m 08s, exit 0. One benign
  linker warning (__eh_frame >16MB compact-unwind note; EH performance,
  not correctness). External writer observed during build: pid 44624
  `cargo build --release --locked --no-default-features --features
  server-full --bin uar-sidecar --target aarch64-apple-darwin` (not this
  child's; package-cache lock wait only, disjoint artifact).
- v5 identity: HEAD 38f0b5d85960e3cb8445607750538ca500efdf17 + dirty tree
  (by design; #419 + checkpoint repairs + this correction);
  `git diff HEAD` sha256 cf384d851dc8c0de704067ebdd2c0f7d2f81da1550711b6292e4123efa8820d2.
- v5 binary sha256: 978e2f8d4e28bc2f3d8887c741cef5b5165575057971ea28c1f23b7bc453822a
  (317,616,384 bytes). Feature proof: `/embedding/provenance` route and
  `asset_manifest_sha256` strings present in binary (local-models compiled).
- Correction diff: 5 files — embeddings/mod.rs (+BackendProvenance, trait
  default), fastembed.rs (eager resolve + per-file digests + load-time
  re-verification), swappable.rs (slot provenance), api/settings.rs
  (provenance route + swap log), server.rs (boot overlay + provenance log).

## Probe receipts — A→B→A on v5 (2026-10-19 16:41–16:43 UTC, port 1911)

Two in-scope fixture defects found and fixed before the pass (hardened
script fail-fasted correctly both times): (1) grpc_port 50053 already held
by foreign pid 17661 → moved fixture to free 50153 (50052/1907 untouched);
(2) KB create path trailing slash 404 (axum nest matches
`/api/uar/knowledge-bases` without slash — verified read-only on 1907) →
script corrected. Reran only the affected legs; final run:

| Leg | Time (UTC) | Provenance dir | Manifest | Search |
|-----|-----------|----------------|----------|--------|
| A1 boot (primary) | 16:41:09 | `…/uar-mirror/primary` | `f6dcb84ec0c04198…` | — |
| A2 ingest+bind | 16:41:10 | KB `4e5964cc-7eeb-46a7-aa81-830824205683`, doc `b7d79149-67b6-4198-88dc-dd4b91c47b2d` accepted | — | baseline hits=1 |
| B boot fallback (primary renamed aside) | 16:42:38 | `…/uar-mirror/backup` | `f6dcb84ec0c04198…` (IDENTICAL) | failover hits=1 |
| A′ recovery (primary restored) | 16:42:56 | `…/uar-mirror/primary` | `f6dcb84ec0c04198…` | recovery hits=1 |

Separated findings, per the operator's record-separately instruction:

1. **Boot-time fallback: PROVEN.** With primary assets absent at process
   start, resolution bound the backup directory and served — and the v5
   provenance surface reported the backup directory truthfully (REST 200,
   `/api/uar/settings/embedding/provenance`).
2. **Backup query continuity: PROVEN; backup INGESTION continuity: NOT
   exercised (claim corrected 16:55 UTC).** The v1 probe uploaded its only
   document while primary served (A2); leg B only SEARCHED. The KB bound on
   primary answered on the backup instance (hits=1, same space) — query
   continuity. The KB space guard correctly did NOT fire — instance change
   is not a space change. An extended probe (below) exercises ingestion
   while backup serves.
3. **Recovery: PROVEN.** Restoring primary and restarting returned
   provenance to the primary directory, manifest identical, query intact.
4. **Live failover: NOT DEMONSTRATED.** Every transition required a
   process restart; no in-process swap occurred. This is restart
   resilience, not live failover. Condition 5 unchanged: NOT MET as
   failover.

Minor observations (recorded, not gate-relevant): the ingestion-job
status endpoint returned an empty status field throughout the bounded
60s wait (script degraded to the hard search assertion as designed);
fixture processes required SIGKILL after a 10s SIGTERM grace on both
stops (shutdown behavior, bounded, cleaned by the trap).

Manifest semantics: sha256 over asset names (MODEL_FILES declaration
order) + per-file sha256 digests, computed by v5 at backend construction;
load-time re-verification refused no legs (assets never drifted).

## Extended probe receipts — backup-served ingestion (16:48–16:52 UTC, v5, port 1911)

Extended B leg per operator criterion correction. Fresh scratch DB;
full A→B→A rerun with the ingestion leg added:

| Leg | Result |
|-----|--------|
| A1 boot | provenance `…/primary`, manifest `f6dcb84ec0c04198…` |
| A2 ingest | KB `20be203c-ad58-4c32-8cef-c16247295e2e`; doc1 `6f197218-3652-4284-b0fe-6d044caae277`; baseline search hits=1 |
| B boot fallback | provenance `…/backup`, manifest IDENTICAL; doc1 query hits=1 |
| **B-ingest (backup serving)** | doc2 `fa463ce0-ce1d-42c6-9cbc-06f948a0c33a` uploaded + accepted; search `zebra quasar lantern` → hits=2, top-hit content contains `zebra` (asserted) |
| A′ recovery | provenance `…/primary`; doc1 query hits=2; zebra query hits=2 with top content `zebra` — backup-ingested document survives recovery |

Final separated findings for condition 5:

1. Boot-time fallback: **PROVEN** (backup bound + truthfully reported).
2. Backup query continuity: **PROVEN** (doc1 searchable on B).
3. **Backup INGESTION continuity: PROVEN** (doc2 ingested while backup
   served; retrievable on B with content assertion; intact after recovery).
4. Recovery: **PROVEN** (primary restored, both documents searchable).
5. Live failover: **NOT DEMONSTRATED** — all transitions are restarts;
   no in-process or provider swap. **Condition 5 remains NOT MET as
   failover** per the frozen criterion.
