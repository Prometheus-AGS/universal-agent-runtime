# R7 + R8 no-login probe receipts — 2026-10-19 ~17:15 UTC (v4, 1907)

Candidate: v4 `96bf4868…5154` (1907, pid 24103). No login, no admin key.

## R7 — Anthropic interface turn → **FAIL (contract gap, truthful refusal)**
- `POST /v1/messages` anonymous → **HTTP 403**
  `{"type":"error","error":{"type":"invalid_request_error","message":"credential_identity_required"}}`
- Retry with session binding (`x-uar-session-id` + cookie
  `41732fd0…`) → identical 403. Receipts: r7.json/r7.hdrs, r7b.json.
- The frozen criterion expects a no-login 200 turn. The v4 surface
  refuses without a credential identity. Same governance family as
  R2(d)/R9. No repair attempted (auth loosening = config/scope change).

## R8 — A2A interface turn → **FAIL (contract gap, truthful refusal)**
- 50052 is a gRPC listener (`a2a_grpc_listener`, feature a2a-transport);
  HTTP/1.1 probes correctly fail (HTTP/0.9 anomaly). Proto compiled from
  `proto/a2a_v1.proto` in scratch; `lf.a2a.v1.A2AService/SendMessage`
  with a valid ROLE_USER message →
  **`StatusCode.UNAUTHENTICATED verified user context required`**.
  Receipt: r8/r8-result.txt.
- Frozen criterion expects a no-login 200 turn. Refused on v4/1907.

## Note
R6 (`/v1/chat/completions` anonymous) remains the only interface-turn
row that passes no-login on this candidate config. R2(d), R7, R8, R9
form one governance family: anonymous interface turns beyond the OpenAI
lane are refused by design in this configuration.
