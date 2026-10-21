# Assess — function-first release certification

Phase: `uar-outstanding-issues-v1-1-1 › function-first-release-certification`.
Method: source inspection and two bounded `uar-core` read-only inventories.
No test, check, build, import probe or acceptance gate ran. The parent operator
verification lock remains **CLOSED**. Parent L11 records authored
implementation complete; V2 and P1–P3 remain open.

## User question and release decision

The release certificate should answer whether a user can submit a prompt,
activate an allowed skill, receive correct streamed output, use a tool without
false success, retain durable work across restart, and encounter safe refusals
where the advertised profile forbids an action. Timing should be observed on
the actual prompt/skill path so an unacceptably slow feature can be investigated.
Each mandatory case needs a distinct user-visible failure it can catch.

## Existing program, by source

`openspec/changes/uar-outstanding-issues-v1-1-1/verification.md` selects six
installed-image groups (`binding`, `streams`, `mcp-outcomes`, `credentials`,
`recovery`, `combined`), four native source supplements, then repeats the full
six-group program on each published architecture. The adapter declarations
currently require 80 scenario labels: 23 in `uar_1_1_1_runtime.py`, 48 in
`uar_1_1_1_credentials.py` (nine common plus 13 migration labels on each of
three backends), five in `uar_1_1_1_recovery.py`, and four in
`uar_1_1_1_compiler.py`. Labels are observations within groups, not necessarily
80 separate image builds. `uar_1_1_1_acceptance.py` builds the local image once
per invocation and runs all selected groups; separate local repair commands
repeat that build.

The current `streams` adapter uses a controlled input and checks a fixed
provider sentinel in streamed output. It proves wire and terminal fidelity,
but it does not prove that a unique user prompt reached the provider or that
the answer depends on the prompt. Its artifact sets `policy.skills.prefer` to
an empty list. No required installed-image case proves `activate_skill` on a
real user run. `tests/skill_activation_runtime.rs` has an in-process mock
case, and the separate `harness-skill-activation-quality` change is not
implemented. The release certificate cannot credit either as an installed
functional observation. No prompt/skill latency objective appears in the
current 1.1.1 release or acceptance contract.

## Distinct release risks to preserve

| Journey | Indispensable observed behavior | Current source |
| --- | --- | --- |
| Prompt, skill and response | Public prompt reaches the provider, an allowed skill is activated into the next step, output depends on both, stream terminates and persisted status agrees; record first-token and terminal timing. | New bounded installed-image journey required; current `streams` supplies parts only. |
| Safe #385 run refusal and control | An unattested store or over-budget catalog refuses before provider/tool effects, emits terminal-only safe history; a qualified bounded run succeeds. | `binding` plus its native event-history supplement until the installed case asserts the same history contract. |
| Tool and stream truth | Application error, protocol failure, timeout, uncertain effect without replay, native incidental `isError`, cancellation and incomplete provider stream retain their distinct truthful outcomes and immutable receipts; valid controls succeed. | `mcp-outcomes`, `combined`, native supplement and `streams`. |
| Durable user work | Qualified upload survives restart and remains owner-isolated; legacy rows and recovery proof do not cross owner or authority boundaries. | `recovery`; supported Surreal and PostgreSQL paths are distinct. |
| Credential and compiler authority | A verified principal uses its own credential across direct, fallback, team, graph, summarization and SDK paths; a foreign tenant cannot. #387 denies all compiler REST authoring aliases/operations in remote/shared profiles before mutation while local single-owner use remains. | `credentials`, binding paths, native SDK supplement and compiler adapter. |
| Published artifact usability | Each published architecture starts and completes a representative public prompt/skill/tool path; image identity is recorded. | P2 currently repeats all six groups on both images. |

The qualified supervisor reclaim case is real behavior, not a configuration
check. Removing it from mandatory certification would withdraw that recovery
claim and requires an explicit scope amendment. Likewise, dropping an
advertised backend without changing the release claim is not test reduction.

## Likely excess or misplaced certification work

- The 13 migration cases repeat across all three backends. Common HTTP request
  formatting may be sampled once. Preview and status inventories also encode
  authority, revision and journal decisions; retain backend-specific outcomes
  unless Analyze proves an equivalent production-path observation. Every
  transaction and journal implementation difference needs a source-grounded
  disposition before pruning.
- The compiler adapter repeats a complete operation × alias × principal ×
  spoof matrix across profiles and restart. Retain coverage of both aliases,
  each registered operation, distinct admitted principals and unchanged
  storage, but avoid repeating the entire cross-product after restart.
- Parser partition matrices and metadata status inventories can remain
  development/regression cases without each being a release-blocking
  installed-image scenario, provided a public stream with malformed EOF and
  a valid control still catches the consequential behavior.
- The full six-group program is currently repeated on each separately built
  published architecture. A representative functional journey on each child
  would reduce work, but also leave child-specific persistence, migration and
  recovery faults unobserved. Analyze must state this lost assurance and amend
  the release contract before such a cut. Image/source mismatch remains a
  blocking condition inside the functional command; provenance and platform
  identity are evidence metadata rather than separate product-function gates.
- Standalone identity, Docker and service-readiness preflights are not
  functional certification. The runner may enforce binding and setup inside
  the exact authorized functional command; no separate preflight is implied.

## Assessment gaps and uncomfortable limit

The current suite is large yet misses the user's most direct questions:
prompt-dependent output and real skill activation. Cutting to `streams` alone
would also miss user-visible denied-provider contact, incorrect tool success,
lost credential ownership, #385 durable recovery and #387 advertised-profile
behavior. The reduction must replace broad repetition with complete journeys,
not simply delete every non-prompt case. No current-byte runtime outcome or
latency has been observed under this child.

Analyze must map every existing requirement and scenario ID to a retained
observation, a demonstrably equivalent observation or an explicit release-scope
amendment. Counts of scenario labels do not measure work: stream and compiler
labels expand into large matrices. A controlled provider can measure runtime
overhead and propagation, but cannot certify external model quality or latency.
Analyze must define the prompt/skill oracle, timing boundaries, collaborator
classification and whether any latency threshold is release-blocking.

## Assess adversarial disposition

A fresh-context `uar-verifier` reviewed this artifact from source only. It
confirmed the 80-label arithmetic and raised four warnings: preserve distinct
tool and credential paths, preserve backend-derived migration status behavior,
name the assurance lost by narrowing published-child programs, and define a
material-latency decision rule. The table and conditions above incorporate all
four. No product verification ran. The review was harness-native with producer
model identity unknown; cross-family independence is unverified. See
`review/assess-findings.md` for the evidence and severity record.
