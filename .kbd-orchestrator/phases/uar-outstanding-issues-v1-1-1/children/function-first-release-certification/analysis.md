# Analyze — function-first release certification

Phase: `uar-outstanding-issues-v1-1-1 › function-first-release-certification`.
Gate: **CLOSED**. This is source analysis; no product test or build ran.

## Decision rule

A mandatory release observation must catch a distinct failure in a user action,
shipped public path, durable result, security boundary or advertised recovery
path. A parser permutation or administrative metadata shape belongs in ordinary
regression coverage unless its omission can change such an outcome. Preserve
assertions when consolidating receipt IDs. The six existing adapter groups and
one runner remain; no new dependency or second certification framework is
needed.

## Proposed program and honest loss of assurance

Run all six **reduced** groups once against the local candidate. Run `streams`
and `recovery` against each separately built immutable arm64/amd64 child. The
published pair therefore proves public prompt/skill/answer/stream and durable
owner-qualified restart on both artifacts. It does not directly reprove #385
budget refusal, #387 compiler restriction, credential migration or MCP failure
handling on each child. Those remain local candidate release blockers. The
publication contract must explicitly say this before adopting the reduction;
image/source/platform mismatch still blocks inside the published command.

This changes installed group executions from 18 (six local + six per child) to
10 (six local + two per child), a 44% cut in group executions. It does not by
itself establish a wall-time reduction. The stream all-split matrix is much
larger than one label; pruning it should remove the dominant repeated run count.
Do not claim a measured time saving until an authorized gate records it.

## Complete label disposition

The exact existing 80 IDs are declared in the five release adapters. The table
below groups IDs by their current prefixes and names. `Fold` removes the ID
only after its consequential assertions move to the retained destination.
All other IDs are retained; none is silently skipped.

| Current IDs | Disposition | User-visible reason / destination |
| --- | --- | --- |
| `binding-qualified-durable-control` | Fold into `binding-16000-catalog-reservation-refusal` | Keep adequate-envelope success, provider contact, durable settlement and post-restart successful run next to actual over-budget refusal. |
| `binding-v1-messages-policy` | Fold into `messages-facade-policy` | Preserve direct `/v1/messages` allowed/denied native stream, exact policy error and zero denied contact; existing cases use distinct provider paths. |
| Other eight `binding-*` IDs | Retain | Unattested and invalid-budget refusal, 16k catalog refusal, shared alias/credential-free, fallback, inherited timeout, team source and summary capture each detect a distinct authority/effect failure. |
| `streams-native-partition-matrix`, `streams-sse-fields-fixed-partitions` | Replace with `streams-prompt-skill-output-timing` | Exhaustive newline/BOM/split parsing is regression coverage. New installed journey must prove unique prompt and allowed skill reach provider and determine public output. |
| `streams-eof-matrix`, `streams-invalid-matrix` | Retain, shrink inner cases | Keep text without terminal, pending tool at EOF, malformed known data, overflow, invalid UTF-8, post-terminal data and a valid unknown event. These are distinct semantic outcomes. Drop exhaustive byte splits and duplicate framing permutations, not these outcomes. |
| `streams-tool-terminal-control`, `streams-cancel-cleanup`, `streams-graph-inherited-control` | Retain | Tool-loop success, cancellation precedence and graph delegation are distinct user actions; keep one consequential case each. |
| All four `mcp-*` IDs | Retain, shrink inner repeat | Application failure, true/false/omitted status, protocol/timeout and uncertain effect/no replay remain distinct. Do not restart and rerun later success for every successful/omitted case; preserve restart/immutable receipt on failure and uncertainty. |
| `combined-valid-control` | Fold into `combined-failed-tool-incomplete-stream` | Keep the later valid run, separate successful receipt, original failed receipt and two effects in one journey. |
| All nine common `credentials` IDs | Retain | Verified identity, installation, grants, absent service, decrypt failures, compatibility, quarantine, messages facade and response correction use distinct authority seams. |
| Each backend's `preview-authority-pagination` | Fold into same backend's `legacy-conditional-copy` | Preserve ordinary-user denial, complete keyset pagination/counts and stale cursor refusal; run common malformed query forms once. |
| Each backend's `preview-status-inventory` | Fold into that backend's drift/conflict/rotation/historical rows | Preserve journal conflict, missing/quarantined/source-changed and already-applied statuses and exact HTTP outcomes. |
| Each backend's `secret-redaction` | Fold into same backend's `legacy-conditional-copy` | Preserve public preview/receipt/raw-log exclusions for keys, ciphertext, hints and free-form reason. |
| Other ten migration IDs per local/Surreal/PostgreSQL backend | Retain | Metadata upgrade, legacy and fixed-local copy, revision/mapping drift, destination conflict, concurrent/lost-response replay, rotation/deletion/restart and historical receipt are distinct transaction/owner decisions in three store implementations. |
| `legacy_principal_namespace_cold_readback` | Fold into `attested_upload_restart_owner_isolation` | Keep unchanged legacy inventories and blocked cross-owner retrieve/retry/adopt after both backend cold boots. |
| `qualified_supervisor_reclaim_controls` | Retain | Qualified takeover is advertised behavior with physical exclusion, proof binding, replay denial and unknown-liability preservation. It is not configuration-only. |
| Other three `recovery` IDs | Retain | Unattested refusal, durable owner-isolated upload/restart and joined quiesce each catch separate lost-work/authority failures. |
| `compiler-restart-preserves-existing-resources` | Fold into `compiler-local-single-owner-control` | Keep exact spec/session/report readback and normal operation after denied profile attempts and restart. |
| Other three compiler IDs | Retain, shrink repeated matrix | Check both admitted remote principals, both REST aliases, every registered operation, local shared mode, representative spoofed headers and malformed bodies, exact denial code and unchanged storage. After restart use representative read/write/session denials plus full persistent-state comparison, not the entire cross-product again. |

