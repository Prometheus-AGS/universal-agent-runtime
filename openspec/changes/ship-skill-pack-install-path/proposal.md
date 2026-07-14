## Why

Assessment C5: customers cannot install the prometheus skill system on a
fresh UAR install - network fetch is unimplemented, the submodule URL is
SSH-only, and no installer or toolchain bootstrap exists.

## What Changes

- Provide a supported install path: fetch the skill pack from its public
  repository (HTTPS), verify, build (with Rust toolchain bootstrap guidance),
  and install where UAR is installed.
- Admin skills UI lists every pack skill after install.

## Capabilities
### New Capabilities
- `skill-pack-distribution`

## Impact
Installer script/CLI, pack detection, .gitmodules URL, docs site.
