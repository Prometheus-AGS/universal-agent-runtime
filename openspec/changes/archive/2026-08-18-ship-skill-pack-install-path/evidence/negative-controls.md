# Negative controls — `ship-skill-pack-install-path`

## Wrong source commit

The deterministic installer test cloned the pinned source locally, created and
committed one extra marker, then invoked `--source-dir` on that clean but wrong
commit.

Observed result:

```text
skill-pack install: source commit <different-sha> does not match UAR pin c25561548aeb9ca656fdb942ab34378beedc2fe2
wrong-commit negative control PASS
```

The installer exited nonzero before invoking Cargo or creating an active version.

## Locked build failure

The test replaced Cargo with an executable that exits 42 and invoked the exact
pinned clean source.

Observed result:

```text
failed-build negative control PASS
```

The installer exited nonzero and
`<failed-build-prefix>/prometheus-skill-pack/1.7.0` did not exist. This proves a
failed build does not expose a partially populated version to UAR.
