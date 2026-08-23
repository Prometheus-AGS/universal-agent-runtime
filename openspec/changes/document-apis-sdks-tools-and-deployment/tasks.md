## 1. Establish the developer-reference contract

- [x] 1.1 Add `docs/publication/developer-reference.json` with exactly the fifteen planned guides, fixed order, stable routes, profiles, classified records, current authorities, required boundary markers, and diagram requirements; verify the manifest parses and every referenced source path exists.
- [x] 1.2 Add the bounded developer-reference validator and isolated control script, register their local package commands, and compose the validator into the final publication entrypoint; syntax-check the scripts but defer executing controls until all guides and compatibility pages are complete.

## 2. Document API and HTTP compatibility

- [x] 2.1 Replace `api/index` with a map of narrative, embedded OpenAPI/Swagger, hosted rustdoc, and hosted TypeDoc references; verify it states that OpenAPI is a summary rather than an exhaustive route inventory and does not advertise a hosted Python generated reference.
- [x] 2.2 Add `protocols/overview` and `protocols/http-compatibility` covering UAR, OpenAI-compatible, and Anthropic-compatible routes, model addressing, authentication, sessions, streaming selection, error/extension limits, and profiles; verify each named route exists in current source and no adapter claims complete upstream parity.

## 3. Document event, MCP, and A2A protocols

- [x] 3.1 Add `protocols/events-and-ui` covering normalized run events, OpenAI chunks, `agui_spec`, deprecated legacy `agui`, `dual`, A2UI declarative state, entity-live SSE, replay boundaries, and profile ownership; verify live state is not represented as interchangeable durable history.
- [x] 3.2 Add `protocols/mcp` covering configured remote servers, UAR and memory MCP endpoints, discovery, namespacing, health, reconnect and execution boundaries, authentication, and profiles; verify MCP discovery is not described as authorization.
- [x] 3.3 Add `protocols/a2a` covering feature-gated JSON-RPC, agent card, registry, optional gRPC transport, verified tenant construction, task/context partitioning, cancellation, and compatibility limits; verify older narrative version claims do not override current source.

## 4. Document tools and trusted-host execution

- [x] 4.1 Add `tools/overview` covering native and MCP tools, catalog/schema discovery, normalized names, disabled-by-default high-risk tools, capability/Cedar/risk/approval gates, host execution, lifecycle events, and profile limits; verify discovery never implies authorization.
- [x] 4.2 Document `uar-jwt-proxy` as a loopback-only development aid within the tools guide and compatibility page; verify the portal explicitly rejects its use as a production authentication gateway.

## 5. Reconcile the SDK guides

- [x] 5.1 Reconcile `sdks` and `sdk-rust/intro` against the Rust package features, HTTP client, embedded modes, examples, local commands, hosted rustdoc, and profile boundaries; verify package metadata is not represented as registry availability.
- [x] 5.2 Reconcile `sdk-python/intro` against its source, async HTTP/SSE client, examples, Sphinx configuration, and local checkout workflow; remove the hosted generated-reference claim and verify no PyPI availability is inferred from version metadata.
- [x] 5.3 Reconcile `sdk-typescript/intro` against its fetch/SSE client, schemas, examples, local commands, and hosted TypeDoc staging; verify npm availability is not inferred from package metadata.

## 6. Consolidate configuration authority

- [x] 6.1 Reconcile `configuration.md` against current config structs, CLI selection, example YAML, schema and reload endpoints, provider/model routing, persistence features, settings API, secret handling, precedence, and profile limits; verify every example uses placeholders and anonymous mode is loopback-only.
- [x] 6.2 Convert `configuration/intro.md` to a concise compatibility page and verify it points to the single current authority without retaining contradictory defaults or precedence.

## 7. Document installation, deployment, and upgrades

- [x] 7.1 Reconcile `installation` with source prerequisites, submodules, locked package-manager commands, server profiles, embedded host requirements, source SDK use, and release-artifact boundaries; verify a tag or manifest is not treated as publication proof.
- [x] 7.2 Reconcile `deployment` with pinned images, Compose, Helm, secrets, persistence, ports, health/readiness, profiles, platform limits, and deployment ownership; verify no floating production pin or unsafe anonymous non-local listener is recommended.
- [x] 7.3 Reconcile `upgrade-guide` with support boundaries, backup prerequisites, immutable versions, configuration comparison, datastore ownership, functional verification, and rollback; verify build success is not represented as deployment health or data compatibility proof.

## 8. Reconcile compatibility routes and truth

- [x] 8.1 Convert `api-reference`, `dev-tools/intro`, and `a2ui/intro` to concise compatibility/index pages, add protocol/tool category metadata and dependency-order cross-links, and verify shared navigation and `docs/publication/routes.json` remain unchanged.
- [x] 8.2 Audit every present-tense interface, SDK, configuration, and deployment statement against current source and canonical OpenSpec; qualify stale narrative conflicts, generated-reference limits, registry status, protocol subset behavior, and profile boundaries while `versions.toml` remains uncited as inspected authority.
- [x] 8.3 Audit public content for raw `.prometheus` or KBD material, machine-local paths, credentials, private keys, raw payloads, floating production pins, unsafe commands, and unsupported certification claims; verify only reviewed current-source synthesis remains.

## 9. Verify only after developer-reference content is complete

- [x] 9.1 Run isolated developer-reference controls and observe missing guide, unclassified authority, invented endpoint, exhaustive OpenAPI, complete protocol parity, discovery-as-authorization, production proxy, hosted Python reference, registry-from-metadata, unsafe anonymous listener, profile transfer, missing deployment health/rollback, and routine-GitHub-test controls fail before the complete source fixture passes.
- [x] 9.2 Run the Docusaurus TypeScript check and bounded publication, architecture, brand, product-workflow, security/operations, developer-reference, staging, and GitHub Actions policy controls locally, recording commands and output without running the phase-level production build or browser/accessibility certification.
- [x] 9.3 Run `openspec validate document-apis-sdks-tools-and-deployment --strict` and the artifact-refiner content gate; correct findings until the OpenSpec bundle, refiner schemas, referenced evidence, and finalized active/history state pass.
- [x] 9.4 Audit tracked and untracked diffs for runtime, React application, provider behavior, dependency, vendored, frozen route/navigation, README, lockfile, raw `.prometheus`, release, and deployment-workflow changes; remove anything outside this change's documentation/process surface.
- [x] 9.5 Write row-form `verification.md` with requirement, command, observed result, limit, source SHA, and documentation profile, explicitly deferring production build, visual, accessibility, deployment, protocol-conformance, registry, runtime-health, and cross-profile claims.
- [x] 9.6 Transition the registered KBD change through the canonical runtime, refresh the human handoff, verify the exact next command names `reconcile-uar-readme-estate`, and commit the complete change independently without pushing the partial phase.
