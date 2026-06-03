# Tasks — guardrail-pii-block

## 0. Bootstrap
- [x] 0.1 Confirm seam: block condition `server.rs:3732`, `GuardrailsConfig` in `config.rs`, `GuardrailCategory::Pii`
- [x] 0.2 `cargo check --features postgres-backend` green on branch base

## 1. Config
- [x] 1.1 Add `block_on_pii: bool` (`#[serde(default)]`, default false) to `GuardrailsConfig`; init in `Default`; update the `block_on_injection` doc note

## 2. Block condition (server.rs)
- [x] 2.1 Widen the block gate to `(block_on_injection && Injection) || (block_on_pii && Pii)`; reuse the existing guardrail-error response

## 3. Validation (gate)
- [x] 3.1 `cargo check --features postgres-backend` clean; zero new warnings
- [x] 3.2 `cargo clippy` — no new warnings in touched files
- [x] 3.3 `cargo test --features postgres-backend --lib` — existing pass
- [ ] 3.4 Manual: `block_on_pii=true` + PII input → rejected; default → flagged-only; injection block unchanged (pending live env)
- [x] 3.5 `openspec validate guardrail-pii-block --strict`; update `.kbd-orchestrator` progress

## Notes
- Default false ⇒ behavior-preserving; opt-in. PII heuristic unchanged. No new dependency.
