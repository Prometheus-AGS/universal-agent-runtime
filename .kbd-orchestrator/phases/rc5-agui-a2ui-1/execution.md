# Execute

OpenSpec is the spec and implementation backend. The lead owns integration, KBD, and the verification boundary. Rust A2UI, AG-UI, frontend protocol and renderer, and dependency preparation are separate writer lanes. No validation or builds may run until the full implementation inventory is recorded. Close after signed tag and published Linux amd64 digest are verified.

## Ownership

- A2UI host and wire projection: rc5_a2ui_rust.
- Dedicated AG-UI route and official schema: rc5_agui.
- Bundled renderer and application bridge: rc5_frontend.
- PEM gitlink, dependency manifests and locks: rc5_dependencies.
- Lead: mounted fixtures and acceptance, documentation, integration, inventory and release execution.

Independent source review identified guarded follow-up repairs for activity round-trip, complete terminal replay, cursor epochs, A2UI action declaration topology, reserved-data semantics and bundled artifact assembly. Passing frozen installs and scope checks are retained. Rust compilation was interrupted before acceptance when source repairs became necessary.

The first OrbStack Linux amd64 image passed both native-chat concurrency examples with one committed tool write. Mounted protocol acceptance stopped at surface deletion. The synthetic provider reused the publication tool-call ID in the deletion call, which conflicts with canonical history. The fixture now distinguishes publication and deletion IDs. The terminal assertion retains its expected result and reports the AG-UI error code on failure. Requalify the final committed candidate with the existing mounted scenarios and release audit.

The second image passed both native examples and the complete A2UI 1.0 scenario. The legacy example selected the first compatibility state-patch snapshot. The behavior check now reads the final published surface creation and requires the original version on both the live stream and replay. The standalone frontend lock was synchronized with the dependency updates on main.
