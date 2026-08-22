# Artifact-refiner validation receipt

Artifact: `fix-container-rust-toolchain-pin-consistency`

Tested implementation:
`64bea08695c341843e79e829ee0d4dfc0c210c3c`

## Constraint results

| constraint | observed result | scope limit |
|---|---|---|
| dated-selector-consistency | PASS — shell syntax passed and the validator reported Docker, repository, and effective `nightly-2026-07-18` values equal. | Source contract only. |
| fail-closed-selector-controls | PASS — floating nightly, Docker/repository mismatch, and effective-argument mismatch each exited 1 with the expected distinct error. | Validator controls only. |
| locked-arm64-compatibility | PASS — the pinned fixture compiled on `nightly-2026-07-18`; identical inputs on `nightly-2026-08-22` exited 101 with exactly three E0283 diagnostics. | `aarch64-apple-darwin` fixture only. |
| clean-production-image | PASS — the clean implementation checkout exported image `sha256:07a9dca99e084bbe132855a196e51ff443ae18273ce04a1e6821c00d92c77b4f` after `diskann-wide v0.54.0` compiled. | Local Docker `linux/arm64`; backend features `server-full,postgres-backend`. |
| bounded-truthful-handoff | PASS for the implementation and evidence prepared so far — strict OpenSpec and the child-scoped diff gate pass; parent certification is explicitly pending. | No runtime, deployment, soak, or cross-profile verdict. |

## Deterministic gates

- `openspec validate fix-container-rust-toolchain-pin-consistency --strict`:
  `Change 'fix-container-rust-toolchain-pin-consistency' is valid`; exit 0.
- `bash -n scripts/verify-runtime-image-toolchain-pin.sh`: exit 0.
- Positive source validator: exact dated values; exit 0.
- Refiner manifest and constraints: schema-valid.
- Manifest reference: present and non-empty.
- Global `git diff --check` reports only pre-existing blank-line changes in six
  unrelated KBD task projections; the child-permitted scoped diff has no error.

The refiner workflow dispatcher was attempted twice as required and failed
before trigger evaluation because its quoted Python heredoc passes the literal
`$EVENT_PAYLOAD` to `json.loads`. This artifact has no workflow triggers, so the
failure prevented no configured action. The canonical refiner scripts were not
patched in this release child.
