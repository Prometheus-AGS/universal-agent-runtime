---
sidebar_position: 3
title: Verify Genuine Inference
description: Confirm that a packaged UAR path reaches a loaded model and returns its response.
source_records:
  - docs/API_CHAT_COMPLETION.md
  - .kbd-orchestrator/phases/perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion/functional-verification.md
current_authority: /docs/providers/inference
---

# Verify genuine inference

## Boundary statement

**Configuration is not inference.** Genuine model inference means a request
enters a supported packaged UAR boundary, reaches the configured provider and a
real loaded model, and returns that model's output through UAR. Catalog rows,
health status, mocked responses, recordings, and hard-coded output cannot prove
that path.

## Prerequisites

- Configure and enable the intended provider and model.
- Ensure the UAR process can reach the provider endpoint and resolve its
  credential without printing that credential.
- Use a fresh, bounded prompt whose returned content can be observed.
- Record the provider, model, packaged boundary, source SHA, checkout/date,
  profile, result, and the claim's limit.

## Packaged API workflow

1. Inspect `GET /v1/models` and confirm the explicit `provider/model` is
   available from a configured provider.
2. Send `POST /v1/chat/completions` or `POST /api/chat/completion` with that
   explicit model and a fresh user message.
3. For a non-streaming request, require HTTP success and assistant content in
   the completion body. For streaming, require model deltas followed by the
   terminal marker; connection alone is insufficient.
4. Retain the returned session identifier only when testing conversation
   continuity. A new request without it begins a new conversation.
5. Report provider/model routing and response completion separately from skills,
   knowledge, agent selection, or other capabilities layered onto the request.

Failures remain visible: invalid model IDs return `404`; malformed input returns
`400`; provider initialization, credential, timeout, and upstream failures do
not become successful completion objects.

## Packaged UI workflow

1. Open Chat in the packaged `server-full` application.
2. Select the intended provider/model directly, or select an agent whose
   effective configuration names it.
3. Start a fresh thread and send a bounded prompt.
4. Confirm the assistant response completes and, when an agent is used, inspect
   the effective run policy rather than assuming the visible selector controlled
   the request.
5. Treat UI rendering as evidence for the UI path only when the network request
   traversed UAR and the returned content came from the configured model.

## Illustrative and non-certifying examples

Examples in prose and request snippets explain the protocol shape. They are
illustrative and non-certifying. Repeating an expected phrase, displaying a
fixture, or receiving a recorded event does not establish provider execution.

## Observed server-full evidence

The following is a reviewed synthesis of the retained 1.0 functional record,
not a fresh test run. The record was created on 2026-08-22 from the
`uar-1-0-readiness` checkout and first committed at source SHA
`d41bf7c3a447869896664d44ac0563e1b4a1d9f3`.

| Provider and model | Packaged boundary | Observed result | Limit |
|---|---|---|---|
| OpenAI-compatible proxy, `openai/gpt-5.4-mini` | `server-full` UAR `POST /v1/chat/completions` | HTTP 200 and a fresh expected model response were observed. | One genuine response through that local provider path at that checkout and date. |
| OpenAI-compatible proxy, `openai/gpt-5.4-mini` | Packaged `server-full` Chat UI | A fresh arithmetic prompt returned the correct model answer. | One completed UI request; no availability or other-profile claim. |
| Kimi for Coding, `kimi-for-coding/k3` | `server-full` provider API plus chat completion | Environment-backed credential status was reported and one fresh completion returned. | One configured API route and response at that checkout and date. |
| Kimi for Coding, `kimi-for-coding/k3` | Packaged `server-full` Providers, Agents, and Chat UI | Effective run policy named Kimi k3 and the model completed the fresh request. | One UI-selected agent/model route; it does not certify other models or profiles. |

The retained record also observed agent, skill, and knowledge workflows. Those
bounded results are documented on their owning pages rather than aggregated
into a runtime-level verdict.

## Evidence limits

These observations apply only to `server-full`, the named providers/models,
packaged boundaries, source SHA, checkout, and date. They do not establish
continuous availability, latency budgets, load behavior, credential rotation,
`minimal`, `embedded-mobile`, or any later source revision. A present readiness
claim requires a fresh genuine-model request in the target environment.

## Profile limits

- `server-full` exposes the packaged API and UI paths described above.
- `minimal` exposes the server completion APIs but has a smaller composition and
  no packaged admin UI claim; verify it independently.
- `embedded-mobile` has no HTTP or packaged UI boundary. Its host must provide a
  real loaded model through the injected driver and observe the result through
  the embedded UAR call path.

Results never transfer silently across profiles. Next, [create and run an
agent](/docs/agents/overview).
