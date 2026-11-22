## Context

`uar-sidecar` already implements the supervised launch contract and is covered by real-binary tests (`tests/sidecar_launch_security/`). It is built and published only in The Boss payload. The standalone binary is 367 MB, so shipping a second copy of nearly the same program in every install is not attractive.

## Decisions

### 1. One entry point, two ways in
The sidecar `main` moves into the library as `sidecar::run()`. `uar-sidecar` calls it; `universal-agent-runtime` calls it when its arguments contain `--supervised`.

The standalone `main` checks its raw arguments before it builds a Tokio runtime or parses anything. The supervised bootstrap writes process environment variables and reads the token, and both must happen while the process is single-threaded. A clap subcommand was rejected: clap would require every existing option to precede it (`uar --port 0 sidecar`), and the supervised bootstrap parses the command line itself.

Alternatives rejected: shipping `uar-sidecar` in the native installers (roughly doubles the install and changes installer scripts on three platforms); leaving hosts to bundle the sidecar payload (the standard install still lacks the mode).

### 2. Port 0 is a request for any free port
An explicit 0 binds port 0 once and reports what the operating system chose. It never falls back to the 1906 walk, so a host that asks for "any port" cannot be handed one it might mistake for a well-known instance. Omitting `--port` keeps the existing deterministic behaviour.

### 3. The version route is credential-free on a standalone server
Operator decision, 2026-11-21. An installer or upgrade script has no JWT or API key, and `/health` is already public. The body is compile-time metadata: no path, configuration value, host name or credential. Uncomfortable consequence: an unauthenticated scanner that can reach a network-exposed server learns its exact build.

The exemption is for `GET` on the exact path. In supervised mode the launch-token guard is the outermost layer and has no exemptions, so the route needs the token there without extra code.

### 4. Where the commit comes from
`UAR_GIT_SHA` at build time when set, in any profile; otherwise `git rev-parse HEAD` in release-profile builds; otherwise absent. Debug builds skip the checkout lookup on purpose: `env!` makes the commit part of the crate's compiler environment, so recording it in debug builds would recompile the whole crate after every commit. A `UAR_GIT_SHA` that is not a full hexadecimal object id fails the build instead of being dropped. The build script watches `HEAD`, `packed-refs` and the branch-ref directory, so a release build made after a new commit records the new commit. The route returns `null` when no commit was recorded and does not claim a clean tree.

### 5. The standalone binary keeps its `.env` rule in supervised mode
`uar-sidecar` calls `dotenvy::dotenv()`, which loads a `.env` from the working directory or the nearest parent directory that has one, before the arguments and the bound principal are read. The standalone binary stopped doing that for issue #419, and the repository tracks the sidecar behaviour as an open security-relevant finding. Supervised mode through the standalone binary therefore loads an environment file only when `--env-file` or `UAR_ENV_FILE` names one, as the standalone server does. `uar-sidecar` keeps its existing behaviour; changing it is a separate decision for the host that ships it.

### 6. No new capabilities route
`GET /api/uar/capabilities` already returns the version, AG-UI profile, a closed capability vocabulary and the administration route manifest; `/api/uar/discovery/{agents,skills,tools}` already list what is registered. A plugin listing belongs to the plugin change (`plugin-loader-wit-contract`), where the loader stops being a stub.

## Risks

- The code move touches security-sensitive start-up code. The existing sidecar cases are the evidence that `uar-sidecar` behaves as before. Their assertions are unchanged; the shared harness now gives every launch a private ingest directory and the checkout's embedding model directory, because the compiled ingest default cannot be created on macOS and a `server-full` process refuses to start without the model files.
- `--supervised` inherits the sidecar's Windows requirement for an overlapped named-pipe stdin. It is documented for hosts and not changed here.
- A supervised child needs more than a token: `server.ingest_directory` and `persistence.database_url` set to paths the host owns, a `policies/` directory of Cedar files in its working directory, the embedding model files where the build includes local embedding, and the settings admin-key requirement switched off or a key supplied. None of these has a default that works for a desktop host. They are documented, not changed.
- The embedded-database maintenance commands stay `uar-sidecar` only; they are selected by the first argument, before the supervised flag is seen.

## Verification

After all implementation, one serial batch: `openspec validate`, the GitHub Actions policy validator, the `sidecar_launch_security` test target (existing cases plus `supervised.rs`), the moved unit tests and the OpenAPI path test, then a by-hand parent run against the built standalone binary. The build for that batch is a debug build with `UAR_GIT_SHA` set, so the supplied-commit path is exercised; the release-profile checkout lookup is not. Windows and Linux are not exercised.
