# RECONCILIATION — intended Mistral/OpenAI configuration vs local BGE test configuration

Date: 2026-10-19 16:30 UTC. Purpose: name the ACTUAL primary and backup in
each configuration, verify embedding-space compatibility pairwise, and state
what may never be mixed. Sources: operator-supplied personal-config facts
(files not read — secret boundary), repo-verified code, fixture observations.

## The three configurations on the table

| # | Configuration | Space identity | Status |
|---|---------------|----------------|--------|
| C1 | Local test (this child's fixtures) | `{fastembed, bge-small-en-v1.5, 384}` | honored, serving on 1907/1908; mirror pair staged |
| C2 | Intended memory per personal `~/.uar/config.yaml` (operator-supplied) | `{openai, text-embedding-3-small, 1536}` | named in YAML; NOT exercised in this child (no calls, no paygo) |
| C3 | Environment-hinted Mistral (`MISTRAL_API_KEY` export reference; dirty `service.rs:100-107` arm) | **UNDETERMINED** — no model, no dimension selected in any config the operator cited; compatibility with C1/C2 is therefore UNKNOWN, not measured-incompatible | code-supported, config-absent |

## Actual primary and backup, per configuration

- **C1 (local test)**: primary = dedicated mirror A
  (`uar-mirror/primary`, `UAR_MODELS_DIR`), backup = dedicated mirror B
  (`uar-mirror/backup`, config `llm.embedding.models_dir`). Same space:
  all five assets byte-identical, SHA-256 verified per file at setup
  (INVENTORY hash table). This is the ONLY pair in scope whose
  compatibility is certifiable.
- **C2 (intended memory)**: primary = OpenAI `text-embedding-3-small`/1536
  as named. No backup is selected in that YAML.
- **C3 (Mistral)**: a credential existing in the environment does NOT define
  an embedding space. Until a model and dimension are selected in config,
  C3 has no space identity — it cannot be named a backup for anything, and
  treating it as one would be exactly the silent substitution the frozen
  contract forbids.

## Pairwise compatibility verdict

| Pair | Verdict | Basis |
|------|---------|-------|
| C1 mirror A ↔ C1 mirror B | **COMPATIBLE** | identical assets, identical manifest (verified) |
| C1 (BGE/384) ↔ C2 (OpenAI/1536) | **INCOMPATIBLE (measured)** | model AND dimension differ; vectors must never share a KB |
| C1 (BGE/384) ↔ C3 (Mistral, no selected model/dim) | **UNKNOWN / UNCERTIFIABLE** | C3 has no space identity; no measured incompatibility — mixing is refused as a precaution, not on evidence |
| C2 (OpenAI/1536) ↔ C3 (Mistral, no selected model/dim) | **UNKNOWN / UNCERTIFIABLE** | C3 space undefined; a 1536-dim Mistral selection might exist but is not configured — not evidence |

## Never-mix guarantees in force

1. KB space guard refuses cross-space queries explicitly
   (`manager.rs:4658-4704`, `embedding_space_mismatch`).
2. v4 honored-check rejects unhonored mutations pre-persist — no REST path
   can point a KB's space at a model the backend does not serve.
3. v5 provenance correction: load-time per-file digest verification refuses
   asset drift between build and load — a substituted or corrupted asset set
   can never silently serve under a matching manifest claim.
4. This child's probes bind every KB to the C1 space only; no probe
   introduces a second space.

## Consequence for condition 5 (failover)

The only compatible, reachable primary/backup pair within the child's
limits is the C1 mirror pair (restart resilience). Hosted failover
(C2↔C3) is (a) unexercisable — no provider calls, no paygo, no personal
config change — and (b) uncertifiable as a space today because C3 selects
no model/dimension: its compatibility with C2 is UNKNOWN, and refusing to
mix it is a precautionary policy, not a measured incompatibility. Both
facts are external blockers, recorded, not routed around. Condition 5
stays NOT MET as failover per the frozen contract.
