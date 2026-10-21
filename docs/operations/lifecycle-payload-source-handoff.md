# Lifecycle payload source handoff — 2026-10-20

Classification: source inspection and authorized documentation writing only.
Verification gate: **CLOSED**. No test, build, check, packaging, hashing,
preflight, review, publication, or dependency-pin mutation ran for this handoff.

The authoritative waypoint read for this task names
`uar-outstanding-issues-v1-1-1::function-first-release-certification`, stage
`execute-lifecycle`, plan revision 69, with 138/153 implementation items complete.
This handoff does not supersede that position or declare the phase complete.

## Source producer and provenance gap

Source discovery found `scripts/package-boss-sidecar.mjs` and
`scripts/build-tauri-sidecars.sh`. It did not find a
`scripts/build-boss-sidecar.sh` entry. The actual Boss sidecar build command is
authored in `.github/workflows/deploy.yml` before the packaging invocation.

The packager maps four exact platform/target pairs, reads the existing binary
from `target/<target>/release/uar-sidecar[.exe]`, and copies working-tree models,
three Cedar policy files, LICENSE, and discovered runtime libraries. It writes
`dist/boss-sidecar/uar-sidecar-<platform>/payload-manifest.json`, the archive,
and `dist/boss-sidecar/uar-sidecar-<platform>.json`.

Both records name `source`, obtained from `GITHUB_SHA` when set, otherwise
`git rev-parse HEAD`. The inner manifest contains schema 1 and individual file
size/SHA-256 entries. The outer record contains archive SHA-256, asset name,
platform, version, features, and packaged paths. These are authored mechanisms;
no hashes were computed or compared in this task.

Source-grounded failure scenario, **not executed**: leave an older sidecar
binary in the target directory, edit lifecycle source or models/policies, then
invoke the packager. Its binary-existence condition is satisfied; its source
label still names HEAD (or the environment's GITHUB_SHA), while the archive can
combine the old binary and new working-tree resources. The source contains no
clean-tree condition, source-tree identity, build-to-source receipt, or lifecycle
protocol assertion. Archive integrity alone therefore does not establish that
the compiled program implements the source commit's lifecycle behavior.

The uncomfortable case is a correctly hashed archive whose metadata names the
new commit while its executable predates that commit. A consumer must not
interpret this manifest alone as build provenance.

## Lifecycle interface the consumer must bind

`src/bin/uar-sidecar.rs` retains the initial 64-lowercase-hex launch-token line
and exact `READY:<port>` stdout line after HTTP readiness. The authenticated
inherited input pipe subsequently carries lifecycle commands. EOF requests
shutdown, including while an accepted reconnect remains pending; the reader,
accepted command futures, response writer, and input closure are joined.

`src/server/lifecycle_control.rs` defines a deny-unknown-fields request:

```json
{"version":1,"request_id":"consumer-assigned-id","operation":{"kind":"snapshot"}}
```

The other operation is `reconnect`, with exact `resource_id` and
`expected_generation` fields. Version must equal 1; request ID must be nonempty
and at most 128 bytes. This is a trusted host capability, not an HTTP route or
kernel write grant.

`src/bin/uar-sidecar/control.rs` emits one serialized
`LIFECYCLE:<JSON>` response line per completed accepted command. Correlated
execution failures retain `request_id`; malformed command parsing emits
`invalid_lifecycle_command` without one. The line input bound is 8192 bytes.
Keep the consumer's existing correlated failure handling and exact generation
binding. A version label in artifact metadata would describe the intended
contract; the later real process gate must establish actual interoperability.

## Lead and Boss writer integration obligations

1. Select the actual immutable UAR commit only after intended source and required
   artifacts are included. Do not label dirty working-tree behavior with its
   previous HEAD, invent a SHA, or weaken the existing Boss clean-pin and archive
   digest checks. No commit or source hash was selected by this handoff.
2. Bind a local candidate to that immutable source, exact target/features,
   producer command, and resulting archive digest. Establish source/build
   provenance before allowing the prepared artifact to satisfy Boss's local
   payload path. Public release identity remains a separate strict path.
3. Keep per-platform records truthful. An arm64 macOS candidate does not renew
   Windows or Intel payload provenance; retain their actual source identities
   or leave those targets unavailable for the new lifecycle candidate.
4. If changing the UAR packager is necessary, assign its writer explicitly.
   It is invoked by `.github/workflows/deploy.yml`; the mandated pre/post
   GitHub Actions policy validator is prohibited under the current CLOSED lock.
   Do not edit it or a workflow until the lead resolves that command-approval
   boundary. A separate local-only producer could avoid changing the deployment
   script, but still needs its own explicit ownership and implementation scope.
5. The Boss writer must inspect its actual local payload loader and preparation
   scripts before choosing their schema. This UAR-only role did not inspect or
   edit Boss, and does not claim compatibility of a proposed metadata extension.

## Proposed commands, not executed or authorized

After whole-phase implementation reconciliation, source freezing, and explicit
operator approval of exact commands, the existing source provides this macOS
arm64 producer sequence, run serially in the selected clean UAR checkout:

```sh
cargo build --release --locked --no-default-features --features server-full --bin uar-sidecar --target aarch64-apple-darwin
UAR_SIDECAR_FEATURES=server-full node scripts/package-boss-sidecar.mjs darwin-arm64 aarch64-apple-darwin
```

These commands alone do not close the provenance gap above. The lead must
include the implemented clean-source/build binding in the final exact command
proposal. The subsequent Boss command required by project instructions is
`pnpm build:mac:arm64`; its local preparation command and environment must come
from the Boss writer's actual source inspection. Do not infer authorization
for standalone hash, service-readiness, source-identity, or other preflight
commands from approval of a build.

## Delivery accounting

- Changed path: `docs/operations/lifecycle-payload-source-handoff.md` only.
- Interface effects: none; existing producer and lifecycle interfaces documented.
- Source inspection: `rg --files`, `rg -n`, `cat`, and `sed` over repository
  instructions, phase/task artifacts, producer/workflow source, and sidecar
  control source. A broad search output was truncated; conclusions above use
  the targeted producer and control-source reads. Two guessed documentation/
  plan paths were absent; neither was used as evidence.
- Verification commands/results: none. Gate remained CLOSED throughout.
- Unrequested additions/guards: none. No production guard was added.
- Unverified: compilation, archive contents, actual source identity, clean-pin
  behavior in Boss, lifecycle interoperability, shutdown/reconnect completion,
  every target's artifact provenance, and customer build delivery.

Lead retains commit/push/PR authority and whole-phase inventory ownership.
