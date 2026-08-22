# Install the Prometheus Skill Pack for UAR

UAR release archives already include the pack version they were built with.
Use this installer when adding or refreshing the pack on an existing UAR
installation without keeping a UAR or skill-system development checkout.

## Prerequisites

- Git with HTTPS access to GitHub.
- A Rust stable toolchain with `cargo` and `rustc` on `PATH`.

Install Rust from [rustup](https://rustup.rs/) when either Rust command is
missing. The installer does not run a remote bootstrap script or use `sudo`.

## Install

Run the script shipped with the matching UAR release:

```bash
bash scripts/install-uar-skill-pack.sh
```

The installer:

1. fetches the UAR-pinned commit from
   `https://github.com/Prometheus-AGS/prometheus-skill-system.git`;
2. rejects any checkout whose commit differs from that immutable pin;
3. builds the canonical `prometheus-cli` with `cargo build --locked --release`;
4. installs the pack atomically at
   `~/.config/uar/skills/prometheus-skill-pack/<version>/`; and
5. leaves the previous version available for rollback.

Restart UAR after installation. Its existing installed-plugin precedence
selects the highest installed pack version before the embedded release copy.
Installed pack skills have built-in origin, so the admin UI offers enable and
disable but not edit or delete.

To use a different cache prefix:

```bash
bash scripts/install-uar-skill-pack.sh --prefix /opt/uar/skills
```

When using a non-default prefix, start UAR with the installed skill directory:

```bash
UAR_BUILTIN_SKILLS_DIR=/opt/uar/skills/prometheus-skill-pack/1.7.0/skills \
  universal-agent-runtime
```

## Verify

After UAR restarts, list the API inventory used by the admin skills page:

```bash
curl --fail --silent http://127.0.0.1:3000/api/skills
```

For the pinned 1.7.0 pack, the default loader inventory contains exactly 147
skills and every returned pack row reports `"origin":"builtin"`. The installer
copies all 311 `SKILL.md` manifests and writes the verified source commit to
`UAR_PACK_COMMIT` in the version directory. Manifests under `skills/imported/`
remain behind UAR's existing opt-in boundary; set
`UAR_LOAD_IMPORTED_SKILLS=true` when that additional inventory is intended.

## Upgrade and rollback

Each UAR release pins the skill-system commit it was tested with. Upgrade UAR,
then rerun that release's installer. Do not edit the commit constant locally;
that would remove the verification boundary.

Because versions install side by side, rollback by removing the newer version
from the search root or by setting `UAR_BUILTIN_SKILLS_DIR` to the retained
older version's `skills/` directory. The installer never mutates user-created
skills stored through the UAR API.

For an offline machine, copy a clean checkout of the exact pinned commit and
run:

```bash
bash scripts/install-uar-skill-pack.sh --source-dir /media/prometheus-skill-system
```

The same commit check and locked Rust build still run.
