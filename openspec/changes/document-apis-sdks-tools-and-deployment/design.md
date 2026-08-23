## Context

See `proposal.md` for motivation and
`specs/dev-portal-2026/spec.md` for the behavior contract. The frozen product
route inventory reserves protocol, tool, MCP-health, and settings documentation
routes. Current source additionally contains the server router, an OpenAPI 3.1
document and optional Swagger UI, three SDK source packages, generated Rust and
TypeScript reference staging, configuration examples, Compose stacks, a Helm
chart, and release/upgrade material.

The uncomfortable constraint is that source presence is not publication
evidence. The Python package has versioned metadata, examples, tests, and Sphinx
configuration, but the Pages assembler stages only rustdoc and TypeDoc. Package
metadata likewise does not prove crates.io, PyPI, or npm availability. The API
summary is intentionally smaller than the complete Axum router. Compatibility
adapters cover implemented subsets, not every upstream extension.

This is documentation-only. It owns API, protocol, SDK, tool, configuration,
installation, deployment, and upgrade guide roots plus a bounded manifest and
validator. It does not own runtime or React behavior, shared site
navigation/theme, the frozen route inventory, README reconciliation, dependency
changes, raw history, release publication, or Pages deployment. Full build and
browser certification remain deferred until all phase content slices finish.

## Goals / Non-Goals

**Goals:**

- Establish a fifteen-guide developer/operator sequence from interface
  selection through upgrade and rollback.
- Keep generated-reference availability, package source availability, registry
  publication, and runtime deployment as separate claims.
- Document every protocol and tool as an adapter to the shared runtime and
  trusted-host boundary, with profile, auth, state, and compatibility limits.
- Make invented routes, unsupported publication claims, unsafe configuration,
  profile transfer, and workflow-policy drift fail deterministic local controls.
- Preserve concise compatibility entry points for existing duplicate pages.

**Non-Goals:**

- Expanding OpenAPI coverage, changing wire protocols, SDK behavior, tool
  authorization, configuration precedence, packaging, or deployment behavior.
- Certifying registry publication, protocol conformance, provider compatibility,
  inference, runtime health, installation, migration, or rollback from prose
  checks.
- Publishing raw `.prometheus`, KBD, private evidence, credentials, payloads,
  or machine-local material.
- Running unit, integration, production build, browser, accessibility,
  deployment, or public Pages gates in this content change.

## Decisions

### 1. Publish fifteen current-authority guides in dependency order

Create or replace these guides:

1. `website/docs/api/index.md`
2. `website/docs/protocols/overview.md`
3. `website/docs/protocols/http-compatibility.md`
4. `website/docs/protocols/events-and-ui.md`
5. `website/docs/protocols/mcp.md`
6. `website/docs/protocols/a2a.md`
7. `website/docs/tools/overview.md`
8. `website/docs/sdks.md`
9. `website/docs/sdk-rust/intro.md`
10. `website/docs/sdk-python/intro.md`
11. `website/docs/sdk-typescript/intro.md`
12. `website/docs/configuration.md`
13. `website/docs/installation.md`
14. `website/docs/deployment.md`
15. `website/docs/upgrade-guide.md`

The sequence starts with the generated and narrative API boundary, separates
wire adapters from tool execution, then explains client libraries and the
operator path that makes those interfaces reachable. Existing
`api-reference.md`, `configuration/intro.md`, `dev-tools/intro.md`, and
`a2ui/intro.md` become concise compatibility/index pages pointing to current
authorities. `architecture/protocols.md` remains the conceptual architecture
boundary and links forward rather than duplicating wire behavior.

**Alternative considered:** retain the long endpoint catalog in one legacy API
page. Rejected because it cannot distinguish generated OpenAPI coverage from
additional source routes or keep five protocol state machines reviewable.

### 2. Add a classified developer-reference authority manifest

Add `docs/publication/developer-reference.json` with exactly the fifteen guide
IDs, files, stable routes, fixed order, profiles, source records, current
authorities, required markers, and diagram/prose requirements. It supplements
but does not edit `docs/publication/routes.json`; the frozen inventory proves
screen-route coverage while this manifest proves interface and deployment
depth.

Current authorities include `src/server.rs`, `src/uar/api/openapi.rs`, protocol
adapters, MCP/A2A and tool registries, SDK package metadata and source,
`src/config.rs` and CLI configuration, example configuration, Compose and Helm
material, staging scripts, and the deployment workflow. Every path is resolved
from the checkout. Absent `versions.toml` is not cited as inspected evidence.

**Alternative considered:** infer truth from existing narrative pages. Rejected
because several of those pages are the stale objects being replaced.

### 3. Treat the router, generated spec, and generated references as distinct layers

The API guide distinguishes:

- the actual server router and feature-gated routes;
- the embedded OpenAPI summary at `/api/openapi.json` and Swagger UI at
  `/api/docs` under the API-docs feature;
- hosted rustdoc and TypeDoc assembled into Pages;
- narrative protocol and resource guides.

The OpenAPI document is a supported discovery entry, not an assertion that it
enumerates every server route. Narrative examples use only routes confirmed in
current source and direct readers to runtime discovery for the running build.

**Alternative considered:** call the generated OpenAPI document exhaustive.
Rejected because current source mounts APIs that are not present in its path
map.

### 4. Describe compatibility adapters by translation boundary

The HTTP guide documents `/api/chat/completion`, `/v1/chat/completions`, and
`/v1/messages` as adapters into shared provider/model routing. The events guide
separates OpenAI chunks, the conformant `agui_spec` profile, deprecated legacy
`agui`, `dual`, normalized run events, A2UI declarative state, and entity-live
SSE. MCP and A2A receive separate pages because each adds discovery, lifecycle,
identity, and transport constraints.

