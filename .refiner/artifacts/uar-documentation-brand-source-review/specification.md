# Specification — `uar-documentation-brand-source-review`

- Artifact type: `content`
- Content type: `direct:content`
- Intent: evaluate the bounded Docusaurus branding source and its deterministic
  local controls without claiming rendered-site or deployment certification.
- Deterministic execution: required.
- Inputs: the brand OpenSpec bundle, Docusaurus config, homepage, CSS, static
  brand assets, package manifests, source validator, negative controls, current
  Web Interface Guidelines, and scoped git diff.

## Target state

- The portal uses the shipped UAR identity and exact dark/light surface ladders.
- Search and fonts are local, version-pinned, and require no hosted service.
- The homepage explains the trusted-host boundary, protocol surfaces, reader
  paths, and profile-specific evidence limits with semantic static composition.
- Source controls reject stock identity, asset drift, missing routes, hosted
  services, forbidden decoration, invisible focus, and missing reduced motion.
- Only the website npm lockfile changes, and runtime/application sources remain
  untouched.

## Unknowns and evidence limits

- The full Docusaurus build, browser screenshots, keyboard interaction,
  accessibility tree, and deployed search behavior are intentionally deferred
  until the complete documentation estate exists.
- `npm install` reports 20 high-severity advisories in the existing/transitive
  dependency graph. This bounded change does not run an unplanned dependency
  migration or claim the advisory set is resolved.

## Uncomfortable fact

Passing source controls does not prove that the branded portal renders well.
That claim remains unavailable until the final certification change builds and
inspects the complete site.