This removes 16 old IDs, adds one functional ID, and retains 65 required IDs.
The largest reduction is in expanded loops and published repetitions, not
receipt-label arithmetic. Four native source supplements remain because each
has a distinct otherwise unobservable host/in-memory/SDK seam; #385's native
supplement stays until installed event history asserts terminal-only refusal.

## Prompt, skill, answer and timing oracle

Use the existing public `POST /api/uar/skills/` and `POST /api/uar/runs` routes.
Create an enabled skill with a unique `prompt_overlay` marker. Submit a unique
user input and `skill_attachments` with that ID; the host admits it through
`InvokeType::Attachment`. A controlled external provider process must record
both markers in its received request and emit a deterministic answer derived
from both. Assert public SSE and AG-UI output, terminal status, durable readback
and absence of attachment failures for the enabled run. Also submit a disabled
attachment. The existing runtime reports an activation failure and continues
ordinary prompting: require that exact failure, absence of the rejected
overlay and tool dependencies, and no policy widening in the provider request.
Provider contact is permitted for the ordinary prompt. This negative control
prevents a fixture that merely echoes arbitrary prompt text from satisfying
skill activation without changing product behavior. Keep the disabled skill
inside the effective eligible set so its failure code is `disabled`, not
`ineligible`; give it unique dependencies to make their absence observable.

Record monotonic client timestamps before POST, at admission response, first
parsed content delta and terminal-status observation. Record controlled peer
receive, first-write and last-write timestamps. Report their differences and
the residual including client/network/polling uncertainty; do not label the
residual as pure runtime overhead. The existing `Http.stream()` buffers until
EOF, so an opt-in timing reader is required. No numeric release latency claim
is justified by source alone. Timing remains diagnostic; if observed first
content or completion is materially slow relative to the controlled peer,
investigate and record the cause before release. Do not certify a general
"responsive" claim from this fixture. A controlled peer proves application
propagation, not external-model reasoning quality or latency. Explicit
attachment proves the right user-selected skill is activated; model-selected
skill quality remains outside this certificate and must not be claimed.

The two removed stream partition labels currently expand to
`Σ(len(wire) + 1)` installed runs over the six newline/BOM combinations, plus
12 fixed-field runs. Folding combined and migration receipt labels alone saves
no invocation because their assertions already share sequential journeys.
Plan must identify the actual removed requests/restarts separately from label
arithmetic; wall-time savings remain unmeasured.

## Integration and scope consequences

`uar_1_1_1_acceptance.py` currently accepts all six groups or one failed-group
repair; it must gain an exact published two-group inventory. Its exact scenario
equality check must be updated with each adapter's revised IDs. The OpenSpec
verification contract, runtime/trust/state fragments, parent KBD plan/tasks,
operator docs and child scope must agree. A source or fixture edit invalidates
L11; after all child changes, write a new whole-phase inventory with exact
identity and request exact gate authorization. Do not run source-identity,
Docker, readiness or schema preflights as standalone checks while the lock is
closed. The authorized functional command may enforce its own prerequisites.

## Uncomfortable limit

The suite can be smaller and more meaningful, but the proposed published
reduction accepts less architecture-specific behavioral evidence. The user
should see that in the release receipt. A true model skill-selection quality
claim would require a separate measured corpus; it cannot be inferred from one
controlled attachment run. The release must not advertise either unmeasured
external-model latency or untested skill choice as certified.

Analyze requires an independent fresh-context adversarial review before Plan.

## Analyze adversarial disposition

A fresh-context `uar-verifier` found one critical contradiction: a disabled
attachment does not block the ordinary prompt. The oracle above now matches
the existing failure-report-and-continue behavior and requires the disabled
overlay to stay absent. Four warnings required preserving distinct stream
semantics, defining honest timing boundaries, naming compiler principal/spoof
coverage and separating receipt-label cuts from actual operation cuts. All are
incorporated above. See `review/analyze-findings.md`. No product verification
ran; reviewer model-family separation remains unverified.
