# FC2 handoff — runtime adapter and controlled provider

Date: 2026-10-17. Task: child `2.1` / FC2, Execute. Gate: **CLOSED**.

## Native `/v1/messages` fold contract for FC3

FC2 exports `observe_native_messages_policy(context)` from
`scripts/release/uar_1_1_1_runtime.py`. It owns the controlled peer and native
Anthropic source observation. FC3 imports and invokes this function while
implementing its retained `messages-facade-policy` scenario.

For each `allowed` value, the helper starts FC2's private controlled provider,
uses an Anthropic-native configuration, and sets shared provider fallbacks to
either no grant or the exact `anthropic` grant. It sends the existing direct
request through `POST /v1/messages` with model
`anthropic/claude-haiku-4-5-20251001`, `max_tokens: 32`, `stream: true`, and
one user message. It asserts peer-contact count equals `int(allowed)`.

The allowed control requires a complete native response containing the fixture
text and `message_stop`; the observed peer path must end in `/v1/messages`.
The denied control requires HTTP 403 and exactly
`error.type == "invalid_request_error"` plus
`error.message == "credential_source_denied"`, with zero peer contact.
The return value contains the allowed and denied status, contact counts, peer
paths, and completion outcome for FC3's receipt. FC3 retains this helper's
native observations alongside its own distinct OpenAI facade route assertions.

## Completion state

FC2 implementation is complete for its assigned adapter, support helper and
controlled peer files. The runtime scenario inventory is 19 labels: binding 8,
streams 6, MCP outcomes 4 and combined 1. The removed labels are folded as
specified: qualified durable evidence is inside
`binding-16000-catalog-reservation-refusal`; native messages evidence is owned
by the helper above for FC3's `messages-facade-policy`; partition labels are
replaced by `streams-prompt-skill-output-timing`; and the valid combined control
is retained within `combined-failed-tool-incomplete-stream`.

## Changed files

- `scripts/release/uar_1_1_1_runtime.py`
  - Creates enabled and disabled skills through the documented Admin/Live
    skills surface using the phase-host admin client. It retains the owner user
    client for every `POST /api/uar/runs` admission and subsequent run read.
  - Replaces the partition-matrix journey with public enabled and disabled
    attachment observations. The enabled path requires a provider request with
    the unique prompt and overlay, then exact controlled-answer equality across
    legacy SSE, AG-UI, durable event readback and post-restart run state. The
    second distinct-prompt control explicitly sends the same `skill_attachments`
    value, so its marker-derived answer still proves attachment activation.
  - The disabled eligible path requires the exact `disabled` activation failure,
    excludes its overlay and unique preferred-tool marker from the provider
    request, and still requires one ordinary provider contact.
  - Records client and peer timestamps and only computes durations within each
    clock domain. The receipt explicitly retains client, network and polling
    uncertainty rather than inferring runtime overhead from independent clocks.
  - Shrinks stream cases while preserving EOF text, EOF pending tool, malformed
    data, overflow, invalid UTF-8, post-terminal data and valid unknown-event
    semantics. It removes successful and omitted MCP restart/later-success
    repetitions, folds the 16k positive/restart control, and folds combined
    valid recovery into the failed-tool journey.
  - Exports `observe_native_messages_policy(context)` for FC3.
  - Preserves raw public SSE bytes across observer callbacks and recognizes a
    complete `CRLF`, `LF`, or `CR` frame delimiter before parsing; split CRLF
    callbacks cannot create a false first-content timestamp.
- `scripts/release/uar_1_1_1_support.py`
  - Adds the optional `Http.stream(..., on_text=...)` incremental strict UTF-8
    callback used to timestamp the first parsed public content delta while
    retaining the finite whole-stream deadline and raw partial-byte error data.
    Any failure while an observer callback is installed carries accumulated raw
    bytes on its `partial` attribute before cleanup and re-raise.
- `tests/fixtures/uar_1_1_1_provider.py`
  - Makes `prompt_skill` output a deterministic SHA-256-derived answer only
    when both received request markers are present. It also writes correlated
    peer receive, first-write and last-write timestamps to the private trace.
- `.kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/children/function-first-release-certification/handoffs/FC2.md`
  - Records this interface and source-only handoff.

## Interfaces for adjacent writers

`observe_native_messages_policy(context)` is a public module-level Python
helper. Its return value is a mapping keyed by `"False"` and `"True"`; each
value has `status`, `provider_calls`, `peer_paths` and `response_complete`.
It owns provider startup and cleanup. FC3 may import it as:

```python
from uar_1_1_1_runtime import observe_native_messages_policy
```

The helper's enforced source observations are the native contract in the first
section of this handoff. FC3 should place the returned mapping under its
distinct facade scenario receipt; it must retain its own OpenAI facade route
assertions.

`Http.stream` now accepts the keyword-only optional `on_text` callback. Existing
callers without it preserve their former call shape. The callback receives each
strictly decoded UTF-8 text segment as it is observed by the HTTP reader. A
callback-path exception exposes all accumulated raw stream bytes as `partial`.

The creation request matches `CreateSkillRequest` (`name`, `description`,
`prompt_overlay`, `preferred_tools`, `enabled`), and the response observations
use `SkillResponse` fields `skill_id`, `enabled` and `prompt_overlay`. The run
request carries `skill_attachments`, and its response carries
`activation_failures` from the durable run context.

## Source state and unverified limits

Only source inspection and the authorized edits above were performed. No
product/runtime source file was edited. The controlled peer's synthetic timing
is diagnostic: it cannot certify model quality, external provider latency or
runtime-only overhead. The public route shapes, full exact failure payloads,
SSE event sequence, durable restart behavior and all scenario outcomes remain
unverified until the operator opens the whole-phase gate.

No verification commands were run. In particular, no test, lint, format,
typecheck, compiler/import probe, build, acceptance command, external gate or
`git diff --check` was run because the operator lock is CLOSED. Source reads
used `sed` and `rg`; edits used `apply_patch`.

No additional unrequested file was changed by FC2. The remaining decision is
operator gate opening and the later whole-phase verification inventory; FC2 has
no pending implementation decision in its assigned files.
