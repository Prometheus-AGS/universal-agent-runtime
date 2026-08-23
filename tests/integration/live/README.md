# Live integration tests

> **Current authority:** [Inference workflow guide](/docs/providers/inference).
> Only a request that reaches a real loaded model through UAR counts as inference
> integration evidence; recorded providers are non-certifying diagnostics.

The cases in this directory boot the real UAR server boundary and exercise HTTP
behavior. Backend selection changes what those cases can prove.

## Backend meanings

- `live` targets the operator's configured OpenAI-compatible proxy, normally
  `http://127.0.0.1:8181/v1`. Evidence is certifying only when the proxy reaches
  a real loaded model and the response returns through UAR.
- `recorded` runs an in-process response double. It is useful for fast component
  diagnostics, but it cannot prove inference, resilience, release readiness, or
  production behavior.

Most fixture-exact cases are recorded-only. A live-model case must use
content-tolerant assertions and record the provider, model, UAR source SHA,
profile, command, and observed response boundary.

## Run locally

With the real local proxy running:

```bash
scripts/live-integration.sh
```

The script checks the proxy before starting. Do not enable a recorded fallback
when collecting inference evidence: a fallback would change the claim being
tested. GitHub Actions are deployment-only and do not run this suite.

## Files

| File | Purpose |
|---|---|
| `backend.rs` | Selects the configured backend |
| `stub_llm.rs` | Recorded response double for non-certifying diagnostics |
| `harness.rs` | Starts the server and owns scratch state |
| `baseline_cases.rs` | Shared feature cases |
| `MATRIX.md` | Maps requirements to cases and limits |
