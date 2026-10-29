# Validation queue while the A2UI architecture waits — 2026-10-28

The 48 open catalog scenarios (E01-E28, I01-I20) were classified by four
read-only teammates (uar-runtime ×2, uar-state, general-purpose). Nothing was
run. Verdicts come from source reading and read-only GETs against the live
runtime.

**The runtime is at `http://127.0.0.1:60675`**, the identity proxy, with UAR
upstream on 53587. Port `:8088` is not listening. Behavior scripts need
`UAR_BASE_URL=http://127.0.0.1:60675`.

## Summary

| Group | Count | IDs |
|---|---:|---|
| Ready to author and run | 28 | E01, E02, E03, E08, E10-E18, E20, E25-E28, I01-I07, I09, I10, I20 |
| Waits on the A2UI architecture | 1 | I19 |
| Needs an operator decision | 19 | E04-E07, E09, E19, E21-E24, I08, I11-I18 |

## Ready (no decision needed)

Suggested order is cheapest and most reuse first. Each runs once; stop on PASS.

| Order | ID | Note |
|---|---|---|
| 1 | E11 | Agent update and delete survive restart. Composes `custom-agent-lifecycle` with `restartOwnedUar`. |
| 2 | E12 | Deleted KB stays deleted after cold start. Composes `linked-kb-delete` with `knowledge-restart`. |
| 3 | E28 | Below-threshold and tied skills stay inactive. Keyword ties at 1.0 against a margin of 0.05. |
| 4 | E20 | The legacy classifier tfidf path works as is (it is the hardwired default). |
| 5 | E15 | One prompt, lexical versus semantic winner. |
| 6 | E02 | Auto strategy. Cheapest crossing is glm-5.3: 0.7 × 200k gives sliding_window 336. |
| 7 | E01 | Hierarchical tiers. The C43 summarizer fix makes summaries apply. |
| 8 | I20, I05 | Anthropic default selection, JSON then stream. |
| 9 | I06, I07 | Legacy A2A JSON, then the v1 stream. |
| 10 | I01, I02 | Legacy AG-UI chat framing, then dual framing (dedup oracle). |
| 11 | I03, I04 | Native run plus stream. Subscribe immediately after the POST. Keep `STATE_*` assertions out of I04. |
| 12 | I09, I10 | Shipped Python SDK from `sdks/python/.venv`. |
| 13 | E13, E14 | Catalog activation and the activation limit, which needs two tool calls in one run. |
| 14 | E25, E27 | Inherited and auto policy (the C42 pattern). |
| 15 | E16, E17 | Service embedding and hybrid matchers. No prior runtime pass. |
| 16 | E18 | LLM matcher. Risky: strict JSON parsing fails on fenced or reasoning output. |
| 17 | E10 | AG-UI selection replay with no second dispatch. Choose the dispatch-count evidence first. |
| 18 | E08 | Unknown MCP effect not repeated. Needs a peer that commits and then drops the connection. |
| 19 | E26 | All-skills policy. Needs the disposable empty-skill instance (c52 fixture), because the main runtime has 212 skills. |
| 20 | E03 | Memory retention. Needs an aged-record fixture on the disposable instance. |

Shared support to author once: `support/skill-fixture.mjs`, because six skill
step files each duplicate the same request helper.

## Waits on the A2UI architecture

- **I19:** dedicated AG-UI route projects A2UI. Blocked on
  `a2ui-surface-architecture` and PEM #70-#74.

## Needs an operator decision

| ID | Blocker | Options |
|---|---|---|
| E09 | Needs a second user. Tests carry no credentials, and the proxy supplies one fixed user. | Start a temporary second proxy (`PROXY_JWT_SUB` differs, another port) and stop it afterward; or skip. |
| E05 | All five models are flagged `supports_structured_output:false`. | Correct the flag if a model really supports JSON schema; add a model; or skip. |
| E06, E07 | No vision model is configured. Chat completions also drops image parts (`server.rs:6141-6158`), a probable source gap. | Add a vision model plus a code fix; or skip. E07's tool-group half could run on its own. |
| E04 | No public capability-constrained dispatch or exclude parameter (`manager.rs:5942` uses default requirements). One provider, so no second route to cool down. | Product change; or drop from the catalog. |
| I08 | ACP JSON never returns the answer (`acp/handler.rs:518-523, 606-615`). | Product change; or redefine the row. |
| E19, E21-E24 | `config.intent_classifier` is never passed to the run manager (`manager.rs:1249-1257` hardcodes the default). The `intent_classifier.*` settings are stored but have no effect. E22 also needs a built wasm guest. | One runtime code change unblocks five scenarios; or defer. |
| I11, I12 | The shipped TypeScript SDK `dist` is from July and lacks `agentId`. | Rebuild `dist` (a build step); or skip. |
| I13, I14 | The Rust SDK has no built artifact. | `cargo build` the SDK example; or skip. |
| I15-I18 | gRPC requires a bearer token (`a2a/grpc.rs:55-61`). The proxy forwards HTTP only. No shipped gRPC client exists. | Build a gRPC-capable identity proxy and choose a client; or defer. |

## Other findings

- **Secrets exposed in a tool output.** While reading
  `target/prompt-macos-hbitxj2c/config.json`, one teammate printed its
  plaintext `jwt_secret` and `settings_admin_key` into its own tool output in
  this session's local transcript. The values were not copied into any file or
  message. This is the disposable Mac fixture's configuration.
- **Implicit-default checks may be affected.** `/v1/models` reports
  `delegation_available:false` (`default_agent_not_installed_or_ambiguous`)
  while `GET /api/agents/default-agent` returns 200. This may affect I05 and
  I20. Unverified.
- **Stale module comment.** `realtime.rs:1-37` says there is no A2UI emission
  call site, which is false. Already recorded in the A2UI child.
