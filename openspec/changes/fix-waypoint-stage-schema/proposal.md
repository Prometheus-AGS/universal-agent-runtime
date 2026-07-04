# fix-waypoint-stage-schema

## Why

`write-position-reminder.sh` read `.stage // "unknown"` from
`current-waypoint.json`, but this project's waypoint schema has
historically only populated `.status`, never `.stage` — silently
rendering `Stage: unknown` on every regeneration until a `stage` field
was added to `current-waypoint.json` by hand during
`uar-spec-v2-and-polish`. That was a workaround, not a fix — the
underlying script/schema mismatch remained, and a future waypoint write
that omits `.stage` would silently regress.

## Cross-repo scope (disclosed, user-authorized)

The target script (`shared/scripts/write-position-reminder.sh`) lives in
`/usr/local/src/prometheus-skill-system` — a **separate git repository**
(`Prometheus-AGS/prometheus-skill-system`), not this one. This was
surfaced to the user as a blocker before proceeding (see
`current-waypoint.md`'s "Blocked" section, now resolved); the user chose
to fix it at the source in that repo rather than work around it locally
or re-carry it as debt.

## What changed (in `prometheus-skill-system`, commit `91006b8`)

- `shared/scripts/write-position-reminder.sh`: `.stage // "unknown"` →
  `.stage // .status // "unknown"`, matching the fallback pattern
  already used one line below for `exact_next_command`/`exactNextCommand`.
- `shared/scripts/write-session-summary.sh`: same fix — it had the
  identical bug, found while fixing the first script.
- `position-stop-gate.sh` already had the correct fallback order
  (`.status // .stage // empty`) — only these two scripts needed it.

Rebased onto `origin/main` (20 unrelated commits had landed upstream in
the meantime, none touching either file) rather than force-pushing, and
a pre-existing unrelated uncommitted change
(`rules-cache.md`, not mine) was stashed and restored around the rebase
rather than touched or discarded.

## Verification

- Tested against a synthetic waypoint with only `.status` set (no
  `.stage` at all): both scripts now resolve the real value instead of
  falling back to `"unknown"`.
- Re-ran `write-position-reminder.sh` against this project's real
  `current-waypoint.json` — renders correctly.
- Pushed to `prometheus-skill-system`'s `origin/main` (commit
  `91006b8`), not just committed locally.

## Known follow-up (disclosed, not fixed here)

`write-session-summary.sh` also reads `.next_pending_change` /
`.last_completed` (snake_case only) — these don't match this project's
actual `nextPendingChange` camelCase convention either, a separate
schema mismatch from the one this change fixes. Left unfixed since it
wasn't part of this change's assessed/planned scope; flagged for a
future pass.
