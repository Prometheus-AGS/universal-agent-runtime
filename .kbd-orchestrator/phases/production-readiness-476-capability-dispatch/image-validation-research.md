# Inline image validation preparation

Source inspection only, 2026-10-31. Checkout frozen for #462 acceptance at the
lead-supplied identity `8e0602ee`. No checkout edits, verification commands,
compiler probes, network retrieval, external mutation or paid calls were made.
The only authored artifact is this file. Verification lock remains CLOSED.

## Conclusion

Existing dependencies can provide actual bounded PNG/JPEG decoding in the
`document-intelligence` profile, through the pinned xberg public extraction API.
They do not expose a small public decoder directly to UAR. Base64 decoding and
MIME/signature inspection alone cannot establish that an image is decodable.

Recommend a narrow host-side admission helper: strict inline data-URL/base64 and
MIME validation, then an explicitly configured in-memory xberg extraction with
OCR and other processing disabled. Preserve the original typed image URL only
after this succeeds. Do not fetch a URL, invoke OCR, or reuse operator extraction
settings. If `document-intelligence` is absent, reject image parts explicitly;
do not substitute header-only acceptance or silently drop the image.

The uncomfortable limit: this is a feature-dependent use of a document extraction
pipeline for validation. If inline image support must work in minimal builds, the
existing direct dependency/public-export constraints do not provide a confirmed
full decoder there. That requires an explicit scope decision (for example exposing
a reviewed decoder dependency), not an invented xberg API or a home-grown JPEG
parser. No new dependency is proposed by this preparation.

## Verified local source

Repository root: `/Users/gqadonis/.claude/worktrees/production-readiness-476`.

- `Cargo.toml:584`: direct `base64 = "0.23"`; lock resolves0.23.1.
- `Cargo.toml:597`: optional xberg dependency at exact commit
  `ffb3b1a25b6c14c13858142997dd82a16d5b2f35`, including `ocr` and
  `tokio-runtime`; `document-intelligence` enables xberg at line318 and
  `server-full` includes that feature at line265.
- `Cargo.lock:5579`: `image`0.25.10 is present transitively. It is not a direct
  UAR dependency. Do not write `image::...` in UAR on that basis.
- `src/server.rs:2920`: existing HTTP envelope is
  `DefaultBodyLimit::max(10 * 1024 * 1024)`. Keep this unchanged. The limit applies
  to the entire encoded HTTP body, not10MiB of decoded pixels or per image.
- `src/llm/mod.rs:188`: typed `ContentPart::ImageUrl`; existing URL/detail
  representation can retain the accepted original input.
- `src/uar/file_processing/kreuzberg.rs:60,97,316`: existing UAR examples of
  explicit `ExtractionConfig`, in-memory `ExtractInput::from_bytes`, extraction on
  a blocking worker and `ExtractionResult.results`. Reusing the operator-facing
  extraction configuration would be inappropriate for admission validation.

Pinned upstream source root inspected locally:
`/Users/gqadonis/.cargo/git/checkouts/xberg-38dfa5019adc2f59/ffb3b1a/crates/xberg`.
All xberg behavior below refers only to this materialized pinned source, not a
different release or a recalled API.

- `src/lib.rs:270,276,408`: public `extract`, `ExtractInput`, `ExtractionConfig`,
  `PostProcessorConfig`, `SecurityLimits` and MIME detection exports.
- `src/core/config/extraction/types.rs:193–254`: `ExtractInputKind::Bytes` and
  `ExtractInput::from_bytes(bytes, mime_type, filename)` construct bytes-only input
  with no URI. `ExtractionResult` at line297 has separate `results` and `errors`.
- `src/core/config/extraction/core.rs:38–240`: actual fields include
  `use_cache`, `enable_quality_processing`, `disable_ocr`, `images`,
  `postprocessor`, `security_limits`; defaults at line595 enable cache and quality
  processing and do not disable OCR. Default configuration alone is insufficient.
- `src/core/config/processing.rs:122`: public `PostProcessorConfig.enabled`;
  configure false rather than relying on absent configuration to disable plugins.
- `src/core/extract/mod.rs:23`: public async `extract` delegates to default engine.
- `src/engine/mod.rs:91`: engine extraction uses no-op cache/progress seams by
  default; explicit `use_cache=false` is still required to avoid extraction cache
  behavior and ensure validation is not satisfied by an old cached result.
- `src/extractors/image.rs:3748`: image extraction performs metadata extraction
  before its OCR-disabled early return at line3814.
- `src/extraction/image.rs:779,830`: for ordinary PNG/JPEG, metadata extraction
  calls `decode_standard_image_with_security_limits` before reporting dimensions
  and format. Thus this path does not merely inspect headers.
