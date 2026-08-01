# Embedded run-policy resolution + embedded admin surface

## Why

The embedded runtime (`EmbeddedRuntime`, consumed by host apps that link UAR
in-process — mobile and desktop shells with no HTTP service) resolves run policy
through `resolve_legacy_run_policy`, which considers only the agent artifact and
per-conversation policy. It never reads the global `run_policy.global` setting,
so the **global default model/policy tier is silently ignored on every embedded
host**. The `SettingsManager` and the full `resolve_effective_run_policy`
(Global → Agent → Conversation → Turn) already exist but are wired only into the
HTTP/service path (`api/discovery.rs`). Additionally, the embedded `Runtime`
exposes no settings/agent administration surface, so an embedded host cannot
read or write settings (e.g. `run_policy.global`) or manage agent definitions
without standing up the HTTP service — defeating the point of embedding.

This blocks first-party embedded hosts (and any future app linking UAR) from
offering the same three-tier model selection (global default → per-agent →
per-conversation) and settings/agent administration that the service path
already provides. The capability belongs in the runtime library so every
embedding app inherits it, rather than each app re-implementing policy storage
and resolution against its own seam.

## What Changes

- Wire the embedded `RunManager` to honor the **global** policy scope: construct
  a `SettingsManager` from the embedded persistence layer (already passed to
  `RunManager::new`) and resolve run policy via `resolve_effective_run_policy`
  (Global → Agent → Conversation → Turn), so `run_policy.global` is respected on
  embedded hosts exactly as on the service path. `resolve_legacy_run_policy`
  remains as a fallback when no settings manager is available (no behavior change
  for callers that don't opt in).
- Expose an **embedded administration surface** on the SDK `Runtime`: read/write
  typed settings (get/set by key, incl. `run_policy.global`) and agent-definition
  CRUD (list/get/create/update/delete), delegating to the same `SettingsManager`
  and persistence the runtime already owns. This lets an embedded host implement
  its control-plane admin without an HTTP service.
- No change to the HTTP/service resolution path or its public contract; the
  embedded path gains the behavior the service path already has.

## Capabilities

### New Capabilities

- `embedded-admin-surface` — settings + agent administration exposed from the
  embedded SDK `Runtime` (get/set typed settings, agent CRUD), backed by the
  runtime's own persistence + `SettingsManager`.

### Modified Capabilities

- `run-policy-resolution` — the embedded runtime's policy resolution now includes
  the Global scope (via `SettingsManager` + `resolve_effective_run_policy`),
  matching the service path. (If no existing spec named `run-policy-resolution`
  is present under `openspec/specs/`, this is introduced as a new capability spec
  describing embedded + service resolution parity.)

## Impact

- **Runtime UX:** embedded hosts (mobile, in-process desktop) gain a working
  global-default model/policy tier and can administer settings + agents
  in-process — enabling three-tier model selection and admin UIs with no HTTP
  service. No change for service-mode consumers.
- **Provider compatibility:** unchanged — provider/model routing still flows
  through the existing registry and `resolve_to_llm_config`. This only changes
  which *scope* supplies the `ModelRoute`, not how a model is called.
- **Realtime state:** the per-run `effective_run_policy` artifact already emitted
  at run start now reflects the resolved Global scope on embedded hosts too, so
  provenance/telemetry is consistent across surfaces.
- **KBD workflow state:** yes — record under the active
  `uar-hybrid-app-architecture` phase; this advances the operator-directed hybrid
  web/Tauri/mobile + admin/model-picker direction. Add an ADR (next number:
  `0013`) documenting the embedded-resolution + embedded-admin decision.
