# Runtime contracts — proposed acceptance fragment

> Current certification authority: the 2026-10-17 function-first amendment at
> the end of this document supersedes older gate-opening and full published-group
> instructions. Historical task states and receipts are preserved. Gate CLOSED.


Phase: `uar-outstanding-issues-v1-1-1`  
Stage: Spec  
Status: proposed; no production implementation, fixture, runner, or gate has run.

## Behavior: trusted-host run-binding diagnostic

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1` / R1 / `provider-diagnostic-status` |
| Source identity | The v1.1.1 candidate tree selected by the lead; source revision recorded in the final receipt. |
| Production entry point | Authenticated `POST /api/uar/runs`, followed by `GET /api/uar/runs/{id}/stream` and the nested AG-UI run stream when configured. |
| Real collaborators | The UAR server, `RunManager`, trusted-host accounting provider, authenticated principal context, persisted run history, and the configured provider wire peer. |
| Boundary exercised | Authenticated HTTP admission to trusted-host model binding and the normalized/AG-UI SSE projections. |
| Observable result | A bounded run refused by accounting/binding projects the existing `orchestrator_start_failed` code and a safe diagnostic reference; no provider or tool effect is observed. |
| Negative control | An unattested remote accounting backend or missing verified accounting principal rejects before provider dispatch; a valid qualified durable profile remains separately eligible. |
| Isolation | Runner-owned scratch service config, storage roots, credential sources, provider/MCP traces, ports, and evidence root. The runner removes only its declared scratch roots. |
| Prerequisites | Completed R1 implementation; an authenticated fixture principal; a real configured provider peer; a qualified durable profile whose infrastructure evidence is retained. |
| Final local gate | The R5 acceptance manifest includes this case; only the lead invokes the final phase gate after the full production boundary. |
| Evidence | Pending: manifest-selected candidate source identity, sanitized HTTP/SSE transcripts, provider trace, persisted run outcome, exit code, and receipt path. |
| Limitations | A fixture's durable profile does not attest the reported deployment's backing storage or identify its erased historical inner error. |

## Behavior: ordinary-manager terminal truth and native Anthropic stream framing

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1` / R2 and R3 / `model-path-resiliency` |
| Source identity | The v1.1.1 candidate tree selected by the lead; the receipt records the pinned Liter revision from `versions.toml`. |
| Production entry point | Authenticated `POST /api/uar/runs` and `GET /api/uar/runs/{id}/stream`; the configured native Anthropic driver calls its provider's `POST /v1/messages` stream endpoint. |
| Real collaborators | UAR server, ordinary `RunManager`, orchestrator, native Anthropic driver, pinned Liter decoder, configured HTTP provider peer, persisted run history, and AG-UI projection. |
| Boundary exercised | HTTP request admission, provider SSE bytes, normalized event stream, manager settlement, persisted status/history, and AG-UI `RUN_ERROR` projection. |
| Observable result | EOF without observed orchestrator `Done` is persisted and projected as error; valid terminal text and tool-loop streams succeed. Equivalent LF/CRLF/CR byte streams produce equivalent normalized events, while malformed/unfinished streams fail. |
| Negative control | Empty, metadata-only, text-only, partial-tool, and complete-tool EOF without completion do not settle successful or execute pending tools; malformed known Anthropic data and EOF without `message_stop` fail. |
| Isolation | Runner-owned scratch database, authenticated fixture credentials, dedicated provider/MCP peer ports and traces, and a disposable evidence directory. Cancellation joins the captured run before cleanup. |
| Prerequisites | Completed R2/R3 implementation, real controlled provider peer, valid run artifact, authenticated caller, and no concurrent runner using the same ports or roots. |
| Final local gate | The R5 acceptance manifest includes full-harness and native provider cases; it runs only after every phase implementation item is complete. |
| Evidence | Pending: byte-partition matrix, HTTP/SSE transcript, persisted status/history, AG-UI projection, provider/MCP trace, candidate identity, exit code, and receipt. |
| Limitations | Controlled peers establish UAR's protocol behavior; they do not certify a third-party provider deployment or change Liter's pinned decoder contract. |

## Behavior: MCP outcome and receipt preservation

