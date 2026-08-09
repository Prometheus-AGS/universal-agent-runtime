# Analysis — SKIPPED (deliberately)

Phase: **uar-spec-conformance-2026-08**. Written 2026-08-09.

## Verdict

**No external research is required for this phase.** Analyze is skipped, and this
file records why so the Spec gate passes by decision rather than by drift.

## Why

The KBD Analyze phase exists to turn assessment gaps into researched
build-vs-adopt decisions. This phase has no build-vs-adopt decision to make:

| What the phase needs | Status |
|---|---|
| A test harness that boots a real server | **Exists** — `tests/integration/live/harness.rs` |
| An LLM stub requiring no keys or spend | **Exists** — `tests/integration/live/stub_llm.rs` |
| A capability case pattern to extend | **Exists** — `capability_cases.rs`, 20 tests |
| Serial-execution discipline | **Exists** — `serial_test`, pinned 4.0.1 in `Cargo.toml` |
| A CI runner for the matrix | **Exists** — GitHub Actions, already runs the compile gate |

Verified 2026-08-09: `cargo check --locked --no-default-features --features
server-full --test integration` exits 0 in 1m54s, and the matrix ran to
completion in 194.70s.

The phase's five changes are: run an existing instrument, correct two of its
assertions, wire an existing runner to an existing test, extend an existing case
file, and add a shutdown hook to code we own. **None of these is a library
selection.** Running the tiered pipeline would burn its 20-minute budget
producing candidates for problems this phase does not have.

## The one genuine research question, and where it goes

`C-05(a)` — a shutdown hook on `start_server` plus a harness seam permitting
write→reboot→read against the same DB path — touches the boot path and the
SurrealDB lifecycle. Whether that is a clean `oneshot` + `graceful_shutdown`
seam or a refactor of connection ownership is a real design question.

It is **not** a library question. Axum and SurrealDB are already chosen and
pinned; nothing is being adopted. It is scoped separately from the test handoff
(see `goals.md` → Ownership) precisely because it is runtime work, and it should
be answered by reading `server.rs`, not by searching for candidates.

## Scope requests routed elsewhere

Four requests arrived 2026-08-09 that are **not** measurement and are
deliberately not folded into this phase:

| Request | Routed to | Why not here |
|---|---|---|
| UAR documentation — features, APIs, SDKs, theory, UAR branding | `uar-docs-and-pages-2026-08` | A deliverable, not a measurement. Genuinely needs Analyze: SSG selection, rustdoc extraction, SDK doc generation, branding reuse |
| GitHub Pages built and refreshed by Actions on every change | `uar-docs-and-pages-2026-08` | CI infrastructure paired with the docs deliverable |
| "Cover ALL functionality representing a 1.0 release" | `uar-1-0-readiness` (proposed) | Scope redefinition. `docs/SPECIFICATION.md` already names 39 GAPs; closing them is implementation, and this phase was scoped to measure rather than fix |
| Prometheus skill system auto-included as a base skill set | `uar-1-0-readiness` (proposed) | A new capability absent from the spec. C-07 covers skills as a catalog surface; auto-bundling PSP is new product behaviour — and PSP measured **~41x over the skill-description budget** on 2026-08-09 (2,266 skills, ~163,000 tokens against ~4,000), so "include automatically" has an unsolved technical problem underneath it |

**Folding these in would invalidate the adversarial review this phase's plan
already passed.** MiniMax-M3 and Kimi k3 reviewed a measurement phase,
artifact-only, and returned INSUFFICIENT with six amendments now applied. A
phase that also ships documentation, a website, and a skill-bundling capability
is a different artifact and would need its own review.

`uar-1-0-readiness` additionally **depends on this phase's output**: planning
gap-closure before knowing which capabilities currently work would be planning
against unknowns.

## Open questions carried to Spec

1. **C-05(a) seam shape.** Does `start_server` hold state whose lifecycle
   prevents a clean shutdown hook? Answer by reading `server.rs`, not by
   research. Blocks the L4 result for C-12 and C-13.
2. **C-24 exclusion.** Peer reachability needs two devices. Confirm it is
   published as an exclusion with the reason named, rather than tested weakly.
3. **Codex acceptance bar.** `plan-draft.md` records a re-review gate on Codex
   deliverables. The gate's concrete form — which artifacts, reviewed against
   what — is a Spec-phase decision.

## Adversarial review findings, and what closed them

Vetted artifact-only by MiniMax-M3 on 2026-08-09. Verdict **SUFFICIENT**, no
CRITICAL findings, three WARNINGs. Two were closed by a two-minute registry
check the critic was right to demand:

**W-1 — the skip never compared the custom stub against off-the-shelf mocks.**
Fair, and cheap to close. `cargo search`: `wiremock = "0.6.5"`,
`httpmock = "0.8.3"`. Both are **HTTP-level mocks** — they intercept requests
and return canned responses. `stub_llm.rs` is an **in-process
OpenAI-compatible server**, which is what lets the harness point
`UAR_LLM__BASE_URL` at a real socket and exercise the runtime's own HTTP client,
connection handling, and streaming path. A request-interception mock would move
the test boundary inward and *reduce* what is exercised. Verdict: incumbent
kept, now on evidence rather than inertia.

**W-3 — incumbents listed with no attestation that they were checked.** Also
fair. `cargo search serial_test` returns **4.0.1**, matching the pin exactly, so
the pin is current as of 2026-08-09. Attestation now on the record.

**W-2 — three routing destinations point at `uar-1-0-readiness (proposed)`, a
phase that does not exist.** Upheld and NOT closed. Those requests currently
have no owner, no timeline, and no gate. This is a real hole and it is the
operator's to fill: either the phase gets opened or the requests need another
home. Carried into the handoff rather than quietly dropped.

The two MINOR findings (scope-boundary reasoning, open-question routing) are
recorded and not acted on — both ask for a stronger argument rather than a
different decision.

## Confidence

**High** that no research is needed: every dependency this phase touches is
already present, pinned, and exercised. The instrument ran to completion today.

The risk of skipping is that a better-fitting conformance-testing framework
exists and was not considered. That risk is accepted: replacing a working,
repo-specific harness mid-measurement would reset the baseline this phase's
first change just established, and the reviewers' central complaint was about
evidence quality, not tooling.
