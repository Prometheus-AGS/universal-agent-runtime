# ASSESS-MIRROR — condition 5: local asset-mirror primary/backup feasibility

Date: 2026-10-19 15:35 UTC. Basis: v4 source + live probes; operator correction
(two byte-identical asset copies exist; Boss sidecar excluded as a dependency;
dedicated mirror only; no hosted spend; OpenAI route barred).

## The distinction that makes this feasible

One embedding SPACE = `{fastembed, bge-small-en-v1.5, 384}`. One space does
NOT mean one possible backend INSTANCE. Two independently stored,
byte-identical copies of the five FastEmbed assets are two instances of the
same space. Byte-identity is the correctness requirement: equal
dimension/model label alone is insufficient, per the operator.

## Hash verification (independent re-check, this session, read-only)

SHA-256 over all five `MODEL_FILES` (fastembed.rs:12) in both existing
copies:

| file | src/uar/runtime/matching/models | dist/boss-sidecar/.../uar-models | match |
|------|------|------|------|
| bg-small-en-v1.5.onnx | 6c9c6101…bfe4 | 6c9c6101…bfe4 | YES |
| tokenizer.json | d241a60d…5c66 | d241a60d…5c66 | YES |
| config.json | f2c87ea1…e186 | f2c87ea1…e186 | YES |
| special_tokens_map.json | b6d346be…7ee3 | b6d346be…7ee3 | YES |
| tokenizer_config.json | 9261e7d7…2ab3 | 9261e7d7…2ab3 | YES |

A no-paygo local mirror is therefore technically available. The Boss sidecar
copy is evidence of availability only; any implementation uses a NEW
dedicated mirror under the fixture tmp root, never the sidecar path.

## Mechanism (as-built, with citations)

- `resolve_models_dir` (fastembed.rs:48-71): ordered candidates
  `[UAR_MODELS_DIR env, config models_dir, /app/models,
  src/uar/runtime/matching/models, ./src/uar/runtime/matching/models]`;
  first dir containing all five files wins. Existence check only — no hash,
  no readability check.
- Engine is LAZY (fastembed.rs:73-111): directory resolved and assets loaded
  on first embed call, then cached. A directory change affects only the next
  engine init (fresh process, or retry after a failed init — `engine` stays
  `None` on failure, so the next call re-resolves).
- Slot metadata (`swappable.rs:38-55`): `{backend_name, model_id, endpoint,
  vector_dimension}` captured eagerly at swap. **No instance provenance.**
- Current truthful surfaces: activation map (`settings.rs:856-895`),
  `fingerprint()` (`swappable.rs:118+`) feeding the KB space guard. Neither
  carries which directory serves.
- The init log line `info!(dir = …)` (fastembed.rs:80) is forensic only — it
  is not a status surface.

## Verdict

**Mechanical failover: FEASIBLE without code change.** Explicit
configuration = `UAR_MODELS_DIR=<primary>` + `models_dir=<backup>`; failure =
primary assets absent at engine init → resolution binds backup; recovery =
restore primary → next init binds primary. All within local disk, no
credentials, no network.

**Truthful backup status: NOT feasible without the smallest shared
correction.** After a directory failover the slot metadata and every status
surface remain byte-identical — the instance change is invisible to an
operator. The init log line is forensic, not status. Per the operator, this
silent behavior does not count as proved failover or truthful backup status.

**Concrete correction (smallest shared, ~80 lines, no behavior change to
matching/ingest/guard):**

1. `embeddings/mod.rs`: `BackendProvenance { source_dir: String,
   asset_manifest_sha256: String }`; default trait method
   `fn provenance(&self) -> Option<BackendProvenance> { None }` (no ripple
   to openai/voyage/cohere/candle).
2. `fastembed.rs`: `FastEmbedBackend::new` eagerly resolves + SHA-256-hashes
   the five assets (manifest digest = sha256 over concatenated per-file
   digests, sorted by filename); construction FAILS when assets are absent.
   Side effect, named: the honored-check at activation/boot now also rejects
   a fastembed candidate whose assets are missing — closes the latent gap
   where build succeeds but first embed fails. On 1907 assets exist; no
   regression.
3. `swappable.rs`: `BackendSlot` captures `provenance`; surfaced in the
   activation map payload (`settings.rs`) and boot overlay log (`server.rs`).
4. **Provenance is deliberately NOT added to `EmbeddingFingerprint`**: an
   instance change is not a space change; the KB guard must not fire on
   A→B failover. This positively tests that the guard distinguishes
   space-change from instance-change (feeds condition 3 evidence).

## Probe protocol (if approved)

Fixture instance on scratch port 1911, scratch surreal KV, config
`models_dir=<dedicated-backup>`, env `UAR_MODELS_DIR=<dedicated-primary>`;
both mirrors fresh copies of the five assets, hashes re-verified at setup.
1907/1908 untouched.

1. Boot → provenance=primary, manifest H; bind probe KB; baseline search.
2. Rename primary aside → restart → provenance=backup, manifest H (identical)
   → search returns hits (same space, no guard refusal).
3. Restore primary → restart → provenance=primary → search intact.
4. Receipts: per-step provenance readback + manifest digests + search scores.

Reversible end-to-end; fixture removed after. Estimated: one v5 build
(`--bin universal-agent-runtime` only), three fixture boots, ~30 calls.

## If the correction is not approved

Concrete reason for NOT READY: no status surface carries instance
provenance; the deterministic fallback is operator-invisible, so failover
cannot be truthfully reported — only forensically inferred from logs.

## Claim-class correction (2026-10-19 15:50 UTC, operator-directed)

The mirror probe's evidence class is narrowed BEFORE it runs:

1. Local BGE mirror boots (A→B→A across byte-identical asset copies) prove
   **restart resilience with truthful provenance** of the RAG fastembed
   instance pair — nothing more.
2. They are NOT live provider failover: no provider switch occurs, no
   second provider is exercised, and the memory-service embedding
   configuration (OpenAI/1536 in personal config; Mistral arm in dirty
   `service.rs`; #420 silent-fallback history) is a different subsystem
   this probe never touches.
3. Therefore the probe does NOT complete condition 5
   (failover/recovery). Condition 5 closes as: restart resilience
   evidenced with truthful provenance; bounded compatible FAILOVER not
   demonstrated — child verdict remains NOT READY on condition 5 per the
   frozen exit contract, with the blocker named.
4. Build correction: the first v5 build attempt ran with default
   (`minimal`) features, which EXCLUDE `local-models` — that binary is not
   a FastEmbed/provenance proof and is discarded for evidence purposes.
   The valid same-source build is
   `cargo build --release --features server-full --bin universal-agent-runtime`
   (sequential after the minimal build, no concurrent cargo).
