# Publish the existing v1.2.0 release

## Why
Issue #425 requires a source tag, GitHub release, unambiguous OCI index, integration changelog and honest acceptance status. The current main workflow moves 1.2.0 on every push.

## What changes
Preserve index sha256:dec5284e97b78ff65a66ef19eb43632154e23ea946f05a1d241cf979dd09566d, add v1.2.0 alias, bind the git tag to attested source 035bbdc702ba20daaa1add7294b35f5783ed6e0e and publish the Kubo integration contract. Remove main-branch version aliasing; future semantic releases publish both version spellings. A fixed-digest manual job handles package-write publication after local credentials were rejected. Pause deploy.yml around historical tag creation because that tag contains the old workflow.

## Capabilities
- release-provenance: retrospective source/image binding with explicit acceptance limitations.

## Uncomfortable limitation
The image predates the git tag. This release cannot satisfy a downstream requirement for a tag-triggered build. Functional acceptance stays pending in #421.
