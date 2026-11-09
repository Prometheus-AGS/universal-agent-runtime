# Capability ingress implementation handoff

Phase: `production-readiness-476-capability-dispatch`, Execute.
Worktree: `/Users/gqadonis/.claude/worktrees/production-readiness-476`.
Verification: **CLOSED**. Source inspection and authorized implementation only.
No test, Cargo, compiler, lint, formatter, build, review gate, `git diff --check`,
external mutation or paid call was run. This is an authored-source handoff, not
acceptance. Lead must integrate all nine implementation items and reopen only
under the applicable operator gate instruction. Any new edit closes the gate.

## Source-grounded reproducer before implementation

The prior `src/server.rs::extract_input_message` called `extract_text_content`
for the last user message. That function retained only `type=text` parts.
A current turn containing text/PNG/text/JPEG therefore became text-only; an
image-only current turn was rejected as empty. `build_multipart_content`
serialized image attachments and text parts as a JSON **String**, which the
runtime consumed as text. This finding came from source, not an executed request.
The source-grounded reproducer was sent to the lead before editing.

## Exact owned changes

1. `src/server.rs`
   - Replaced text extraction/stringified attachment helpers with
     `parse_current_chat_content` and `admit_current_chat_content`.
   - Strict manual part grammar: string or array of `{type:"text",text}` and
     `{type:"image_url",image_url:{url,detail?}}`; unknown/missing keys and null
     detail refuse with fixed diagnostics. JSON-looking strings remain text.
   - Last current user content is admitted. Earlier/non-current arrays must be
     text-only; unsupported/ignored multimodal history refuses. Legacy nonblank
     convenience `message` keeps precedence over all-text `messages`, and blank
     convenience text falls back to the last current user. When convenience
     precedence would discard an image or unsupported part, the request refuses
     as ambiguous. This compatibility rule was grounded by source inspection of
     `git show 629f5702:src/server.rs` (the old `extract_input_message`).
   - Image attachments refuse; text attachments contribute `[filename]\ntext`
     before the original ordered current content. Caller-owned tool multipart
     remains explicitly refused by the existing caller-tool admission path.
   - Full ordered text is separate from typed content, including every text part
     and attachment text, for guardrails, memory and run context. Image-only is
     valid. The candidate host request calls
     `with_admitted_current_user_content(MessageContent)` before execution.
   - Image count/aggregate encoded bound are checked before any decoder work.
     Decoder failures occur before guardrails, target/model selection, memory or
     run creation. Busy/unavailable decoder returns 503; invalid image returns
     400. Original existing HTTP envelope remains 10 MiB, unchanged.
   - Removed raw `Debug` derives from `OpenAiMessageInput` and
     `ChatCompletionRequest`; admission debug events contain MIME and byte
     counts only, never URLs/bytes. Existing native Anthropic path is unchanged.
2. `src/llm/inline_image.rs` (new)
   - `MAX_INLINE_IMAGE_BYTES = 10 * 1024 * 1024` caps one URL; ingress also caps
     the sum of image URL lengths. This is additional internal admission
     protection, not a replacement for the whole-body HTTP envelope.
   - `MAX_INLINE_IMAGES = 8`, checked before decoding.
   - `inline_image_metadata(&ImageUrl) -> Result<InlineImageMetadata,
     InlineImageError>` checks strict data URL grammar, supported detail,
     canonical STANDARD base64, byte bound and PNG/JPEG signature. It does
     **not** establish full decoder acceptance or generic redaction authority.
   - `validate_inline_image(&ImageUrl)` has the same result and additionally
     performs actual xberg decoder admission under `document-intelligence`.
     Other builds explicitly return `image_decoder_unavailable`.
   - `InlineImageMetadata` fields: `mime_type: &'static str`,
     `encoded_bytes: usize` (whole URL length), `decoded_bytes: usize`
     (decoded encoded-file bytes, not pixels/provider tokens).
   - `InlineImageError` fields: `code: &'static str`, `message: &'static str`.
     Debug/Display contain static diagnostics only, never library errors/input.
   - Decoder uses direct pinned base64 0.23 and xberg's actual public
     `ExtractInput::from_bytes`/`extract`, no new dependencies or pin changes.
     MIME detection plus exactly one result/no errors and PNG/JPEG decoded
     dimension/format metadata are required. Signature alone cannot admit.
   - Fresh extraction config: `disable_ocr=true`, `use_cache=false`,
     `enable_quality_processing=false`, `postprocessor.enabled=false`,
     explicit `SecurityLimits.max_content_size=100*1024*1024`, one extraction,
     ten-second budget and cooperative cancellation token. Other processing
     remains unset; no filename/URI/operator config/OCR/network path.
   - Global semaphore admits at most two workers without an awaiting queue.
     `runtime::lifecycle::spawn_blocking` moves the permit and actual lease into
     the worker. HTTP middleware installs the existing host lifecycle scope.
     Drop requests xberg cancellation; timeout requests cancellation, aborts
     queued work and joins any running work before reporting timeout. A dropped
     HTTP waiter cannot release the worker's permit/lease prematurely.
