# Positive verification — `ship-skill-pack-install-path`

## Public HTTPS pin

Command:

```bash
probe_dir=$(mktemp -d)
git -C "$probe_dir" init --quiet
git -C "$probe_dir" remote add origin https://github.com/Prometheus-AGS/prometheus-skill-system.git
git -C "$probe_dir" fetch --quiet --depth 1 origin c25561548aeb9ca656fdb942ab34378beedc2fe2
git -C "$probe_dir" rev-parse FETCH_HEAD
```

Observed output, exit 0:

```text
c25561548aeb9ca656fdb942ab34378beedc2fe2
```

## Real public build and install

Command:

```bash
CARGO_TARGET_DIR=/Users/gqadonis/Library/Caches/cargo-build/uar-skill-pack-real \
  scripts/install-uar-skill-pack.sh --prefix <temporary-prefix>
```

Observed final output, exit 0:

```text
Finished `release` profile [optimized] target(s) in 4m 53s
Installed prometheus-skill-pack 1.7.0 at <temporary-prefix>/prometheus-skill-pack/1.7.0
Verified commit: c25561548aeb9ca656fdb942ab34378beedc2fe2
Installed SKILL.md manifests: 311
```

Installed binary and receipt:

```text
prometheus 1.7.0
c25561548aeb9ca656fdb942ab34378beedc2fe2
311
```

This observed real build preceded a staging-path-only correction: staging moved
from inside the loader's scanned two-level root to a sibling directory on the
same filesystem. The final script's build inputs and command are unchanged. The
current post-correction shell and API replays below both pass.

## Final deterministic installer replay

Command:

```bash
bash scripts/tests/test-install-uar-skill-pack.sh
```

Observed output, exit 0:

```text
Installed prometheus-skill-pack 1.7.0 at <clean-prefix>/prometheus-skill-pack/1.7.0
Verified commit: c25561548aeb9ca656fdb942ab34378beedc2fe2
Installed SKILL.md manifests: 311
clean-prefix install PASS: version=1.7.0 skills=311
wrong-commit negative control PASS
failed-build negative control PASS
```

## Clean-prefix admin API inventory

The pinned pack contains 311 copied manifests. Under the default loader policy,
which deliberately excludes `skills/imported/`, the exact active inventory is
147 skills. The focused test requires that exact count and exact API ID set.

Command:

```bash
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --test skill_pack_install_path -- --test-threads=1
```

Observed tail:

```text
running 1 test
installed_pack_inventory=147
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.80s
```

## Change-level checks

Commands:

```text
cargo fmt --all -- --check
cargo check --locked -p universal-agent-runtime --no-default-features --features server-full
cargo clippy --locked -p universal-agent-runtime --no-default-features --features server-full --lib --no-deps
bash -n scripts/install-uar-skill-pack.sh scripts/tests/test-install-uar-skill-pack.sh
shellcheck scripts/install-uar-skill-pack.sh scripts/tests/test-install-uar-skill-pack.sh
openspec validate ship-skill-pack-install-path --strict
git diff --check -- .gitmodules scripts/install-uar-skill-pack.sh scripts/tests/test-install-uar-skill-pack.sh tests/skill_pack_install_path.rs docs/skill-pack-installation.md openspec/changes/ship-skill-pack-install-path
```

Observed results:

```text
cargo fmt: exit 0, no output
cargo check: exit 0 with 3 known warnings
cargo clippy: exit 0 with 574 warnings
bash -n: exit 0, no output
shellcheck: exit 0, no output
Change 'ship-skill-pack-install-path' is valid
git diff --check: exit 0, no output
```

Candidate SHA-256 values:

```text
6a6ffac689d017453fb30031f739555dcb5e136a00679142d5bfcfd24329e8a2  .gitmodules
68db85c0bc1a6de2626764957c3d85fb75b671a852710d84e26e17bad7a39aa6  scripts/install-uar-skill-pack.sh
cb0b085c756eb4f0c0d04b2cb453787d0c20c34fddc1dcdd156b33283d41bbc9  scripts/tests/test-install-uar-skill-pack.sh
a2f4a7c698f60e34146dc854300d1d1c28d74b52c349579685fa4c37a01525b7  tests/skill_pack_install_path.rs
5dc0c17405ec891d01e6743958fc56f4861e577f538c9171e2899afa0f361659  docs/skill-pack-installation.md
```

## Artifact-refiner and independent review

The current artifact and its finalized history snapshot were validated against
the artifact-manifest, constraints, and refinement-state schemas. Referenced
files existed, constraint IDs matched, and the final state was converged at
4/4 constraints.

Observed output, exit 0:

```text
.refiner/artifacts/ship-skill-pack-install-path: schemas=PASS references=PASS state=4/4 converged=PASS
.refiner/history/ship-skill-pack-install-path/2026-08-18_18-04-21Z: schemas=PASS references=PASS state=4/4 converged=PASS
```

The independent artifact critic returned PASS after the XDG-path and inventory
corrections. The independent judge also returned PASS on the corrected final
candidate. Their shared warning is commit scope only: exclude unrelated
operator and generated files.
