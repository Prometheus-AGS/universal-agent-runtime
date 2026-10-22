# Release design

Use the existing complete OCI index, preserving both architecture images and attestations. Both attestations name 035bbdc702ba20daaa1add7294b35f5783ed6e0e; successful run39696576430 used that commit. The earlier75bf899b/run39690429050/acb2f941 index is historical evidence, not the canonical index. Operator explicitly approved the corrected source tag.

The lead owns deploy.yml, OpenSpec and phase records. Documentation teammate owns CHANGELOG.md and docs/releases/v1.2.0-kubo.md. All product verification remains CLOSED. Operator separately authorized policy commands and commit hooks only.

Disable deploy.yml before pushing the documentation/workflow commit, push v1.2.0 at the historical attested source while disabled, and restore its original active state in a finally path. This prevents both unnecessary rebuild and historical-tag overwrite. Dispatch only publish_v1_2_0_alias=true on main after restoration. Publish GitHub release and contract asset after alias delivery. Existing phase acceptance tasks remain unchanged.
