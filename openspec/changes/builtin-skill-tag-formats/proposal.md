## Why
The operator explicitly requires compatibility with both metadata.tags formats present in shipped Boss skills. Source inspection found scalar comma-separated tags in packaged prometheus-ui-review, better-writing and better-typography SKILL.md, while UAR MetadataFrontmatter.tags accepts only Vec<String>. Native loading of those skills is not yet confirmed.

## What Changes
Accept scalar comma-separated tags and existing arrays in the builtin manifest parser. Reuse the existing untagged string/list frontmatter shape; scalar parsing trims comma-separated labels and omits empty separators, array parsing preserves values exactly. Preserve all tool, identity, version, digest, location and refresh behavior.

## Capabilities
### Modified Capabilities
- builtin-skill-manifests: shipped scalar/array metadata tag compatibility.

## Impact
Only src/uar/runtime/skills/builtin_loader.rs tag deserialization and scoped catalog documentation. Isolated future API payload branch; current frozen C14/Boss trees untouched. No dependency, storage, API, tool admission or refresh changes. No compiler/tests/build/review during source work; native confirmation remains lead-owned.
