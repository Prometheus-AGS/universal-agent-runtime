# Smoke Log — Providers + Agents Direct Migrations

**Date:** 2026-05-27
**Bundle:** `index-ChbheD4z.js` (built 2026-05-27)
**UAR:** PID 70040, listening on `0.0.0.0:1906`
**Proxy:** `127.0.0.1:8088`
**SurrealDB:** `localhost:28000` (Docker)

Open two Chrome windows at <http://127.0.0.1:8088/>. Verify in each tab's
DevTools → Network that ~10 `EventSource` connections to `/api/live/*` are
active before starting.

---

## Automated pre-flight (Claude Preview MCP, 2026-05-27 ~09:45)

Started a second `uar-jwt-proxy` instance on `127.0.0.1:8089` under Claude
Preview MCP to drive a headless smoke. Observations:

- **SPA loads** — Chat (`/`) and Admin Providers (`/admin/providers`) both
  render cleanly through the proxy; no crash banner.
- **Backend reachability via the preview proxy is fine** — direct
  `curl http://127.0.0.1:8089/api/catalog` returns 245 providers; `curl
  /api/uar/providers` returns the configured OpenAI provider. The proxy
  injects JWT correctly.
- **Single-tab harness can't perform cross-tab scenarios** — Preview MCP
  provides one tab; P1/P2/P3/A1/A2/R1/R2 all require two-tab observation.
- **Async eval hangs** — calling `fetch(...)` from `preview_eval` times out
  consistently in this environment, so the headless harness can't fully
  exercise the page either. Likely the `bootstrapEntityGraph()`
  `initSyncTransport` path interacts badly with the preview renderer
  (PGlite init or EventSource handshake).
- **Verdict on automation:** Preview MCP is a useful "does the page
  render?" check (✅) but cannot replace the 2-tab manual smoke. The
  human walkthrough below remains the canonical validation.

---

## Pre-flight

- [ ] Both tabs loaded, no console errors.
- [ ] `EventSource` connections visible in Network tab (filter "live").
- [ ] Default Assistant is set; at least one configured provider exists.

---

## P1 — Configure provider (cross-tab propagation)

**Setup**

- Tab A: Admin → Providers
- Tab B: Admin → Providers (list view, scrolled to a known position)

**Action (Tab A)**

1. Click `[+]` on an unconfigured provider (e.g. `groq` if not yet configured).
2. Enter a valid api_key + base_url.
3. Submit.

**Expected**

- Tab A: dialog closes; the row moves to "configured" section.
- Tab B: same row updates from "unconfigured" → "configured" within ~200 ms with no manual refresh.

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## P2 — Set default provider (optimistic flip)

**Setup**

- Tab A: Admin → Providers → select a configured provider that is NOT the current default.
- Tab B: Admin → Providers.

**Action (Tab A)**

1. Click "Set as default".

**Expected**

- Tab A: the "default" badge flips to the new provider **instantly** (well under 100 ms — the optimistic upsert on `ProviderMeta`).
- Tab B: the default badge moves to the same provider within ~200 ms (SSE-driven).

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## P3 — Remove provider (cross-tab removal)

**Setup**

- Tab A: Admin → Providers → select a configured provider (not the default).
- Tab B: Admin → Providers list.

**Action (Tab A)**

1. Click trash → confirm removal.

**Expected**

- Tab A: row vanishes instantly (optimistic remove); detail panel clears.
- Tab B: row vanishes within ~200 ms.

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## A1 — Agent memory toggle (latent-bug regression guard)

**Setup**

- Tab A: Admin → Agents → select an agent.
- Tab B: **Chat** view — open the AgentSelector dropdown so the agent list is visible.

**Action (Tab A)**

1. Toggle "Memory Enabled" (or any of the per-agent memory fields).
2. Save.

**Expected**

- Tab B: the same agent's row in the selector dropdown reflects the change **without a refresh**.
- Pre-migration, this was the silent staleness bug — the selector ran its own local fetch and ignored SSE.

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## A2 — Delete agent (cross-tab removal)

