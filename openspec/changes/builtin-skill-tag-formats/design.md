## Source mismatch provenance
Read-only inspection of /Users/gqadonis/.claude/worktrees/afc-c10-boss/build/prometheus-payload/skills/prometheus-ui-review/SKILL.md:8, better-writing/SKILL.md:7 and better-typography/SKILL.md:7 found metadata.tags as scalar comma-separated text. The published payload release-manifest lists mini revision abc5a9bd6157e8b6fe602c6f889d5db1d0cb777a. UAR builtin_loader.rs MetadataFrontmatter currently declares tags:Vec<String>. This is source compatibility evidence, not a reproduced native loading failure or acceptance claim.

## Decision
Add field-specific deserialize_with for MetadataFrontmatter.tags, retaining serde default for omitted tags. Reuse AllowedToolsFrontmatter's existing untagged String/List representation solely as a syntax decoder. Do not invoke its allowed-tools conversion: tag labels split only on comma, preserving internal spaces. Scalar labels are trimmed and empty comma segments omitted; array values, order, whitespace and duplicates remain unchanged as before. Reject other YAML shapes through the same typed deserialization. Other fields remain untouched.

## Runtime and authority
Only metadata_tags changes for formerly rejected scalar manifests. The existing raw-manifest sha256, version selection, file location, requiredTools, builtin identity, persistence and host policy remain authoritative and unchanged. Reading or selecting a skill never grants filesystem/tool effects. Discovery/reload behavior is unchanged.

## Validation boundary
Source-only implementation and commit/push authorized. No Cargo/compiler/test/build/review commands run. No personal runtime data scanned. Lead must later inspect actual protected catalog and packaged registration from the completed next payload before claiming availability. Commit/push hooks suppressed under the explicit no-check boundary and reported as suppressed, not passing.
