## Why

Once Manifest + WASM builtins are loaded with `origin = Builtin`, users will see them in the Skills page. We need to communicate the distinction visually (no surprise 409s when they try to delete) and let users filter built-ins out when authoring their own.

## What Changes

- Skill row component shows a Built-in badge (lucide `Shield` icon + label) when `skill.origin === "Builtin"`.
- Delete button is disabled with a tooltip "System skill — cannot be removed" for built-ins.
- Filter chips above the list: `All / Built-in / User`, defaulting to `All`.
- Skill detail panel displays `kind` (Manifest / Wasm / Native) and any AOT-cached marker (e.g. shows `.cwasm` available for WASM skills).

## Acceptance

- Visual review matches the affordance described above.
- Clicking the disabled delete button shows the tooltip; no network request fires.
- Filter chips work and persist via URL query string.