**Setup**

- Tab A: Admin → Agents → select a non-default agent.
- Tab B: both Admin → Agents list AND Chat → AgentSelector dropdown open.

**Action (Tab A)**

1. Click trash → confirm.

**Expected**

- Tab A: row vanishes instantly.
- Tab B Admin: row vanishes ≤200 ms.
- Tab B AgentSelector dropdown: agent disappears from the list ≤200 ms.

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## A3 — Switch active agent in chat sidebar

**Setup**

- Single tab on the Chat page.
- Note the current chat header / model badge.

**Action**

1. Open AgentSelector → pick a non-default agent.

**Expected**

- Chat header model badge updates to reflect the new agent's policy within one frame.
- Sending a message after the switch should respect the new agent's `policy.provider.default` model (verifiable via UAR `/tmp/uar.log` or via a distinctive system prompt).

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## R1 — Force setDefault rejection (optimistic rollback)

**Goal:** drive the `setDefault` mutation into a server-side failure and verify the optimistic flip rolls back.

**Recommended natural path:**

```bash
# Mint a JWT (use the proxy secret) and POST to set-default on an
# unconfigured provider id. The backend should reject (404 or 409).
SECRET='BbGRgttW6ZrWbapOXbubgIh+zZSBFxCbku7vkZxR07zYVLy3L/0bngpHIsT9n8XQ+PMcSShUB6UIRP5M3/wpVg=='
HEADER=$(printf '%s' '{"alg":"HS256","typ":"JWT"}' | openssl base64 -A | tr '+/' '-_' | tr -d '=')
EXP=$(( $(date +%s) + 3600 ))
PAYLOAD=$(printf '{"sub":"dev","name":"dev","roles":["admin"],"exp":%d}' $EXP | openssl base64 -A | tr '+/' '-_' | tr -d '=')
SIG=$(printf '%s' "${HEADER}.${PAYLOAD}" | openssl dgst -binary -sha256 -hmac "$SECRET" | openssl base64 -A | tr '+/' '-_' | tr -d '=')
JWT="${HEADER}.${PAYLOAD}.${SIG}"

# Replace <unconfigured-id> with a real one (e.g. "mistral" if not configured)
curl -sS -o /dev/null -w "%{http_code}\n" \
  -H "Authorization: Bearer $JWT" \
  -X POST "http://127.0.0.1:1906/api/uar/providers/<unconfigured-id>/default"
```

**Fallback** (if backend accepts unconfigured ids): use Chrome DevTools → Network → "Override response" on the next `POST /api/uar/providers/{id}/default` and return HTTP 500.

**Action**

1. Reproduce a `setDefault` failure via either path while watching the Admin → Providers page.

**Expected**

- The default badge flips optimistically for a frame, then **rolls back** to the prior default once the server returns the error.
- An error message appears (banner or inline) explaining the failure.

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## R2 — Force patchAgent rejection (optimistic rollback)

**Setup**

- Tab A: Admin → Agents → pick an agent.
- Open DevTools → Network → enable Local Overrides → add an override that returns HTTP 500 for `PATCH /api/agents/*`.

**Action**

1. Toggle a memory field and click Save.

**Expected**

- Memory toggle flips optimistically, then **reverts** when the 500 response lands.
- An error message surfaces.

**Observed:** _(fill in)_

**Verdict:** _Pass / Fail / Inconclusive_

---

## Summary

| Scenario | Verdict |
|----------|---------|
| P1 | _ |
| P2 | _ |
| P3 | _ |
| A1 | _ |
| A2 | _ |
| A3 | _ |
| R1 | _ |
| R2 | _ |

**Pass count:** _ / 8
**Below 6/8?** Escalate before `/kbd-reflect`.

## Triage (fill in only if any Fail)

For each failing scenario, file via `TaskCreate`:

- `phase=browser-smoke-providers-and-agents`
- `entity={provider|agent}`
- `scenario_id={P1|P2|...}`
- `description=` what was observed vs expected, with screenshot path if captured
