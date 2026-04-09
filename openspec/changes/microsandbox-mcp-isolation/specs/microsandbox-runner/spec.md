## ADDED Requirements

### Requirement: MicrosandboxRunner Implementation

The crate SHALL provide a `MicrosandboxRunner` struct that implements the `SandboxRunner` trait. It SHALL use the `microsandbox` crate (libkrun) to create and manage OCI-compatible microVMs. The implementation SHALL be feature-gated behind the `sandbox-microsandbox` Cargo feature.

#### Scenario: Creating a microVM sandbox
- **WHEN** `create` is called with a `SandboxConfig` specifying image `"prometheus-ags/sandbox-python:latest"`
- **THEN** the runner SHALL provision a libkrun microVM using that OCI image, apply the memory and CPU limits from the config, and return a valid `SandboxHandle`.

#### Scenario: Executing code inside a microVM
- **WHEN** `execute` is called with a `Language::Python` request containing `print("hello")`
- **THEN** the runner SHALL invoke `python3 -c 'print("hello")'` inside the microVM and return an `ExecutionResult` with `stdout` containing `"hello"`.

### Requirement: Named Volumes for Workspace Persistence

The `MicrosandboxRunner` SHALL support named volumes as specified in `SandboxConfig.volumes`. Named volumes SHALL be backed by host directories managed by the runner. Volume data SHALL persist across sandbox restarts within the same session.

#### Scenario: Volume data persists across sandbox destroy and recreate
- **WHEN** a sandbox writes a file to a named volume, is destroyed, and a new sandbox is created with the same volume name
- **THEN** the file SHALL be present in the new sandbox at the same mount path.

#### Scenario: Volumes are isolated between different names
- **WHEN** sandbox A mounts volume `"workspace-a"` and sandbox B mounts volume `"workspace-b"`
- **THEN** files written by sandbox A SHALL NOT be visible in sandbox B's volume.

### Requirement: Network Policy Per Sandbox

The `MicrosandboxRunner` SHALL enforce network policy based on the `network_enabled` field in `SandboxConfig`. When `network_enabled` is `false`, the microVM SHALL have no outbound network connectivity.

#### Scenario: Network disabled blocks outbound traffic
- **WHEN** a sandbox is created with `network_enabled: false` and code attempts an outbound HTTP request
- **THEN** the request SHALL fail with a network error inside the sandbox.

#### Scenario: Network enabled allows outbound traffic
- **WHEN** a sandbox is created with `network_enabled: true` and code attempts an outbound HTTP request to a reachable host
- **THEN** the request SHALL succeed.

### Requirement: Resource Limits Enforcement

The `MicrosandboxRunner` SHALL enforce memory and CPU limits specified in `SandboxConfig`. The microVM SHALL NOT be able to consume resources beyond its allocation.

#### Scenario: Memory limit enforced
- **WHEN** a sandbox is created with `memory_mib: 256` and code attempts to allocate 512 MiB of memory
- **THEN** the allocation SHALL fail or the process SHALL be killed by the microVM's OOM handler.

#### Scenario: CPU limit applied
- **WHEN** a sandbox is created with `cpus: 1`
- **THEN** the microVM SHALL be constrained to at most 1 virtual CPU.

### Requirement: OCI Image Pull and Cache

The `MicrosandboxRunner` SHALL pull OCI images from container registries when they are not locally cached. Pulled images SHALL be cached on disk to avoid redundant downloads on subsequent sandbox creations.

#### Scenario: First use pulls image from registry
- **WHEN** `create` is called with an image that is not in the local cache
- **THEN** the runner SHALL pull the image from the registry and cache it locally before starting the microVM.

#### Scenario: Cached image avoids re-download
- **WHEN** `create` is called with an image that is already cached locally
- **THEN** the runner SHALL use the cached image without making a network request to the registry.

### Requirement: Cleanup on Destroy

The `MicrosandboxRunner` SHALL fully clean up microVM resources when `destroy` is called. This includes stopping the VM process, releasing memory, and unmounting non-persistent volumes.

#### Scenario: Destroy stops the VM process
- **WHEN** `destroy` is called on an active sandbox handle
- **THEN** the libkrun microVM process SHALL be terminated and all associated OS resources SHALL be released.

#### Scenario: Destroy does not remove named persistent volumes
- **WHEN** `destroy` is called on a sandbox that uses a named volume
- **THEN** the named volume's data SHALL remain on disk for future reuse; only ephemeral resources SHALL be cleaned up.
