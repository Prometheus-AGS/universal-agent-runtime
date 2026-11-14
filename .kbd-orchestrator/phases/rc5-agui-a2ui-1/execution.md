# Execute

OpenSpec is the spec and implementation backend. The lead owns integration, KBD, and the verification boundary. Rust A2UI, AG-UI, frontend protocol and renderer, and dependency preparation are separate writer lanes. No validation or builds may run until the full implementation inventory is recorded. Close after signed tag and published Linux amd64 digest are verified.

## Ownership

- A2UI host and wire projection: rc5_a2ui_rust.
- Dedicated AG-UI route and official schema: rc5_agui.
- Bundled renderer and application bridge: rc5_frontend.
- PEM gitlink, dependency manifests and locks: rc5_dependencies.
- Lead: mounted fixtures and acceptance, documentation, integration, inventory and release execution.

Independent source review identified guarded follow-up repairs for activity round-trip, complete terminal replay, cursor epochs, A2UI action declaration topology, reserved-data semantics and bundled artifact assembly. Passing frozen installs and scope checks are retained. Rust compilation was interrupted before acceptance when source repairs became necessary.