3. `.agent-team/uar-core/production-readiness-capability-ingress-handoff.md`
   - This handoff; no other source, manifest, pin, vendor or history file edited.

## Consumer interfaces and coordinated ownership

- Provider owner exports `pub(crate) mod inline_image;` in `src/llm/mod.rs` and
  provides payload-free `ImageUrl` Debug (including unsupported detail values).
- Runtime owner supplies `with_admitted_current_user_content(MessageContent)`;
  all three append paths preserve it. Runtime uses the synchronous syntax helper
  only for already-admitted canonical history projection; no async re-decode in
  projection/checkpoint code. Generic secret projection remains unchanged.
- No image URLs registered as streaming scrubber secrets; that could require
  retaining arbitrary response content while waiting on a huge prefix.
- Existing trusted identity, target binding, host tool approval and resource
  scope admission stay in place. No kernel mutation path is introduced.

## Error codes for behavior authoring

`invalid_content_part`, `ambiguous_message_input`,
`unsupported_image_attachment`, `unsupported_multimodal_history`,
`too_many_images`, `image_too_large`, `invalid_image_detail`,
`unsupported_image_source`, `invalid_image_base64`, `image_mime_mismatch`,
`invalid_image_data`, `image_decoder_busy`, `image_decoder_unavailable`,
`image_decode_timeout`. Existing client-tool failures retain their codes.

## Evidence, guard rationale and limits

Commands used were bounded `cat`, `sed`, `rg`, and the lead-requested
`git show 629f5702:src/server.rs` source reads plus quoted heredoc / Python file writes with the explicit worktree directory. Inspected exact pinned
xberg source under `~/.cargo/git/checkouts/xberg-38dfa5019adc2f59/ffb3b1a/`:
public extraction/config/types, image extractor and metadata decoder,
`image_decode.rs`, cancellation and pipeline instrumentation. Source shows the
PNG/JPEG metadata path invokes `reader.decode()` with decoded-plus-encoded live
allocation limits before OCR-disabled return. Local base64 source shows STANDARD
requires canonical padding and disallows trailing bits. These are source facts,
not runtime proof. Repository graph coverage is stale and server.rs was a known
parse failure; no graph result is used as acceptance evidence.

All new guards trace to the requested public image trust boundary: prevent
silent part loss, unbounded repeated decoding/allocation, unsupported URL access,
invalid decoder admission and diagnostic disclosure. No speculative adjacent
hardening, dependency changes or extra product features were added.

Uncomfortable limits: xberg is a document pipeline, not a minimal dedicated
codec API. Acceptance means its bounded decoder accepted the bytes, not strict
format conformance, malware scanning, absence of tolerated trailing data or
agreement with all remote decoders. A synchronous codec cannot be forcibly
preempted: ten seconds requests cancellation and the timeout response waits for
actual completion; HTTP drop retains work until it ends. The 100 MiB limit is
per decoder operation, not total process RSS; two workers plus bounded encoded
copies and metadata use additional memory. No genuine-model understanding or
provider capacity claim follows from admission.

Compilation (including feature-gated profiles), clean warnings, concurrency /
cancellation timing, malformed-image behavior, public wire preservation,
history restoration, diagnostics and no-send refusal remain **unverified**.
The lead-owned public behavior fixture and serialized production gate must prove
those claims after the complete phase inventory. Independent review is deferred
with the closed gate, not reported as passed.
