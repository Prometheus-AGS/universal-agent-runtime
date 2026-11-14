# RC5 protocol migration

Deliver one signed Linux amd64 candidate from main containing #469 and the final AG-UI and A2UI 1.0 protocol upgrade. Adopt PEM 5.0.0 while retaining UAR's own renderer, policy, and durable action receipt flow. Keep legacy A2UI v0.9/v0.9.1 readable. Exclude A2UI remote function execution and third-party UI qualification.

## Implementation boundary

- PEM release gitlink, package pins, vendored upstream compatibility and all affected consumer locks.
- Rust A2UI publication, history and actions, frontend renderer, and AG-UI 1.0 transport.
- Official schema validation, mounted behavior coverage including legacy replay, operator documentation and release metadata.

## Execution order

Implement all lanes before verification. Record the exact source inventory. Run frozen installs, source review, Rust and protocol integration checks serially. Land the verified source on main, build the owned Linux amd64 image, mount the native concurrency and protocol gates, audit the same image and source, sign and publish, and verify the published index and tag.

The operator approved this implementation and bounded qualification in the RC5 release chat. The previous narrower RC5 job was stopped before it could publish. Release and verification scope is UAR. RC3 and RC4 are excluded.

## Exit evidence

- Official final JSON event schema and SDK validation cover the dedicated AG-UI stream. Public provider reasoning closes before user-visible text and terminal. Reconnect preserves the retained cursor and does not dispatch.
- One mounted A2UI scenario proves inline initial components and data, a subsequent data update, two distinct governed actions, same intent reconciliation, changed intent and undeclared-action refusals, explicit function refusal, and deletion replay. A second scenario proves original-dialect legacy replay. The published PEM 5 headless runtime processes actual events from the installed image.
- Both original #469 native concurrency examples pass on that same image. The provider and tool journals record the expected arrivals, one committed tool effect at revision 1, and clean shutdown.
- The unchanged local security audit passes for that source and artifact. The signed release tag binds the qualified Linux amd64 image digest and main commit, and GHCR serves identical image index bytes.
