## ADDED Requirements

### Requirement: Container lock matches committed path dependencies

The committed Cargo.lock SHALL satisfy the manifests at the repository's pinned
path dependency and submodule revisions. Deployment builds SHALL retain locked
Rust resolution on linux/amd64 and linux/arm64.

#### Scenario: Pinned dependency raises its minimum version

- **WHEN** a committed path dependency requires a version outside the locked range
- **THEN** the lockfile is reconciled before publication, without removing
  --locked from the container build

#### Scenario: Native image publication

- **WHEN** the lock repair is delivered through the deployment workflow
- **THEN** both native architecture builds succeed and their digests are assembled
  into the published multi-platform manifest

#### Scenario: Pinned provider error representation

- **WHEN** the pinned provider library represents Network as a struct variant
- **THEN** UAR matches that representation and preserves transport-error
  classification in the production image build