- `src/extraction/image_decode.rs:312–333`: that decoder probes dimensions,
  validates decoded+encoded live-byte budget, sets `ImageReader` limits and calls
  `reader.decode()`. Budget derives from `SecurityLimits.max_content_size` at
  lines22–32; checked arithmetic rejects degenerate dimensions and overflow.
- `src/extractors/security.rs:28–100`: public `SecurityLimits`, including
  `max_content_size`, whose documented default is100MB per-operation allocation.
- `src/extraction/image.rs:727,751` and `src/extraction/image_decode.rs:243`:
  direct decoder functions are `pub(crate)`. `xberg::image` is preprocessing/DPI
  infrastructure, not a re-export of the external `image` crate; its examined
  resize/normalization helpers are also crate-private.
- `src/core/mime.rs:1483–1510`: public MIME detection uses content inference;
  useful for comparing format against the declared MIME, not proof of full decode.

Base64 source inspected locally:
`/Users/gqadonis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/base64-0.23.1`.
`src/engine/general_purpose/mod.rs:331–343,436` defines the standard engine with
canonical padding and disallowed trailing bits. `src/decode.rs` defines errors for
invalid bytes/padding/last symbols. Use the already available `Engine::decode`
with `general_purpose::STANDARD`; propagate decoding errors.

## Proposed narrow admission sequence

1. At the host OpenAI request normalization boundary, validate each current-turn
   image part before admitting the run, opening a stream or calling a provider.
   Keep typed parts and their order; do not collapse them into text. A malformed
   image part rejects the request rather than disappearing.
2. Accept only the deliberately supported inline forms
   `data:image/png;base64,...` and `data:image/jpeg;base64,...`. Reject remote
   HTTP(S), file and other URI schemes, unsupported MIME, unexpected data-URL
   parameters, missing payload and invalid base64. A narrow canonical grammar
   avoids percent-decoding/alternate-encoding ambiguity. Decide and document
   unpadded-base64 compatibility explicitly; strict STANDARD rejects it.
3. Preserve the existing total10MiB HTTP-body envelope. Bound the encoded field
   before allocating decoded bytes when this helper can also be called without
   HTTP ingress; use checked size arithmetic. Decode the bounded payload once.
   Do not introduce a larger per-image allowance or remote download path.
4. Compare the declared MIME against content detection restricted to PNG/JPEG.
   A PNG/JPEG signature match is a routing check only. Continue into full decode.
5. Under `document-intelligence`, call the verified `ExtractInput::from_bytes`
   and public async `xberg::extract` with a fresh explicit config:
   `disable_ocr=true`, `use_cache=false`, `enable_quality_processing=false`,
   `postprocessor.enabled=false`; no image output, OCR, captioning, classification,
   structured extraction, redaction, embeddings or operator-provided options.
   Do not call `from_uri` or config autodiscovery. Keep work off the request's
   asynchronous executor thread using the repository's existing owned blocking
   extraction pattern rather than a new unowned detached task.
6. Set explicit `SecurityLimits.max_content_size` for decoded/live allocation;
   using the current100MB default as an explicit initial ceiling avoids conflating
   this bound with the10MiB transport envelope. A tighter ceiling is a product
   compatibility decision because highly compressed legitimate images can exceed
   it. Retain checked decoder limits. Limit concurrency using the host's existing
   owned-work model; a per-image bound alone is not an aggregate memory bound.
7. Require exactly one successful extracted document, no extraction errors, and
   decoded format metadata matching the declared PNG/JPEG MIME. Treat any decoder
   or envelope failure as invalid image without returning image bytes/base64 in
   error text or logs. Preserve the original validated URL/detail for the typed
   current turn; discard the temporary decoded pixels/result.
8. Without `document-intelligence`, fail explicitly before provider execution.
   This compatibility limit must be part of the proposed feature contract.

## Limits and next-phase evidence needed

Successful decoder output establishes that the bounded decoder accepted the
image. It is not a claim of strict ISO/PNG conformance, removal of metadata,
rejection of every tolerated truncation/trailing byte/polyglot, malware scanning,
or byte-for-byte equivalence of all provider decoders. Do not call it complete
file validation. Preserve original bytes unless canonical re-encoding becomes a
separately requested behavior. The reviewed public extraction path is larger than
a dedicated decoder; its disabled settings and plugin behavior require focused
implementation review before accepting the feature.

At the next authorized completed-phase boundary, evidence should include valid
PNG/JPEG, invalid base64, MIME mismatch, header-only/truncated/corrupt images,
decoded-size bomb rejection, no network-fetch path, text-only compatibility and
typed current-turn preservation through ordinary aliases. These are proposed
cases only: none was authored or executed here. Source inspection has not proven
compilation, decoder behavior on adversarial inputs, scheduling bounds or runtime
acceptance. Do not reopen #462 or claim this preparation is a passing gate.
