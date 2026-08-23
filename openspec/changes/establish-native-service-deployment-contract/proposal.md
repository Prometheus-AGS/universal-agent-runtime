## Why

UAR 1.0 has a production server and web application but no normative native-service contract. Installing it directly on a workstation would otherwise leave platform paths, listener exposure, shutdown, credentials, logs, upgrades, and evidence claims to ad hoc scripts. The current server also binds A2A gRPC independently of `server.host`, so an apparently local service can expose gRPC beyond the machine.

## What Changes

- Add a native-service deployment capability covering macOS LaunchAgent, Linux systemd, and Windows SCM packaging.
- Lock local-only defaults, platform paths, state preservation, `.prometheus` operational logs, non-destructive upgrades, and bounded verification.
- Define platform-specific evidence limits so template/compile checks are not reported as runtime deployment.

## Capabilities

### New Capabilities

- `native-service-deployment`: native service lifecycle, paths, security defaults, preservation, and verification boundaries.

### Modified Capabilities

- `product-validation-evidence`: require source/profile/platform/limit fields and genuine model evidence for native-service inference claims.

## Impact

- Establishes the contract consumed by the following four changes.
- Adds no runtime code or external dependency.
- Keeps GitHub Actions deployment-only and all phase verification local.
