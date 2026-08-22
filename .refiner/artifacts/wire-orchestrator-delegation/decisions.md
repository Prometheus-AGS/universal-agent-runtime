# Decisions — `wire-orchestrator-delegation`

## Iteration 1 — 2026-08-18T19:28:08Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** local AgentNodes now use the already-resolved run driver, so
  server and embedded runtimes can delegate without inventing endpoints or
  configuration. Remote URL AgentNodes retain A2A behavior.
- **Uncomfortable result:** the dormant local AgentNode path claimed to resolve
  an in-process agent but actually called `/a2a/{id}` routes that the server did
  not mount. Attaching that graph unchanged would have converted a cosmetic
  feature into a production failure.
- **Independent review:** pending critic and judge.

## Iteration 2 — 2026-08-18T19:39:12Z

- **Decision:** terminate after independent re-review.
- **Iteration:** 2 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** delegation is successful only when the specialist returns
  non-whitespace text; recorded mode requires the exact fixture and live mode
  requires a non-whitespace contribution after the attributed prefix.
- **Uncomfortable result:** iteration 1's live test proved routing and step
  events but could accept `[rust-reviewer]` with no answer. Its retained PASS
  claim was stronger than its assertion.
- **Independent review:** critic PASS; judge PASS. Both independently confirmed
  the prior empty-output blocker is closed and found no remaining blocker.
