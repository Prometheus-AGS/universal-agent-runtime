# FC6 handoff — qualified same-image supervisor preparation

Date: 2026-10-18. Execute task `4.1`. Operator instruction: bounded existing-guest
repair plus the planned FC6 preparation sequence, named in the OpenCode session
continuation of the 2026-10-17 handoff. Preparation is not acceptance; no
functional gate has run.

## Root-cause repair (setup defect, guest-only provisioning)

The failed rc05-10 staging routed team attempts to a dead provider port. The
controlled fixture `uar111-provider.service` binds an OS-assigned ephemeral port
on every start (`ThreadingHTTPServer(("0.0.0.0", 0))`) and publishes it to
`/var/lib/uar/observer/provider-port`. The guest binding hook
`/usr/local/sbin/uar-bind-provider.py` (`ExecStartPre` of `uar.service`) synced
only `llm.base_url`; the `providers[openai].base_url` route used by team
attempts stayed stale. The script exists nowhere in the repository; the FC5
freeze hashes the production source paths (`src`, `scripts`, `Cargo.toml`,
`Dockerfile`, and the other SOURCE_PATHS entries) and the fixture paths
(`tests/`, `scripts/release/`), so this guest provisioning edit touched
neither frozen tree, and the candidate image and installed binary are
unchanged.

- Backup preserved in-guest: `uar-bind-provider.py.bak-20261017-fc6`
  (`sha256:93cb96c370d3c342a174d1f1b9b771ca9948716016b7439170a7566b5fd0cdf0`).
- Corrected script installed (`sha256:d6dea5f7bc9e40f433dcaedd8e07930968fcaf8183dc2efd104b0f739c7dba9d`):
  ExecStartPre reads the live port file and sets both `llm.base_url` and the
  `providers` entry with `id == "openai"` to the same endpoint, failing hard if
  the entry is absent. First rendered edit assumed a dict-shaped `providers`;
  the live config uses a list — observed via the service start failure journal,
  corrected, reinstalled.
- `systemctl restart uar.service` (service restart only, not the boot proof):
  all three services active; both effective routes bound to then-current port
  41887. Verified from config and service state without any HTTP call to the
  fixture.

## Fresh same-image staging (scope ff-rc05-11)

- Fresh preparation `qualified-function-first-preparation-04.json`: predecessor
  incarnation `fdf26bf3-3e55-4cfd-8c75-0c90b46756ee`, runtime epoch
  `27916d1d-92b7-4d9b-b26f-d044a4ddcc07` (journal-exact registration),
  ownership claim held at epoch 2.
- Staging command (host, existing helper):
  `node stage-qualified-rc05.mjs --runtime-url http://127.0.0.1:21906 --credentials credentials.json --preparation qualified-function-first-preparation-04.json --provider-mode provider-mode-hold601.json --receipt qualified-function-first-team-staging-04.json --repo <checkout> --suffix qualified-supervisor-ff-rc05-11`
  Result `active-dispatched`, 17/17 requests 200/201. Team
  `ee70f824-c754-4ad0-9ff7-608f6fd9dc70`; queued attempt
  `237a58c7-7839-4e8d-8f02-c4bac2c0dc56` (queued); active attempt
  `7f8511b7-c10d-4fc1-a561-2c81da640ad3` (running, reservation
  reserved-unknown/confirmed) on the predecessor fence.
- Real provider contact: provider trace row 8 records the ff-rc05-11
  active-predecessor `/v1/chat/completions` call (fixture mode `complete`,
  delay 601) with fixture credentials (preboot receipt
  `qualified-function-first-preboot-04.json`).

## Physical whole-guest proof

- `limactl stop --force dot17-linux-qualification` 2026-10-18T00:54:28Z (SIGKILL
  to qemu) then `limactl start` 00:54:39Z. Boot ID changed
  `defe476c-0c12-4962-9064-212c66dc9248` → `114a756f-018f-4990-ada7-6543c67eaf8d`,
  same machine `bac25da7bc5d4d5d84b12fa478b4d465`. A container or service
  restart cannot produce this transition.
- Post-boot the fixture re-bound ephemeral port 41887 → 41829; the corrected
  ExecStartPre bound **both** provider routes to 41829 at replacement start
  (verified in `/etc/uar/runtime.json`), closing the original desync durably.
- Replacement `uar.service` start 00:55:24Z registered runtime epoch
  `4b897bcf-9d04-4403-9157-771aaa41a9ec` / incarnation
  `7917a6e8-75c8-4005-bad3-42e693083d6d`. Live execution-owner view after
  replacement: `ownsExecution: false`, claim = predecessor fence (fdf26bf3,
  epoch 2, held), currentFence = replacement (7917a6e8, epoch 0); both staged
  attempts still on the predecessor fence with unsettled accounting.

## Sealed receipts

- Identity receipt `qualified-function-first-identity-04.json`
  (`sha256:2b9883574e76f527eb3064f83f2bbc4ac7023d71a3a4b9443bebe65184995c56`):
  phase/isolated, image/platform/source/fixture identities, runtime_image_id
  `sha256:56032b74…` (matches host `docker image inspect {{.Id}}`),
  runtime_url, journal-exact predecessor/replacement registrations and fences,
  team_scopes with queued/uncertain attempt IDs, physical transition record.
- Private infrastructure `acceptance/infrastructure-function-first-local.json`
  (new): surreal/postgres pinned backend settings (carried from local04),
  supervisor block binding qualification `sha256:0a3710de…` (controller source
  hash matches the frozen repo `scripts/supervisor/controller.py`), identity
  receipt hash, Lima journal transport (limactl `sha256:4636491d…`),
  runtime_url and scoped tokens by private reference.
- Local manifest `evidence/gates/function-first-local.json` is now executable
  (NON-EXECUTABLE marker removed; all fields match receipts). arm64/amd64
  published manifests remain non-executable templates.

## What this does not claim

No acceptance, certification, or functional observation has occurred. FC7
independent review and the FC8 local functional command remain. The recovery
group's generation-1 challenge, physical exclusion and reclaim run inside the
later authorized gate and consume this preparation exactly once. Failed
rc05-10/-03 receipts are retained untouched as evidence. Dirty checkout,
durable guest stores, candidate image and installed binary are preserved.
