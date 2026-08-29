## ADDED Requirements

### Requirement: Chat display artifacts are rendered from production A2UI messages

The chat application SHALL parse display artifact content as A2UI v0.9.1
messages, process those messages through `@prometheus-ags/a2ui-core`'s
`MessageProcessor`, and render the resulting surface models through
`@prometheus-ags/a2ui-uar`'s `UarSurface`.

#### Scenario: The runtime publishes an effective policy surface

- **WHEN** a run starts and publishes its effective run policy artifact
- **THEN** the artifact content contains `createSurface`, `updateComponents`,
  and `updateDataModel` messages using A2UI v0.9.1 and the certified UAR catalog
- **AND** the chat window renders the policy as structured native components
  rather than displaying the serialized message stream

#### Scenario: A display artifact is invalid or unsupported

- **WHEN** the artifact cannot be parsed, uses an unsupported protocol version,
  or references an unapproved component
- **THEN** the application reports that the surface is invalid
- **AND** it does not render arbitrary markup or treat the source as a successful
  text surface
- **AND** the original source remains available through a bounded disclosure

#### Scenario: A current A2UI message arrives through a custom AG-UI event

- **WHEN** a custom AG-UI payload contains `createSurface`,
  `updateComponents`, `updateDataModel`, or `deleteSurface`
- **THEN** the payload is recognized as A2UI v0.9.1 traffic and routed to the
  display artifact path
- **AND** messages for the same surface are accumulated in order so component
  and data updates share the surface created by the first message

#### Scenario: An artifact exceeds the client rendering budget

- **WHEN** an artifact exceeds the configured source-byte, message, component,
  or surface limit
- **THEN** the application rejects it before unbounded synchronous rendering
- **AND** preserves the source behind the same bounded diagnostic disclosure
