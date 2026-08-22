# Embedded SSE recovery verification summary

Scope: local Chromium, Vite frontend, UAR `server-full`, embedded SurrealDB SSE,
and the source entity-management workspace. Results transfer to no other
browser, runtime profile, provider, deployment, or platform.

- Named delivery: PASS locally. Valid named `entity.change` payloads map once;
  transport-only and malformed controls do not mutate graph state.
- Recovery: PASS locally. The failed source closes before one capped
  replacement; status recovers; unsubscribe cancels a pending replacement; one
  received post-reconnect event yields one update.
- Browser boundary: PASS locally. The exact fresh-process scenario instruments
  the application's EventSource and observes one visible initial update, a
  second real stream request, and one visible recovered update without reload,
  probe, store injection, or manual replay.
- Upstream projection: PASS locally. Both normalized view hooks rerender an
  existing entity behind stable IDs; the upstream React suite passed 58/0.
- Source build: PASS locally. The upstream pnpm 11 compatibility scenario passed
  1/0; dependency-aware preparation built core and React before the browser run.
- UAR checks: typecheck, lint, focused adapter 3/0, build, preparation, and exact
  Chromium 1/0 passed. The full frontend command exited 1 with 328 passing and
  10 unrelated A2UI Storybook failures; no full-suite pass is claimed.
- Delivery limit: the adapter resumes received events from the replacement
  connection. It does not replay or claim lossless delivery while disconnected.
- Publication limit: source/compatibility PR #20 and generated rc.2 PR #21 are
  open and separate. No npm publication occurred.

Independent artifact review: PASS. A fresh history-free critic returned zero
findings and an independent judge returned PASS on the corrected source hashes,
focused controls, progressive checkpoint sequence, upstream PR separation,
schemas, strict OpenSpec, and bounded scope.
