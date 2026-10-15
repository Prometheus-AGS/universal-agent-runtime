# Why
Kubo integration is blocked by missing A2A 1.0 gRPC, per-agent MCP grants, remote skill import and release acceptance in #379.

# What Changes
Finish all nine conformance items and four follow-up requirements, preserve documented legacy compatibility, enforce host-owned skill roles and per-agent MCP restrictions, import exact manifest content durably, correct defaults and publish verified v1.1.0.

# Capabilities
## New Capabilities
- kubo-runtime-conformance: authenticated protocol, skill and MCP contracts for Kubo integration.

# Impact
Runtime protocols, trusted host authorization, persistence, deployment configuration, documentation and release packaging. No dependency upgrades. A published but unaccepted image remains a release failure.
