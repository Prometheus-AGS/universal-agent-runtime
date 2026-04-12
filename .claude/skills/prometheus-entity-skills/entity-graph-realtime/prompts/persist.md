# Persist — entity-graph-realtime

## 1. Files on disk

Confirm adapter + provider modules merged; remove experimental duplicates.

## 2. State file (optional)

```bash
bash prometheus-entity-skills/entity-graph-realtime/scripts/state-init.sh .
```

Update `.entity-graph-skills/entity-graph-realtime/state.json`:

- `current_phase`: `persist`
- `phases_completed`: full pipeline
- `artifacts`: touched paths
- `realtime_spec` summary

## 3. Runbook for operators

Document:

- Required env vars
- How to verify green connection
- What to restart when rotating credentials

## 4. Follow-up automation

If orchestrators detected, link OpenSpec/KBD tasks for hardening (authz on channels, monitoring).

## Done when

Runbook exists (PR description acceptable), code merged, typecheck clean.