| Field | Required declaration |
|---|---|
| Canonical identity | `uar-outstanding-issues-v1-1-1` / R4 / `tool-call-protocol` |
| Source identity | The v1.1.1 candidate tree selected by the lead; source and MCP fixture identities are recorded in the receipt. |
| Production entry point | Authenticated `POST /api/uar/runs` that invokes a configured MCP descriptor, then `GET /api/uar/runs/{id}/stream` and the persisted tool/admission receipt path. |
| Real collaborators | UAR server, `RunManager`, trusted-host tool admission and receipt persistence, configured stdio or HTTP MCP peer, configured provider wire peer, and durable store. |
| Boundary exercised | Runtime admission, real MCP application/transport response, ToolEnd emission, durable receipt persistence, and later independent run admission. |
| Observable result | `isError=true`, `isError=false`, omitted `isError`, protocol error, timeout, and uncertain disconnect preserve distinct typed outcomes; later recovery does not rewrite the original receipt. |
| Negative control | A disconnected MCP effect with no definitive result remains unknown and blocks automatic replay; a native tool payload with an incidental `isError` field does not acquire MCP semantics. |
| Isolation | Runner-owned MCP processes or ports, trace/disconnect controls, scratch persistence, unique run identities, and a disposable evidence root. The runner owns fixture shutdown. |
| Prerequisites | Completed R4 implementation, a controlled real MCP peer, authenticated caller, a configured provider peer, and persistent receipt storage. |
| Final local gate | The R5 acceptance manifest includes this MCP matrix in the single completed-phase integration invocation. |
| Evidence | Pending: MCP trace, event/receipt snapshots with secret-safe redaction, persisted original and recovery outcomes, candidate identity, exit code, and receipt. |
| Limitations | The fixture cannot prove an external effect never occurred; it proves the runtime retains uncertainty and does not auto-replay it. |

## R5 fixture and documentation inventory

R5 SHALL deliver the real-path fixture matrix and operator documentation required by R1 through R4 before the phase verification boundary opens. Fixture coverage SHALL include the R1 refusal and qualified controls, R2 EOF and cancellation controls, R3 newline/partition/known-and-unknown event matrix, and R4 MCP application/transport/uncertainty matrix. Documentation SHALL state durable-attestation prerequisites, preserve #384 inherited timeout/retry configuration, and record sanitized skill troubleshooting by resolved root/path/error without changing the pinned skill-system submodule.

The fixture and documentation work preserves captured authority, reservation ownership, checkpoint and tool-result invariants. It does not add a kernel write path, infer durability from a remote endpoint, loosen a budget, replay an uncertain tool, or alter pinned libraries.

## Proposed final gate — lead-owned future task L3

The following is a **new proposed** final integration gate. It is selected and run by the lead only after the lead records the complete production implementation inventory. The runner is a future L3 deliverable: it does not exist at this stage and was not run.

```text
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/arm64.json
python3 scripts/release/uar_1_1_1_acceptance.py --manifest /Users/gqadonis/Projects/prometheus/universal-agent-runtime/.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/amd64.json
```

`local.json` is the completed-production local integration manifest. `arm64.json` and `amd64.json` select the corresponding published immutable image digests. Each manifest and receipt SHALL record candidate or published digest, exact source/fixture identities, isolated resources, negative controls, observed output, and exit code. A missing runner, manifest, prerequisite, isolation boundary, or receipt is BLOCKED, not a passing gate. Historical v1.1.0 receipts remain historical evidence only and do not certify v1.1.1.


## Function-first runtime acceptance amendment — 2026-10-17

This amendment changes release observations, not the production framing/terminal
contract. Child FC2 retains eight binding, six stream, four MCP and one combined
labels. Fold qualified durable control into the real 16k refusal journey and move
distinct-provider messages-facade assertions into credentials via handoff. Keep
successful settlement/restart, exact refusals and zero denied effects. Four local
native supplements remain separate from image evidence.

Replace exhaustive release partition/field loops with public prompt plus explicit
enabled-skill attachment. A controlled provider derives the exact answer from
both unique input and overlay; assembled SSE, AG-UI and persisted answer text must
each equal it exactly. A second prompt changes the answer. A disabled eligible
attachment reports `disabled` and omits its overlay/unique dependencies while
ordinary prompting may continue. Do not widen policy or assert zero contact.
Retain representative text/pending-tool EOF, malformed data, overflow, invalid
UTF-8, post-terminal and valid unknown-event outcomes, tool success, cancellation
and approved graph delegation. Preserve MCP failure/uncertainty no-replay and
immutable receipts; only repeated successful/omitted restart controls are removed.
Combined later success stays a distinct receipt with the original failure intact.

The opt-in observer measures first parsed content during delivery, not buffered
EOF. Client/peer durations respect clock domains and disclose uncertainty; operator
timing disposition is required before acceptance. No model-selected skill quality,
real-provider latency or measured speedup is claimed. Existing parser regression
sources remain; none has been run for this child.

Current local gate is `function-first-local.json` under parent `evidence/gates/`;
published gates are `function-first-arm64.json` then `function-first-amd64.json`.
Use the exact commands in child plan/verification.md only after whole-phase
implementation, fresh same-image qualified preparation, explicit review and gate
instructions. Local keeps all six groups; published runs streams/recovery only.
The uncomfortable loss is no direct per-child binding or MCP-failure re-observation.
