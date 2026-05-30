# Scope — Credentials Admin UI (BYO provider keys)

- Date: 2026-05-30
- Author: claude-code (F4 follow-up of `uar-wisc-cli`)
- Status: **SCOPED — recommend a dedicated phase, not inline work**

## Why this is its own phase (not done in this follow-up)

CLAUDE.md mandates a **UI/UX work-routing protocol** that must run *before any UI
code is written* (memory recall → UI/UX Pro Max analysis → `/impeccable
audit|critique|polish` → Anthropic `frontend-design`/`ux-designer` → Vercel React
skills → web-search for the relevant patterns → a written distillation). That
protocol is the entry gate for the implementation phase; doing it justice — plus
the build itself — exceeds a backend follow-up and needs its own phase with the
routing performed up front. Blind-coding a credential UI now would violate the
repo's own rules and risk rework.

## What the UI is

A small authenticated surface where an end user manages their own provider API
keys against the already-shipped REST API:

| Endpoint (live) | UI action |
|-----------------|-----------|
| `GET /api/uar/credentials` | List the caller's keys (masked: provider, last-4 hint, updated_at) |
| `PUT /api/uar/credentials/{provider}` | Add or rotate a key (raw key entered once, never echoed back) |
| `DELETE /api/uar/credentials/{provider}` | Remove a key |

Plus read-only context from the existing `GET /api/uar/providers` (which
providers exist + their models) to populate the "add key" provider picker.

## Stack constraints (from CLAUDE.md)

- **HTML-first**: HTMX 2.x for server interaction + a Web Component for the panel;
  Alpine.js for local-only UI state. No SPA framework, **no CDN scripts** (all
  assets served locally), Tauri-compatible (web/desktop/mobile identical).
- **No API keys in the browser** beyond the single write moment; reads are masked
  server-side (already enforced — handler returns `CredentialView`, never
  plaintext/ciphertext).
- SSE-friendly: list refresh after mutation via HTMX swap, not client state.

## Proposed surface (for the implementation phase to refine post-routing)

1. **Credentials panel** (`<credentials-manager>` web component) — table of the
   caller's keys: provider, masked hint (`••••1234`), updated_at, row actions
   (Rotate, Delete).
2. **Add/Rotate sheet** — provider `<select>` (from `/api/uar/providers`), a
   single password-type key input, submit → `PUT`. On success, HTMX swaps the
   table; the raw value is cleared and never re-rendered.
3. **Empty / disabled states** — "multi-tenant credentials not enabled" when the
   server returns `503` (no `CREDENTIAL_ENCRYPTION_KEY`); "no keys yet" otherwise.
4. **Auth** — panel only mounts for authenticated sessions; all calls carry the
   JWT the rest of the app already uses. Anonymous → the panel is hidden.

## Security/UX guardrails (non-negotiable in implementation)

- Never display, log, or round-trip a full key. Show only the last-4 hint.
- Key input is `type=password`, `autocomplete="off"`, cleared on submit/cancel.
- Destructive delete behind a confirm affordance.
- Surface the `503` (service disabled) and `401` (unauth) states explicitly — do
  not present an empty table as "you have no keys" when the service is off.

## Recommended next step

Open a dedicated phase, e.g. **`uar-credentials-admin-ui`**, whose first task is
the CLAUDE.md UI/UX routing protocol (producing the one-paragraph distillation
that becomes the implementation prompt context), then an OpenSpec change for the
component + HTMX wiring + integration test against the live endpoints.

This document is the pre-scope; it is **not** a substitute for the routing pass.
