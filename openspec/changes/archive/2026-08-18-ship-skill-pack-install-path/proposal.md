## Why

Assessment C5: customers cannot install the prometheus skill system on a
fresh UAR install - network fetch is unimplemented, the submodule URL is
SSH-only, and no installer or toolchain bootstrap exists.

## What Changes

- Provide a supported install path: fetch the skill pack from its public
  repository (HTTPS), verify the UAR-pinned commit, build the canonical CLI
  after a Rust toolchain preflight, and atomically install into UAR's existing
  versioned installed-plugin search root.
- Admin skills UI lists the complete loader-eligible pack inventory after
  install, while preserving the existing opt-in boundary for imported skills.

## Capabilities
### New Capabilities
- `skill-pack-distribution`

## Impact
Installer script/CLI, pack detection, .gitmodules URL, docs site.
