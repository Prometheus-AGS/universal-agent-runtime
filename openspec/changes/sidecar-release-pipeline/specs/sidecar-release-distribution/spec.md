## Purpose

Defines how the `uar-sidecar` binary is built for release and published for desktop hosts: the release profile, the tag-only deployment workflow, the targets and features, archive contents and checksums, recorded sizes, and the local build that precedes a tag.

## ADDED Requirements

### Requirement: Release builds use the release profile
Release builds of UAR SHALL use thin link-time optimization, one code-generation unit, and stripped symbols. Release builds SHALL keep panic unwinding while the runtime contains panics at run, tool or sandbox boundaries.

#### Scenario: Profile values
- **WHEN** the release profile in the package manifest is read
- **THEN** it sets thin LTO, one codegen unit and symbol stripping, and does not set panic to abort

### Requirement: The sidecar release workflow runs only on a release tag
The sidecar release workflow SHALL be triggered only by pushing a tag of the form `uar-sidecar-v<version>`, and SHALL fail before building when `<version>` differs from the package version. It SHALL NOT run on branch pushes, pull requests, schedules or manual dispatch.

#### Scenario: Branch push
- **WHEN** a commit is pushed to `main` or any other branch
- **THEN** the sidecar release workflow does not run

#### Scenario: Mismatched tag
- **WHEN** tag `uar-sidecar-v9.9.9` is pushed while the package version is `1.0.0`
- **THEN** the workflow fails before any build job starts and publishes nothing

### Requirement: The workflow builds without testing
The workflow SHALL build `uar-sidecar` for `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` and `aarch64-pc-windows-msvc`, each on a native runner of that platform, with default features disabled and exactly the features `minimal`, `local-models`, `document-intelligence` and `wasm-runtime`. Its build commands SHALL compile no test targets, and the workflow SHALL run no test, lint, format, type or coverage check and SHALL NOT execute the built binary. The repository's GitHub Actions policy validator SHALL pass with the workflow present.

#### Scenario: Policy validation
- **WHEN** the GitHub Actions policy validator runs on a checkout containing the workflow
- **THEN** it passes, and it fails if the workflow gains a test command, a branch or dispatch trigger, a different feature set, or a step that executes the built binary

### Requirement: The release policy names the sidecar exception
The repository's release-verification policy SHALL state that tag-triggered, test-disabled release builds of the sidecar are deployment, not development verification, and are the one exception to the rule that GitHub Actions are not a release build runner. The policy validator's allowlist SHALL name the sidecar release workflow and SHALL keep every existing rule.

#### Scenario: Reading the policy
- **WHEN** a maintainer reads the release-verification policy
- **THEN** it names the sidecar release workflow as a deployment-only exception and states that it runs no tests

### Requirement: Each target is published as a checksummed archive
For each target the workflow SHALL publish one archive containing the `uar-sidecar` binary, the on-device embedding model assets the `local-models` feature loads, and the repository license files, together with a `.sha256` file holding that archive's SHA-256 digest. The GitHub Release SHALL be created only when every target in the matrix produced its archive and checksum.

#### Scenario: Checksum matches
- **WHEN** a published archive's SHA-256 digest is computed
- **THEN** it equals the digest in the archive's `.sha256` file

#### Scenario: One target fails
- **WHEN** one target's build fails
- **THEN** no GitHub Release is created for the tag

### Requirement: Release notes record binary sizes
The release notes SHALL list, per target, the size in bytes of the `uar-sidecar` binary and of its archive. No size limit SHALL fail the release.

#### Scenario: Published release
- **WHEN** a sidecar release is published
- **THEN** its notes contain one size row for each of the six targets

### Requirement: A local build precedes each release tag
Before a release tag is pushed, a maintainer SHALL build the sidecar locally with the same command, target selection and features the workflow uses, following documented guidance, and SHALL record the result. The workflow is not the place where build breakage is first discovered.

#### Scenario: Pre-tag build
- **WHEN** a maintainer prepares a release tag
- **THEN** the documented local build command matches the workflow's build command and its dated result is recorded before the tag is pushed
