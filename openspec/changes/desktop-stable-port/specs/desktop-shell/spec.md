# desktop-shell

## ADDED Requirements

### Requirement: Desktop shell boots to a stable runnable state

The desktop shell SHALL boot to a stable, interactive state on macOS: the app
launches, the runtime initializes, and the local model can be selected without a
crash or an unrecoverable error. A missing or not-yet-downloaded model MUST
surface as an actionable state, never a hang or panic.

#### Scenario: cold launch reaches interactive state

- **WHEN** a macOS user launches the desktop app for the first time
- **THEN** the shell SHALL reach an interactive state where a local model can be
  selected and a run started, without crashing

#### Scenario: model not yet present is actionable

- **WHEN** the selected local model has not been downloaded yet
- **THEN** the shell SHALL present a download/progress state rather than hanging
  or emitting a terminal error

### Requirement: Local runs stream full A2UI ContentBlocks on the desktop shell

A local (on-device) run on the desktop shell SHALL stream the same `A2uiEvent`
ContentBlocks as a cloud run — text, thinking, tool-use, tool-result, citation,
memory, and skill — through the existing Tauri event forwarder, so the shell
renders full agentic operation locally, not just chat.

#### Scenario: local run emits agentic content blocks

- **WHEN** a macOS local run invokes a tool and produces reasoning
- **THEN** the shell SHALL receive and render `thinking`, `toolUse`, and
  `toolResult` ContentBlocks live, identical to a cloud run

#### Scenario: local run updates live runtime state

- **WHEN** a local run progresses through steps, tool calls, and completion
- **THEN** the runtime operations console SHALL update its runs/steps/tool-calls
  state live from the same event stream a cloud run uses
