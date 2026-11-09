# Capability dispatch Python source review

Disposition: **Warning**. One MEDIUM compatibility finding; no CRITICAL or HIGH finding established by this source-only review. Runtime state: **NOT_EXERCISED**. Verification gate: **CLOSED**.

Scope: cumulative Python changes from `629f570239654717a59b6a3db5793c9bcd34d846` in `scripts/post_lifecycle/controlled_peer.py`, plus the new `scripts/behaviors/fixtures/capability-dispatch/generate-png.py`. Contract context came from the two OpenSpec delta specifications and the capability-dispatch feature and comparator source. This is a fresh native reviewer context in the same model family; its isolation is weaker than cross-family review.

## Finding

[MEDIUM] New observer changes existing success/failure behavior for null tool metadata

File: `scripts/post_lifecycle/controlled_peer.py:68` and `scripts/post_lifecycle/controlled_peer.py:74`

Issue: `body.get("tools", [])` preserves an explicit JSON null. The subsequent `tools[:64]` raises `TypeError`. Because `chat()` calls the observer at line 295 before its configured mode is handled, `do_POST()` converts this into a 400. Previously, an ordinary text request with `tools: null` reached the configured success response, or the configured `provider_failure` 503; neither path inspected tool metadata. Instrumentation therefore changes the peer behavior it is meant to observe. Both request counters have already advanced, but no observation is appended, so the new capability consumer also fails its arrival/observation equality. That fails visibly rather than falsely certifying a zero-send refusal.

Falsifier to exercise only after the operator opens the gate: compare the baseline and candidate peer responses to the same ordinary text request containing a case marker and `tools: null`, with that case configured first for success and then for provider failure. The source predicts baseline 200/503 versus candidate 400/400. No such request was executed during this review.

Fix: normalize explicit null tool metadata to an empty tool list for observation while retaining the original body for peer response behavior. If malformed non-list tool metadata must be represented, give it an explicit safe invalid-shape observation rather than letting instrumentation silently change the configured mode.

## Source conclusions and limits

- Ordered text and image summaries preserve sequence and hash UTF-8 text or decoded image bytes. Raw text, image URLs, base64, tool definitions and schema bodies do not enter the new observation receipt. Model/case labels and admitted safe scalar settings remain visible by design.
- `chat_arrivals` advances before `read_json()`. The existing whole-body 1 MiB bound therefore cannot turn a peer-side body rejection into zero-send evidence: `changed()` requires the arrival delta to equal the observation delta. It also requires all new observation ordinals to remain in the 128-entry ring. Message and part capture truncation flags are authored, though these statements do not establish runtime coverage of truncation cases.
- Output-cap observations record actual integer `max_tokens` and `max_completion_tokens` fields; they do not substitute the requested or configured ceiling. Existing absent settings remain null rather than manufactured scalar values.
- Control updates retain their existing condition lock and per-case selection. Fixed JSON content is optional and does not replace configured tool-call output. No new defect was established in those paths beyond the observer compatibility finding above.
- The generator constructs PNG signature, IHDR, compressed IDAT and IEND chunks with CRCs. Its oversized image supplies 6200 rows of one filter byte plus 6200 RGB pixels: 115,320,000 decoded pixel bytes. Compression consumes constant zero rows incrementally. No source defect was established in the generator. Asset bytes, decoder acceptance and cross-environment compressed-byte reproducibility remain unverified.

## Commands and disclosure

Only source inspection was performed: `git diff 629f5702 -- '*.py'`, scoped `git diff`, `git show` of the baseline peer, `git status --short`, `git rev-parse HEAD`, `rg`, `cat`, and `nl`/`sed`. The Python diff showed the observer/control changes described above; the generator was untracked and was read directly. The scoped report-file discovery returned no existing Python review file. No Python execution/import, static analyzer, linter, formatter, typechecker, test, build, server, network request, Docker or Cargo command ran. Static-analysis instructions could not be fulfilled under the operator verification lock; no static-analysis result is claimed.

The fixture README was inadvertently included in an ancillary source read and contained producer narrative. It was excluded from the findings and acceptance assessment. No producer handoff, self-assessment file or other review was consulted. This limits the strict artifact-only isolation claim and is disclosed rather than treated as pristine isolation.

Only this review document was added. Production files, fixtures and contracts were not edited. No guards were added. Every behavioral claim above is source reasoning; actual wire behavior, fixture execution and generated-asset acceptance remain NOT_EXERCISED.
