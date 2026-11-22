## 1. Supervised entry point in the standalone binary

- [x] 1.1 Move `src/bin/uar-sidecar.rs` and its three modules into `src/sidecar/` and expose `sidecar::run()`.
- [x] 1.2 Reduce `src/bin/uar-sidecar.rs` to a wrapper around `sidecar::run()`.
- [x] 1.3 Add `Cli::supervised` and dispatch `--supervised` in `src/main.rs` before the runtime is built.
- [x] 1.4 Bind an OS-assigned port when port 0 is requested.

## 2. Build identity route

- [x] 2.1 Record build profile, target, features and source commit in `build.rs`.
- [x] 2.2 Add `GET /api/uar/version` (`src/uar/api/version.rs`) and register the route.
- [x] 2.3 Exempt `GET /api/uar/version` from standalone credentials.
- [x] 2.4 Add the route to the OpenAPI document and the administration manifest.

## 3. Tests and documentation

- [x] 3.1 Add unit tests for flag detection, port selection and the version body.
- [x] 3.2 Add real-binary cases in `tests/sidecar_launch_security/supervised.rs`.
- [x] 3.3 Give harness launches a private ingest directory.
- [x] 3.4 Write `docs/operations/supervised-launch.md` and link it from the README.

## 4. Verification

- [x] 4.1 `openspec validate host-supervised-launch` and the GitHub Actions policy validator pass.
- [ ] 4.2 The `sidecar_launch_security` target, the moved unit tests and the OpenAPI path test pass with no build warnings.
  - Open: 33 of 41 pass. The other eight fail identically, with the same messages, on unchanged main (0ce227af) given the same launch configuration. The unit and OpenAPI tests pass (16).
- [x] 4.3 A by-hand parent run against the built standalone binary shows the launch, the version route with and without the token, and exit on stdin close.
