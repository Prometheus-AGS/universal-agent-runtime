## ADDED Requirements

### Requirement: Intel macOS sidecar preserves local embeddings

The `darwin-x64` Boss sidecar release SHALL build with `server-full` and the opt-in `tract-embeddings` backend. The backend SHALL be initialized before sidecar startup. Other Boss sidecar platforms SHALL continue to build with `server-full` and the native ONNX Runtime backend.

#### Scenario: Intel runner has no upstream ONNX Runtime prebuilt

- **WHEN** the Intel macOS release job builds `uar-sidecar`
- **THEN** it builds using the tract CPU backend without requesting the unavailable `ort-sys` prebuilt binary
- **AND** the packaged model assets remain present.

### Requirement: Sidecar records declare their build features

The sidecar archive manifest and release record SHALL contain the feature set selected by the native release job.

#### Scenario: Intel artifact is published

- **WHEN** the `darwin-x64` archive and record are generated
- **THEN** both declare `server-full` and `tract-embeddings`.