Every guide states that compatibility is limited to implemented fields and
events. A successful request through one adapter does not certify another.

**Alternative considered:** group protocols by external standards body.
Rejected because operators need to follow the actual UAR translation and state
boundary, not a standards taxonomy.

### 5. Keep discovery, authorization, and execution separate for tools

The tools guide distinguishes built-in native tools, boot-discovered MCP tools,
catalog/schema visibility, normalized names, capability grants, Cedar/risk
decisions, human approval, host execution, and lifecycle events. It documents
disabled-by-default high-risk native tools and the local-only JWT proxy without
describing either discovery or proxy reachability as authorization.

**Alternative considered:** list tool names only. Rejected because the security
property is the path from advertised schema to trusted-host execution.

### 6. Report SDK status in four independent columns

Each SDK guide reports source package, supported mode, local build/reference
path, and public availability separately. Rust covers HTTP client plus optional
embedded modes. Python and TypeScript cover HTTP/stream clients. The Pages site
links hosted rustdoc and TypeDoc because the assembler stages them. Python links
its repository source and local Sphinx path but makes no hosted generated-
reference claim. Installation snippets are labeled for registry use only when a
published artifact is independently confirmed; source-checkout commands remain
available regardless.

**Alternative considered:** assume version `1.0.0` means all three registries
are live. Rejected because package metadata is an authoring input, not registry
evidence.

### 7. Consolidate configuration around authority and lifecycle

`configuration.md` becomes the current authority. It documents selection and
precedence, structured environment names, schema discovery, secret indirection,
provider/model configuration, persistence and feature constraints, runtime
settings, reload behavior, and profile ownership. Large field tables remain
only where confirmed against current structs and examples; otherwise the
running schema endpoint is the exact build-specific authority.

`configuration/intro.md` becomes a compatibility pointer. Examples that disable
authentication are bound to loopback development and never paired with a
non-local listener.

**Alternative considered:** preserve duplicated overview and reference pages.
Rejected because contradictory defaults and precedence are more dangerous than
one longer authority page.

### 8. Separate build, distribution, deployment, and health evidence

Installation covers prerequisites, source checkout, local profiles, and
artifact acquisition. Deployment covers pinned images, Compose, Helm,
persistence, secrets, ports, health/readiness, and deployment ownership.
Upgrade covers support boundaries, backup prerequisites, immutable version
selection, configuration comparison, functional verification, and rollback.

No page treats a tag, manifest, image name, generated site, or successful build
as proof of publication or service health. GitHub Actions remain limited to
assembling/deploying Pages and validating the deployed artifact; all routine
documentation controls run locally.

**Alternative considered:** fold release availability into install commands.
Rejected because a plausible command can silently become a false availability
claim.

### 9. Compose bounded fail-closed controls after content completion

Add `scripts/validate-documentation-developer-reference.mjs` and
`scripts/test-documentation-developer-reference.mjs`, expose root local
commands, and compose the validator into the existing publication entrypoint.
Validate manifest order, paths/routes, source classification, authority
existence, frontmatter, required boundary markers, local links, diagram prose,
compatibility pages, package/publication wording, profile vocabulary, and
deployment-only workflow language.

Observe isolated failures for at least a missing guide, unclassified authority,
invented endpoint, exhaustive-OpenAPI claim, complete-protocol-parity claim,
discovery-as-authorization claim, production JWT-proxy recommendation, hosted
Python reference claim, registry availability inferred from metadata, unsafe
anonymous listener example, profile evidence transfer, missing deployment
health/rollback boundary, and routine tests placed in Actions.

Execute controls only after all content is complete. The final production
build/browser gate stays in the phase's last change.

**Alternative considered:** rely on Docusaurus broken links and editorial
review. Rejected because neither observes the dangerous claims fail closed.

## Risks / Trade-offs

- **[Risk] Router source changes faster than narrative endpoint lists.** → Keep
  the guide at resource-family level, identify runtime OpenAPI as a summary, and
  validate named routes against classified source markers.
- **[Risk] Existing protocol documents disagree on versions or event names.** →
  Use current source and canonical specs for present tense, label historical
  documents as non-authoritative, and expose disagreements rather than merge
  them.
- **[Risk] Registry publication status changes after this commit.** → State
  observed repository/Pages facts and require a separately verified registry
  link before changing availability language.
- **[Risk] Configuration examples leak or normalize unsafe settings.** → Use
  placeholders, bind anonymous mode to loopback, and reject credential-shaped
  or unsafe listener examples.
- **[Risk] Readers transfer server behavior to embedded hosts.** → Require a
  profile boundary on every guide and keep host-owned transport/persistence
  explicit.
- **[Trade-off] Marker validation cannot prove runtime or protocol behavior.** →
  Limit verification to documentation accuracy and retain runtime certification
  as separate evidence.

## Migration Plan

1. Add and parse the fifteen-guide authority manifest; resolve every classified
   source path from the checkout.
2. Add validator/control composition and local package commands without running
   the controls before content completion.
3. Write API, HTTP/event, MCP/A2A, and tool guides.
4. Reconcile SDK guides and publication status.
5. Consolidate configuration, installation, deployment, upgrade, and rollback.
6. Convert duplicate entry pages to concise indexes and add category metadata
   and cross-links without changing shared navigation or the route inventory.
7. Audit all present-tense claims against current source, profiles, staged
   references, and deployment artifacts.
8. Run isolated controls, bounded TypeScript/source checks, strict OpenSpec,
   artifact-refiner content review, and permitted-surface audit; record row-form
   evidence.
9. Transition KBD, refresh the human handoff, and commit independently without
   pushing the partial phase.

Rollback is a revert of this documentation commit. No runtime state, public
API, dependency, registry, image, or deployed site changes in this unit.
