## Implementation

- [x] Add an opt-in tract embedding backend against the pinned `ort` API and initialize it before sidecar startup.
- [x] Build the Intel macOS sidecar with `server-full,tract-embeddings`; retain `server-full` elsewhere.
- [x] Record the selected feature set in the archive manifest and release record.
- [ ] Complete the native Intel release build and verify its published archive and record.
