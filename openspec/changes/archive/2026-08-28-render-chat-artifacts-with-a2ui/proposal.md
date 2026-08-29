## Why

Chat display artifacts are currently presented as a single text component. When
the runtime emits structured policy data, the artifact window therefore shows a
large serialized JSON blob instead of an A2UI surface. The display path also
bypasses the canonical `@prometheus-ags/a2ui-core` message processor and
`@prometheus-ags/a2ui-uar` renderer already maintained by this repository.

## What Changes

- Emit the effective run policy artifact as current-production A2UI v0.9.1
  `createSurface`, `updateComponents`, and `updateDataModel` messages.
- Process chat display artifacts with the official A2UI `MessageProcessor` and
  render the resulting surface models with `UarSurface`.
- Keep invalid or unsupported source inspectable behind a bounded disclosure
  instead of presenting it as a successful surface.
- Recognize current v0.9.1 message names when A2UI arrives through custom AG-UI
  events.

## Acceptance

- The effective run policy artifact renders a structured policy summary and
  does not expose its serialized message stream in the default view.
- The chat application imports the canonical UAR A2UI packages rather than
  recreating display components locally.
- Invalid, unsupported-version, and unknown-component artifacts fail closed and
  expose their source only through an explicit disclosure.
- Focused Rust and React regressions cover the emitted message sequence and the
  rendered/fallback states.
