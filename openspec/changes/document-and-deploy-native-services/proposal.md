## Why

Native packaging is not usable until operators can install, upgrade, inspect, troubleshoot, and verify it. The local macOS deployment also needs bounded genuine-model evidence through both the API and shipped UI.

## What Changes

- Document macOS, Linux, and Windows native deployment in README, product docs, and branded Docusaurus.
- Build and install the release server and React bundle locally.
- Verify health, UI/static assets, listeners, provider visibility, genuine inference, restart persistence, database access, graceful stop, and `.prometheus` logs.
- Supersede the obsolete native Alibaba seed/default with released `alibaba/qwen3.8-max` and repair only the observed malformed Alibaba credential reference so the installed service survives restart.
- Advance the `models.dev` and `liter-llm` gitlinks and refresh UAR's reviewed offline catalog snapshot so `/api/models` and the Models UI receive the release through the existing catalog architecture.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `native-service-deployment`: installed macOS release and verification evidence.
- `customer-documentation`: native installation, upgrade, uninstall, configuration, and troubleshooting.
- `documentation-publication-contract`: native deployment pages in the branded site.
- `product-validation-evidence`: bounded real-provider API/UI observations and separate platform claims.

## Impact

- Installs a user-level LaunchAgent and files beneath operator-approved home paths.
- Narrowly migrates the exact obsolete Alibaba values authorized by the operator; unrelated existing YAML and persisted provider settings remain unchanged.
- Does not deploy Linux or Windows, push, tag, publish, or create a PR.
