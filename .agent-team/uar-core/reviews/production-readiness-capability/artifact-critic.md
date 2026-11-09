# Capability-dispatch artifact critique

Phase: `production-readiness-476-capability-dispatch`, Execute; source-only artifact review after the implementation inventory. Runtime acceptance: **NOT_EXERCISED**. This report does not open or satisfy a verification gate.

Reviewed worktree: `/Users/gqadonis/.claude/worktrees/production-readiness-476`. Artifact scope: cumulative production diff from `629f5702`, plus the new `src/llm/inline_image.rs`, applicable OpenSpec deltas/design/tasks, and directly referenced contracts. The source identity here is the inspected working tree, not a separately hashed immutable candidate. Subsequent writer edits require renewed source review.

Isolation: fresh native child context, artifact-only packet. Producer conversations, handoffs, self-assessments and other review reports were not read. Judge: Codex GPT-6-family session; exact producer model identity was not supplied. This is the explicitly requested same-family native fallback, weaker than an independently configured model-family review. It is not approval from the blocked Rust-reviewer workflow.

## CRITICAL — unbound formatter prevents compilation

Evidence: `src/llm/mod.rs:243` names the formatter argument `formatter`; `src/llm/mod.rs:263` invokes `f.debug_struct("ImageUrl")`. No `f` binding exists in this implementation. The new unconditional `Debug for ImageUrl` implementation therefore contains an unresolved identifier in every feature profile that compiles this module. The ordinary image implementation cannot be delivered in this source state.

Basis: direct Rust lexical-scope inspection, not an executed compiler result. Falsifier: show an in-scope `f` binding in this implementation, or update this call to the actual formatter argument. Minimal fix: replace `f.debug_struct` with `formatter.debug_struct`, preserving the payload-free projection. The lead's authorized compile/build gate must later establish compilation; no such gate was run by this critic.

## WARNING — advisory context admission can disagree with dispatch

Evidence: `src/llm/router.rs:241–281` constructs registered runtime candidates using their runtime context window and filters them. Catalog supplementation at `src/llm/router.rs:291–324` can add the same identity when it failed that filter. Its exclusion at line 319 uses `has_hard_capability_requirement()`, whose definition at lines 685–689 omits `min_context`. Final `evaluate_destination` instead rejects missing or insufficient runtime context at lines 440–445.

Concrete falsifying case, not executed: register an enabled model whose identity exists in the embedded catalog, whose runtime `context_window` is `None` or below a requested minimum, and whose catalog window meets that minimum. Route with only `min_context` set and no other hard flags. The runtime candidate fails the first filter, but catalog supplementation can re-add it and advisory routing can return it. Admission for that same destination and requirements rejects `MissingRequiredCapability { capability: "context" }`. This contradicts the OpenSpec shared candidate-admission requirement; the final refusal itself remains fail-closed.

Minimal fix: use the same runtime destination evaluator for advisory context eligibility, including catalog supplementation, rather than letting the catalog restore missing or insufficient runtime context facts. A mere preference change does not resolve the mismatch. Falsifier: source evidence that supplementation cannot reintroduce this identity, or later authorized execution showing identical eligibility decisions for this case after correction.

## Scope limits and evidence

Source inspection traced the semaphore permit and lifecycle lease into the actual blocking decoder closure, cancellation-on-drop and timeout join, HTTP resource-scope propagation, typed ingress parsing and image count/size limits, canonical-history versus generic-data projection, captured adapter facts and inherited binding preservation, finite-accounting image refusal, and primary/retry/fallback/graph admission placement. These are inspected mechanisms, not runtime acceptance. No additional defect is asserted solely because a scenario remains unexecuted.

All shell calls in this task were source inspection (`pwd`, `cat`, `sed`, `rg`, `nl`, `head`, and `git diff` without `--check`) or this authorized documentation write (`mkdir`, shell heredoc). `git diff --stat 629f5702` reported 19 tracked changed files; the new inline-image module was read separately. No tests, lint, format, compiler probes, Cargo, builds, imports, services, Docker, network calls, model effects, or acceptance scripts were started. The Compass snapshot documented in the repository map predates this candidate and was not used as proof of current source behavior.

Machine sycophancy screening: **NOT_EXERCISED** under the packet's no-model-effects/source-only restriction; no score is available. Manual screening removed praise/certification language, retained a concrete critical defect, and separated static deductions from unexecuted falsifying scenarios. The lead may screen this artifact using separately authorized tooling.

Only this report was written; production source and other teammates' files were not edited. No production guards or unrequested implementation were added. Compilation across feature profiles, decoder cancellation/resource behavior, actual protected-history restoration, primary/retry/fallback/graph sends, zero-send refusals, finite-accounting behavior and warning-free customer artifact delivery remain unverified because this task authorizes source inspection only. Genuine-model vision remains unqualified. Certification is withheld with the critical defect unresolved.
