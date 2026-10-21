# Lifecycle deployment image receipt

Run: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39532243053

Production source: `32cb3bcfc9691212ec0b0359ce22474968bc6850`.
Matching Boss source pin: signed commit
`a609869079bf6d51af196944965e6f86ee6c0758`.
Routine verification remains **CLOSED**.

## Authorized command

```sh
/opt/homebrew/bin/gh workflow run deploy.yml --repo Prometheus-AGS/universal-agent-runtime --ref codex/uar-1-1-1-implementation
```

The run metadata confirmed the production SHA above before later documentation
commits. This is the same narrow Docker delivery exception. No separate local
Docker inspection, pull, smoke test, native build or product acceptance ran.

## ARM64 publication

Job119000651255 completed successfully after50m9s. Its production Cargo build
reported `Finished release profile [optimized]` after34m43s. The action then
pushed and exported this actual image index digest:

```text
ghcr.io/prometheus-ags/universal-agent-runtime@sha256:3ff38b564bed0e3d544448c262dc77a67dc957da2d2a141e99a618db30521f88
```

The action metadata names it `containerimage.digest`, and the subsequent digest
export step uses the same value. This is the ARM64 job's index, including its
provenance; it differs from the merged two-architecture index below. The observed build
durations are historical measurements, not completion deadlines.

## AMD64 and merged publication

AMD64 job119000614670 completed successfully after1h2m5s. Its production
Cargo build finished the optimized release profile after42m19s. Its action
exported this actual index digest:

```text
sha256:d76a02511d8dd566b063b9cf702f5d343161c70408a816d9ab8775ff67b9db3e
```

Manifest job119019008132 completed successfully after19s. Its existing
workflow inspection step reported the combined image:

```text
Name: ghcr.io/prometheus-ags/universal-agent-runtime:sha-32cb3bcfc9691212ec0b0359ce22474968bc6850
Digest: sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8
Platform: linux/arm64
Platform: linux/amd64
```

The index also contains the two provenance entries, reported as
`unknown/unknown` by that inspection. Fresh run metadata reports
`status: completed`, `conclusion: success`, and the same immutable source SHA.
This is actual workflow output; no separate local inspection was executed.

Use the immutable combined reference:

```text
ghcr.io/prometheus-ags/universal-agent-runtime@sha256:8e6fb6f453323fa921d956adb78d15b4ab4ae6137acb3cc5827a83bcd9bc1fc8
```

The first two delivery attempts failed and published no image. Their receipts
are `lifecycle-delivery-build-failure-39523808909.md` and
`lifecycle-delivery-build-failure-39528865081.md`. Their bounded source repairs
and renewed Boss pins are included in the production source above.

## Evidence limits

This proves both Linux deployment artifacts compiled and the combined image
was pushed. It does not
prove runtime startup, actual embedding inference/failover, database release,
migration races, consumer recovery, Windows input cancellation, native Boss
packaging or acceptance. The deferred exact commands remain separate.

The local observation process once exited after a GitHub API TLS handshake
timeout. A fresh metadata read confirmed the same run and both build jobs were
still active, and observation reattached to that run. The build was not
canceled or redispatched. That observer failure is not a build failure.
