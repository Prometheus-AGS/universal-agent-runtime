# Local Boss lifecycle candidate payload

The active phase's operator verification gate is CLOSED. These commands are
documented proposals, not authorization or evidence of execution. The lead must
complete the whole-phase inventory and obtain explicit command approval first.

## Immutable source and fresh build

The standalone `scripts/build-boss-candidate.mjs` producer is local only. It does
not change or invoke the deployment workflow's existing sidecar packager. Its
inputs are the exact committed UAR SHA and a new absolute output directory outside
the checkout, whose parent already exists. The exact committed root object and
its submodule objects must be available locally. The helper exports
those commit objects using Git archives, then builds that snapshot
with a fresh temporary `CARGO_TARGET_DIR`. Existing target binaries are never
input to this path. Dirty checkout files are never build inputs, and the helper
does not clean or revert them. Its own source must match the selected committed
producer. `GITHUB_SHA` and the working checkout's HEAD are not source inputs.

Proposed command, from the selected UAR repository:

```sh
node scripts/build-boss-candidate.mjs "$UAR_SOURCE_COMMIT" "$UAR_CANDIDATE_OUTPUT"
```

The helper executes exactly this Cargo command inside the immutable snapshot:

```sh
cargo build --release --locked --no-default-features --features server-full --bin uar-sidecar --target aarch64-apple-darwin
```

The only supported candidate is native Apple Silicon, `darwin-arm64`, target
`aarch64-apple-darwin`, feature set `["server-full"]`. No other platform's release
identity or artifact is updated. The output directory must not already exist.
Failure retains any created output and build log, but does not publish a success
record. Retrying requires a new output directory. Temporary build/source state is
removed after completion or failure; retained candidate output is not removed.

## Retained producer and consumer contract

The producer retains the package directory, `uar-sidecar-darwin-arm64.tar.gz`,
and `uar-sidecar-darwin-arm64.json`. The package contains the fresh executable,
the committed models, three Cedar policies, LICENSE, discovered fresh runtime
libraries, `uar-build.log`, and `payload-manifest.json`. The outer record retains
the archive SHA-256. The manifest retains every packaged file's size and SHA-256.

Both record and manifest contain matching `provenance` with schema 1, producer
`scripts/build-boss-candidate.mjs`, `sourceMode: "immutable-git-objects"`, exact source commit and Git tree, target,
features, lifecycle control requirement, and `build`. The build object contains
the exact command array, `isolatedTarget: true`, snapshot mode
`git-archive-with-submodules`, and the `path` and `sha256` of `uar-build.log` and
`uar-sidecar`. The inherited control requirement is
`{"transport":"inherited-stdin","version":1,"responsePrefix":"LIFECYCLE:"}`.
This describes the intended authenticated inherited-input protocol; its real
snapshot/reconnect/EOF behavior still requires the later process integration gate.

Boss `build/local-uar-source.json` schema 2 pins the exact committed source and
this platform/target/features/protocol contract. Boss requires both:

- `THE_BOSS_LOCAL_UAR_SOURCE_DIR`: absolute repository root containing the exact
  pinned commit and tree objects; unrelated working changes may remain.
- `THE_BOSS_LOCAL_UAR_CANDIDATE_RECORD`: explicit absolute path to the retained
  `uar-sidecar-darwin-arm64.json` file.

An explicit candidate record selects this committed-object contract. Without
that variable, Boss retains the legacy local path: clean checkout HEAD must
equal the pin and the record is read from `dist/boss-sidecar`. That legacy path
does not gain candidate provenance or permission to package dirty source.

`scripts/prepare-local-uar-payload.cjs` now loads the entire candidate, checking
the archive inventory, regular-file paths, inner/outer provenance equality,
retained build output digests and all payload file digests before reporting it
ready. The staged local marker retains provenance and the archive digest. Neither
preparation nor packaging changes personal runtime settings or credentials.

Proposed Boss local packaging command, with those two environment variables set:

```sh
pnpm build:mac:arm64:local
```

The existing `pnpm build:mac:arm64` command explicitly selects the public profile
and overrides `THE_BOSS_UAR_LOCAL=0`; it does not consume a local candidate. The
phase still requires its separate public/customer build gate. The lead must
name the appropriate commands and their prerequisite payloads explicitly rather
than treating the local candidate build as public release acceptance.

## Limits that remain material

The uncomfortable case is an internally consistent receipt manually fabricated
by someone who can write the retained artifact directory. These local receipts
are integrity and first-party producer provenance, not a signed supply-chain
attestation against a malicious local owner or compromised compiler. A real
build and process operation must establish compatibility before acceptance.
The legacy deployment packager remains unchanged and does not gain this
provenance merely because the helper exists.

No producer command, artifact digest, build, preparation, archive extraction,
process probe, or customer packaging operation was executed to author this
document. No candidate output currently constitutes acceptance evidence.
