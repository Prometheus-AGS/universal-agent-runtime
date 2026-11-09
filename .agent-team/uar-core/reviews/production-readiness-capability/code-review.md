# Capability dispatch code review

Scope: cumulative source diff against `629f5702`, plus new `src/llm/inline_image.rs` and the `carry-inline-images-and-constrain-dispatch` specification deltas. Phase: `production-readiness-476-capability-dispatch`, Execute / implementation complete, acceptance pending.

This is an independent fresh-context, same-family native source review. It is weaker independence than a cross-family review. Producer handoffs, generation conversation and self-assessments were not consulted. Only this review file was written; production source was not edited.

## Findings

### [HIGH] Undefined formatter variable prevents compilation

File: `src/llm/mod.rs:263`.

Exact source:

```rust
fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
```

```rust
f.debug_struct("ImageUrl")
```

Failure scenario: any build compiling this module encounters a reference to `f`, although the argument is named `formatter` and no local `f` exists. This unconditional implementation is not protected by a feature gate, input guard or type narrowing. Rust name resolution therefore cannot resolve this expression. This blocks delivery of all affected build profiles before runtime behavior can be exercised.

Minimal correction: call `formatter.debug_struct("ImageUrl")`.

Evidence / falsifier: static lexical inspection of the complete implementation at lines 242–269 establishes the missing binding. A later authorized compiler gate should confirm compilation after correction; no compiler command was executed in this review, and no compiler diagnostic is claimed.

### [MEDIUM] Context-only requirements can resurrect an ineligible runtime registration through catalog metadata

File: `src/llm/router.rs:319`; related new predicate at `src/llm/router.rs:685`.

The catalog supplementation guard calls `requirements.has_hard_capability_requirement()`, but that predicate includes only tools, reasoning, vision and structured output. It omits `min_context`.

Concrete static reproducer: choose an enabled model represented in the embedded catalog, with catalog context at least 8192, and register that same provider/model with `context_window: Some(4096)`. Request only `min_context: Some(8192)` and prefer that identity. The runtime candidate loop constructs limits from the 4096 registration and filters it out. The catalog loop then sees no surviving duplicate, finds the registration enabled, treats the request as having no hard requirement, and adds the catalog copy at line 324. Preferred selection can return that identity. In contrast, `evaluate_destination` uses the live registered 4096 window and rejects the same 8192 requirement.

This contradicts the added shared candidate-admission requirement that preferences/catalog supplementation cannot override hard context requirements. It is an advisory-routing mismatch; final admission still refuses the illustrated destination, so this finding does not claim that an oversized provider call succeeds.

Minimal correction: prevent catalog supplementation from certifying `min_context` when runtime metadata rejected or did not establish that minimum. Including `min_context.is_some()` in the hard-requirement predicate is the smallest consistent correction; assess its no-registry admission use as part of that correction.

Evidence / falsifier: the runtime construction/filter is at lines 240–280, catalog supplementation at 291–324, preferred selection at 337–347, and live context admission at 442–448. An authorized later regression scenario with the state above should return no route rather than the preferred model. It was NOT_EXERCISED here.

## Inspection and limits

Commands were source inspection only: `pwd`, `git status --short`, `git diff --stat 629f5702`, `git diff --staged --stat`, scoped `git diff 629f5702 -- ...`, `cat`, `sed`, `rg`, `rg --files`, `head`, and `nl`. They returned the source and diffs supporting the findings. A few bounded reads referenced nonexistent candidate paths and returned ordinary file-not-found output; no tools, packages or checks were launched through those paths.

The review consulted repository operating rules, tool policy, repository map, relevant retained gotchas, Rust workspace/best-practices skill instructions, Rust stack rules, the phase production contract, specification deltas, changed production paths, and related callers/types. The historical Compass graph described in the repository map predates this cumulative change and has material server parse gaps; no graph rebuild or external retrieval was performed. No upstream behavior claim is needed for the two findings.

All tests, checks, lint, formatting, typechecking, Cargo/compiler execution, builds, packaging, Docker/services and runtime/provider calls: **NOT_EXERCISED**. No acceptance or runtime certification is implied. Decoder behavior/allocation, actual image wire identity, retry/fallback behavior, checkpoint continuation and existing identity/approval/budget/replay/uncertain-effect regressions remain for the lead's explicitly authorized serialized production gates. Skill commands that would execute verification were deferred under the task's source-only boundary.

No unsolicited production changes or guards were added. The only added artifact is this requested review. Existing guards were inspected as source; none are certified by this review.

## Review Summary

| Severity | Count | Status |
|----------|-------|--------|
| CRITICAL | 0 | pass |
| HIGH | 1 | warn |
| MEDIUM | 1 | info |
| LOW | 0 | note |

Verdict: WARNING — resolve the HIGH compile blocker before the authorized build gate; correct the context-admission mismatch before claiming the shared admission contract complete.
