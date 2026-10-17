# rc05 scoped predecessor staging helper — 2026-10-17

## Source state and ownership

Private helper: `/Users/gqadonis/.local/state/uar-v1.1.1/supervisor/stage-qualified-rc05.mjs`.
It is based on the existing rc04 helper, the live API routes and request types in
`src/uar/api/collaboration/{team_planning,team_execution}.rs` and
`src/uar/domain/team_execution.rs`, and the native staging packet. It has not
been run. It touches only a fresh private receipt when an operator later runs it;
this handoff is the only repository file added for the helper. Other agents share
the checkout; preserve their edits.

## Exact inputs and intended result

Invoke `node /Users/gqadonis/.local/state/uar-v1.1.1/supervisor/stage-qualified-rc05.mjs`
with each of these required `--name value` arguments:

| Argument | Required value |
| --- | --- |
| `--runtime-url` | Bare HTTP loopback origin forwarded to the fresh qualified rc05 predecessor, for example `http://127.0.0.1:21906` only if that forwarding is actually active. |
| `--credentials` | Private JSON with `admin_token` and `admin_key`, such as the existing private `credentials.json`; never copy values into a command or repo file. |
| `--preparation` | Fresh rc05 preparation JSON with status `fresh-qualified-predecessor-ready`, a live capabilities/ownership pair, and controller-journal registration rows. Do not pass the rc04 preparation. |
| `--provider-mode` | The active provider mode JSON, already atomically set to `{"mode":"complete","delay":601}`. |
| `--receipt` | New nonexistent private JSON path, unique to this scope; the helper creates it mode `0600`. |
| `--repo` | `/Volumes/T7/Projects/prometheus/universal-agent-runtime`, containing the checked-in collaboration fixtures. |
| `--suffix` | Unique `qualified-supervisor-...rc05-NN` suffix, such as `qualified-supervisor-20261017-rc05-01`; use a new suffix after partial staging. |

The helper compares fresh live capabilities and the execution claim with the
preparation, selects the actual journal registration whose `team_incarnation`
matches the claim, installs a resealed package and binding, creates a two-member
team, creates and claims two tasks, admits two queued attempts, and dispatches
only the active predecessor. It reads execution and ownership again and requires
one still-queued attempt plus one running, cancellation-requested, or uncertain
attempt on the same predecessor fence. Its private receipt contains request
names/statuses, IDs, fences, registration and small attempt summaries. It does
not contain bearer tokens, admin keys, request bodies, or arbitrary server
responses. Console output is a compact status and receipt path.

## Remaining boundary

The helper does not create a production qualification record, reboot the guest,
start the replacement, inspect the provider request trace, capture the new
replacement fence, write the immutable same-image identity receipt, or run
recovery/acceptance. The lead must observe the provider trace before the
guest-only boot transition. A successful staging receipt is preparation, not
recovery proof. The old rc04 controller qualification may be reused only after
the lead checks its controller source digest, mechanism, controller ID and
qualification ID against the fresh registration. The final recovery adapter
still requires actual journal rows, distinct fences, nonempty queued/uncertain
attempt IDs, the exact rc05 image ID and source/fixture identities.

No script execution, test, build, or guest action was performed by this helper
author. The source was inspected only; runtime behavior remains unverified.
