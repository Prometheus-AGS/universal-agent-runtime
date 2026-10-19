# UAR 1.1.1 Release Qualification Report, Round 4 — Profile B (function-first, local)

**Date**: 2026-10-18
**Contract**: unchanged acceptance contract; round-4 bounded allowance (90 min,
50 tool calls, 2 repair cycles, no-progress 2; 15 min / 8 calls reserved for
closeout) to resolve the two round-3 named blockers and reach an explicit
release decision.
**Prior reports preserved byte-identical**: round-1
`release-qualification-report-1.1.1-function-first.md`, round-2
`…-round2.md`, round-3 `…-round3.md`.

## Blocker 1 — local-backend store-verification failure: CONFIRMED AND CORRECTED

**Trace (hypothesis tested, not assumed).** The missing-`surreal_user`/
`surreal_pass` hypothesis was falsified as the direct cause. The actual path
(`scripts/release/uar_1_1_1_credentials.py` `Store.query`, local branch) stops
the node and opens a temporary SurrealDB host over the runtime's own
SurrealKV volume, then `POST /sql` with `Basic root:<random>`. That store's
system credential was set when the runtime first initialized it
(`src/uar/persistence/providers/surreal.rs:111-112`, product defaults
`root`/`root` via `unwrap_or("root")`); a reopened store keeps the init-time
credential, so the random fixture password could never match → **401**.

**Correction (demonstrated cause only).** The fixture now authenticates as the
credential the store was initialized with — `persistence.get("surreal_user",
"root")` / `persistence.get("surreal_pass", "root")`, mirroring the product's
own defaults; a configured non-default password is registered for redaction.
No assertion changed, no authentication disabled, negative-authorization
scenarios untouched, no secrets in logs.

**Second demonstrated cause en route (round-3 staleness class, deeper).**
`local-metadata-upgrade`'s first run after the auth fix still failed at
`install_mappings` with a truncated-config startup exit — and the new
config-view diagnostics (intended == visible at probe time, runtime had read
EOF at the *previous* config's length) proved the shared file-view staleness
is transient and immune to post-mortem detection. Deterministic correction:
each config version is written under a fresh content-hashed filename
(`config-<sha12>.json`) — a name no container has resolved cannot be served
from a stale cache entry — with atomic stage+fsync+replace; `restart()`
recreates the container only when the config name changed (warm `docker
restart` semantics preserved otherwise). Startup retries were removed
entirely; every failure capture records `config_view` (intended vs
visible hashes), so nothing is concealed. This consumed repair cycle 2.

**Demonstration.** The previously failing case family ran end-to-end: store
seeding, direct store queries, `install_mappings`, and the runtime config
reloads all pass. The case **reaches its intended assertions** — and fails the
first migration-status assertion on a third, newly-surfaced defect (below).

## The failing criterion (exact)

`local-metadata-upgrade`'s first intended assertion:

```
require(item(node, host, "upgrade")["status"] == "revision_unavailable",
        "historical revision was invented")
```

**Observed** (captured verbatim by an instrumented scenario-head run, evidence
`uar111-r4-preview-evidence`):

```json
{"provider_id": "upgrade", "status": "ready",
 "source_revision": "None", "created_at": "2026-10-18T18:15:52.636125315Z",
 "precondition": "\"c49b4d6bbf99c1e395c5ca6fa61f7c454ada441fbd400f274a6ca12351e164c6\""}
```

The scenario seeds `upgrade` with `row_revision = NONE` (SurrealQL). The
preview reports `source_revision` as the **string `"None"`** and a computed
precondition over it — the product literally invented a historical revision.

## Root cause (precisely located)

`src/uar/persistence/providers/surreal.rs:3010-3016`
(`unwrap_surreal_value`): the converter maps the serde variant string
`"Null"` → JSON `null`, but Surreal `Value::None` serializes as the string
`"None"`, which falls through `J::String(_) => v` unchanged. `Option<String>`
fields (`row_revision`, `source_revision`, …) therefore deserialize as
`Some("None")`. In `src/uar/security/credentials/migration.rs:162` the ladder
keyed on `item.source_revision.is_none()` never fires; `revision_unavailable`
is skipped and the item is classified `ready` with a precondition over the
invented revision. Sibling rows confirm the mapping: a UUID-revision row and
an unmapped row classify correctly (`ready`, `unmapped_quarantined`); only
the NULL-revision row is corrupted.

## Smallest next action

One-line product fix in `unwrap_surreal_value`: map the `"None"` variant
string (Surreal `Value::None`) to `J::Null` alongside the existing `"Null"`
mapping; then rebuild the candidate and rerun the credential scenario family.
No fixture change is needed for this defect.

## Blocker 2 — candidate/build identity: NOT EXECUTED (barred by contract)

The mandated order places the freeze/rebuild before acceptance, and bars the
acceptance run unless readiness passes and allowance remains for the observed
runtime (~24 min) plus closeout. Readiness fails on the criterion above, and
the remaining allowance (17-min rebuild + 24-min run + closeout) does not fit
in the reserve-respecting window. Per contract ("Launch the final
whole-program acceptance run only if these preconditions pass and sufficient
allowance remains"), no rebuild-and-accept cycle was started; no identity
validation was bypassed and no image was relabeled.

## Release decision

> **UAR 1.1.1 certification — NOT READY FOR RELEASE (round 4, profile B).**
> The round-3 blocker "local-backend store-verification" is resolved at its
> demonstrated cause (fixture temp-host auth identity + the transient config
> view class eliminated by content-versioned config names). The mandatory
> coverage item `local-metadata-upgrade` cannot pass against the current
> candidate because of a precisely located product defect: Surreal `NONE`
> values deserialize as the string `"None"` through `unwrap_surreal_value`,
> causing the credential migration preview to invent historical revisions.
> Exact failing criterion: `local-metadata-upgrade`
> `status == "revision_unavailable"` (observed `ready` with
> `source_revision: "None"`). Smallest next action: one-line null-mapping fix
> in `unwrap_surreal_value`, rebuild, rerun the credential family. No second
> full-program attempt was made; the acceptance program was not launched.

## Cumulative budget accounting (with round-3 overrun preserved)

| Round | Allowance | Wall | Tool calls | Repair cycles | Notes |
|---|---|---|---|---|---|
| 3 | 75 min / 50 calls / 2 cycles | ~78 min | ~54 | 2 | Overruns disclosed in the round-3 report (prospective approval only) |
| 4 | 90 min / 50 calls / 2 cycles | ~72 min | ~26 | 2 (auth identity; config versioning) | Within allowance; 15 min / 8 calls reserved for closeout and used for it |

Round 4 stopped its diagnostic track after two consecutive synthetic demo
attempts produced no mechanism evidence (per the absolute no-progress rule);
the mandated case-level demonstration then supplied the evidence instead.

## Process lesson (recorded once; implementation left to the skills project)

**Readiness must establish authenticated dependency access, correct
configuration, and candidate/build identity before expensive acceptance
begins.** Recorded in `.prometheus/gotchas.md`. No matching follow-up existed
(round-3 check reconfirmed). Not implemented in this phase.

## Preservation

- Rounds 1–3 reports and all historical receipts byte-identical and untouched.
- Round-4 evidence: `uar111-r4-verify401-evidence{,-b}/`,
  `uar111-r4-preview-evidence/`, plus the instrumented preview capture quoted
  above.
- All changes remain uncommitted working-tree edits on HEAD `38f0b5d8`;
  no commit, push, tag, or publication. Product source was not modified.
