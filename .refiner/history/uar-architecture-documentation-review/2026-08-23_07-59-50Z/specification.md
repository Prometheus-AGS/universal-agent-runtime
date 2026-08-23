# Specification — `uar-architecture-documentation-review`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: evaluate the seven-guide UAR architecture publication as a bounded
  source artifact without claiming rendered-site, accessibility, deployment,
  runtime, or cross-profile certification.
- Deterministic execution: required for manifest, provenance, structure,
  navigation, sanitization, and negative-control evidence.
- Inputs: the architecture publication manifest, seven public architecture
  guides, their classified OpenSpec records and current source authorities, the
  architecture validator and controls, the publication controls, and the scoped
  git diff.

## Target state

- Seven guides form one conceptual sequence from purpose and trust boundaries
  through lifecycle, state, profiles, protocols, and current delegation.
- Every guide identifies its classified source record and current public
  authority, and every present-tense claim is bounded to delivered behavior.
- `server-full`, `minimal`, and `embedded-mobile` are described separately; no
  evidence or capability claim silently transfers across profiles.
- Protocol adapters and graph execution remain inside the trusted-host
  capability boundary rather than becoming alternate authority models.
- Public content excludes raw history, KBD payloads, machine-local paths,
  credentials, private proposal text, and claims based on absent
  `versions.toml`.

## Unknowns and evidence limits

- The complete Docusaurus build, rendered diagrams, browser links, responsive
  behavior, accessibility tree, and deployed Pages routes are intentionally
  deferred until all documentation content is complete.
- This review validates documentation against the current checkout. It does not
  execute the runtime and cannot certify any runtime profile.

## Uncomfortable fact

Source-valid architecture prose can still mislead if the implementation changes
without updating its authority manifest. The final site gate proves rendering
and publication, not permanent synchronization with future runtime changes.
