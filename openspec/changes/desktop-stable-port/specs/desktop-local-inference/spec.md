# desktop-local-inference

## ADDED Requirements

### Requirement: macOS build floor supports MLX

The desktop build SHALL target a macOS deployment floor of 14.0 or higher so
the in-process MLX (`mlex`) engine compiles and links. The forced
`MACOSX_DEPLOYMENT_TARGET=10.15` inherited from whisper.cpp/llama.cpp MUST be
removed from both the Rust and Tauri cargo configs.

#### Scenario: mlex engine compiles on macOS

- **WHEN** the desktop app is built on macOS with the `local-mlxc` feature enabled
- **THEN** the `mlex` (mlx-c) dependency SHALL compile and link without a
  `MLX requires macOS >= 14.0` CMake error

#### Scenario: whisper/llama do not pin the macOS floor

- **WHEN** the macOS build graph is resolved
- **THEN** neither whisper.cpp nor llama.cpp SHALL be present on the macOS target,
  and no build setting SHALL force `MACOSX_DEPLOYMENT_TARGET` below 14.0

### Requirement: macOS local inference uses the in-process MLX engine

On macOS the desktop app SHALL provide local inference through the in-process
`MlxcEngine` (`mlex`) behind the shared `InferenceProvider` seam, using the same
catalog, pinned/SHA-verified downloads, and memory preflight as the mobile MLX
lane. Model files MUST be downloaded and verified in Rust; no Python runtime is
required.

#### Scenario: model download and load

- **WHEN** a macOS user selects the default local model and starts a run
- **THEN** the engine SHALL download every catalog-pinned file, verify each
  against its SHA-256, load the model, and reach a Ready state before generating

#### Scenario: checksum mismatch is rejected

- **WHEN** a downloaded model file's SHA-256 does not match the catalog digest
- **THEN** the engine SHALL reject the file and re-download rather than load
  corrupt or mismatched weights

#### Scenario: memory preflight rejects an oversized model

- **WHEN** a model's estimated peak memory exceeds the device budget
- **THEN** the engine SHALL refuse to load it (or downgrade to a smaller catalog
  entry) rather than crash the app

### Requirement: Per-platform desktop engine matrix

The desktop app SHALL select its local inference engine per target: macOS uses
MLX (`mlex`); Windows and Linux use llama.cpp. Engine selection MUST be driven
by per-target Cargo features so a single codebase builds the correct lane for
each platform.

#### Scenario: macOS selects MLX

- **WHEN** the desktop app is built and run on macOS (Apple Silicon)
- **THEN** the active local `InferenceProvider` SHALL be the MLX (`mlex`) engine

#### Scenario: Windows and Linux select llama.cpp

- **WHEN** the desktop app is built and run on Windows or Linux
- **THEN** the active local `InferenceProvider` SHALL be the llama.cpp engine

### Requirement: Reasoning is split from content on the desktop lane

The macOS local lane SHALL emit extended-reasoning tokens as `ThinkingDelta`
(via the model's token classification / reasoning format) so reasoning renders
as a distinct thinking ContentBlock rather than inline assistant content,
matching the mobile behavior.

#### Scenario: reasoning renders as a thinking block

- **WHEN** a macOS local run produces a model whose output includes reasoning
  tokens
- **THEN** those tokens SHALL surface as a `thinking` ContentBlock (collapsible
  in the shell), and the assistant reply content SHALL NOT contain the raw
  reasoning text
