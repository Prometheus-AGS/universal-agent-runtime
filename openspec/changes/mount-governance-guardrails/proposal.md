# mount-governance-guardrails

## Why

The Cedar `GovernanceEngine` is well-built (hot-reload, request builders, `is_tool_allowed`/`is_allowed`/`is_skill_mutation_allowed`) and constructed into `AppState` (`lib.rs:98`, `server.rs:451`), but the `governance_layer` middleware (`governance/middleware.rs:48`) is **defined and never mounted** — so HTTP-level policy is dead. Only skill-mutation gating is live (`skills/service.rs:308`). Separately, the chat path does **no content guardrails**: `api_chat_completion` accepts the user input and streams a response with zero prompt-injection or PII screening. This is HP6 of `uar-harness-parity` (decision R4: in-house heuristics + mount the existing Cedar layer; no external service). The boot default is **permit-all** (`policies/default.cedar`), so mounting is safe-by-default.

## What Changes

- **Mount the Cedar `governance_layer`** into the router (after auth, in `server.rs`). It only enforces on requests carrying `X-Agent-Id` and the loaded policy set; with the existing permit-all `default.cedar` this preserves current behavior while making the seam live (restrictive `.cedar` policies now take effect at the HTTP layer).
- **Add the `validate_output` Cedar action** constant to the action model (`policy.rs` `actions`); `call_llm` already exists. This completes the action vocabulary the proposal references; no behavior change until policies use it.
- **In-house input guardrails on the chat path:** before the LLM call in `api_chat_completion`, screen the user input with local heuristics for (a) prompt-injection / jailbreak patterns and (b) obvious secrets/PII (API keys, credit-card/SSN-shaped strings). On a hit: record a metric, log a warning, and emit a `GuardrailFlagged` normalized event. **Detect-only by default** (non-blocking, preserves behavior); blocking is opt-in via config.
- **Config:** add a minimal `GuardrailsConfig` (`input_screening_enabled` default true — screening is non-blocking; `block_on_injection` default false; optional `policy_dir` override is out of scope). No new dependency; screening is local/regex-based.

**Explicitly NOT in scope (deferred):** replacing the 6-keyword `tool_requires_approval` heuristic with Cedar `is_tool_allowed` at the tool loop (a separate change — keeps this one bounded); an external moderation/guard model (R4: in-house only); output content-moderation beyond the existing HP4 sycophancy detection; ML-based PII/injection detection (heuristics only).

## Capabilities

### New Capabilities
- **`request-guardrails`** — `specs/request-guardrails/spec.md`. The Cedar HTTP governance layer is mounted and enforces the loaded policy set (permit-all by default, preserving behavior); chat input is screened for injection/PII with detect-only-by-default flagging and opt-in blocking; flagged input surfaces a `GuardrailFlagged` event + metric; the `validate_output`/`call_llm` action vocabulary exists.

## Impact

- **Affected code:** `src/server.rs` (mount `governance_layer`; input screen in `api_chat_completion` before `start_run`), `src/uar/governance/policy.rs` (add `validate_output` action const), `src/uar/guardrails.rs` (new — heuristic injection/PII screen, pure + testable), `src/uar/mod.rs` (register module), `src/uar/domain/events.rs` (+ `GuardrailFlagged` event + classification), `src/uar/api/sse.rs` (map to `agui.guardrail`), `src/uar/telemetry/metrics.rs` (guardrail metric), `src/config.rs` (+ `GuardrailsConfig`).
- **APIs:** no new endpoints. The mounted layer may return `403 GOVERNANCE_DENIED` for `X-Agent-Id` requests under restrictive policies (none by default). One new SSE event (`agui.guardrail`).
- **Behavior preservation (Rule 32):** permit-all default + detect-only screening ⇒ no change to existing chat behavior out of the box. Blocking and restrictive policies are opt-in.
- **Provider compatibility:** unaffected — screening is pre-LLM and provider-agnostic.
- **Runtime/UX:** flagged inputs surface a guardrail event; with blocking enabled, an injected/PII input is rejected before the LLM call.
- **Security (Rule 33):** adds the first input-side defense (injection/PII) and a live authorization seam; the event must carry only the matched category/pattern label and a short reason — never the full input or the secret value.
- **KBD workflow state:** YES — HP6, the final change of `uar-harness-parity`.
