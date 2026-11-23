## Why

A desktop host that wants to start UAR as a child process (issue #539, MailCraft) needs to learn the bound port, hand over a launch token on stdin, and get one readiness line. All three already exist in the `uar-sidecar` binary, but a standard install ships only `universal-agent-runtime`, so a host that did not bundle the sidecar payload cannot reach them. `--port 0` also does not mean "the operating system picks": the sidecar turns it into 1906 and walks upward. Separately, no route reports which build is running without credentials: `/health` returns only `{"status":"ok"}` and `/api/uar/capabilities` requires authentication, so an installer cannot check the running version.

## What Changes

- Add `--supervised` to `universal-agent-runtime`. It runs the same entry point as `uar-sidecar`: launch token on the first stdin line, one token-guarded loopback listener, one `READY:<port>` line on stdout, exit when stdin closes. The entry point moves from `src/bin/uar-sidecar.rs` into the library (`src/sidecar/`); `uar-sidecar` becomes a wrapper with unchanged behaviour.
- Supervised mode through the standalone binary loads no `.env` file implicitly, in line with issue #419. `uar-sidecar` keeps loading one from its working directory or a parent directory.
- `--port 0` in supervised mode (either binary) binds an OS-assigned port. No port still starts at 1906, and a non-zero port still starts at that port; both still walk upward past occupied ports.
- Add `GET /api/uar/version`: package name and version, source commit, build profile, target triple, enabled Cargo features, wire protocol versions and the administration manifest revision. It needs no credentials on a standalone server and the launch token in supervised mode.
- `build.rs` records the build profile, target, features and source commit for that route. The commit is taken from the `UAR_GIT_SHA` build variable when it is set, otherwise from the checkout in release-profile builds; a debug build without the variable records none.
- Document the launch contract for hosts in `docs/operations/supervised-launch.md`.

Not in this change: a server-enforced "no tools" start mode (the optional ask in #539; follow-up issue), a `--port-file` option (`READY:<port>` already reports the port), any change to the compiled `/data/ingest` default (documented instead), and any new capabilities route (`GET /api/uar/capabilities` and `/api/uar/discovery/*` already serve that purpose).

## Capabilities

### New Capabilities
- `host-supervised-launch`: starting the standalone binary as a supervised child, and OS-assigned port selection in supervised mode.
- `runtime-version-info`: the build identity route and the build facts it reports.

### Modified Capabilities
(none — the sidecar requirements live in the unarchived changes `sidecar-launch-security`, `configurable-sidecar-port` and `fix-sidecar-loopback-auth`; this change adds to them without altering their requirements.)

## Impact

- **Code:** `src/sidecar/**` (moved from `src/bin/uar-sidecar*`), `src/bin/uar-sidecar.rs`, `src/main.rs`, `src/lib.rs`, `src/config.rs` (`Cli::supervised`), `src/uar/api/version.rs` (new), `src/server.rs` (one route), `src/uar/security/middleware.rs` (one credential-free GET), `src/uar/api/openapi.rs`, `src/uar/api/administration_capabilities.rs`, `build.rs`.
- **APIs:** one new read-only route, `GET /api/uar/version`. One new CLI flag, `--supervised`.
- **Security:** the supervised guard is unchanged and still outermost, so the new route requires the launch token there. On a standalone server the route is public by operator decision (2026-11-21); it returns compile-time metadata only. A network-exposed server now tells an unauthenticated caller its exact build.
- **Runtime UX, provider compatibility, realtime state:** unchanged.
- **Hosts:** The Boss is unaffected unless it passed `--port 0` expecting 1906; no in-repository caller does.
- **Build:** release-profile builds invoke `git rev-parse` when `UAR_GIT_SHA` is unset; a build without git or a checkout records no commit. `scripts/build-boss-candidate.mjs` builds from an exported snapshot and now passes its source commit as `UAR_GIT_SHA`. Container images report `git_commit: null` until the image build passes `UAR_GIT_SHA`.
- **Tests:** `tests/support/sidecar_process.rs` now gives every launch a private `server.ingest_directory` and the checkout's embedding model directory. Without them no process started by that harness reaches readiness on macOS with the `server-full` feature set.
