# Design

## Context

The installed UAR Linux amd64 API behind a trusted host needs final AG-UI and A2UI 1.0 wire support while preserving replay, action admission, and chat recovery.

## Goals and scope

Validate the dedicated AG-UI route against the final schema for its supported events. Accept final A2UI surfaces and actions, preserve legacy history, and ship the native admission concurrency fix on the same artifact. Remote A2UI function execution and qualification of consumer apps and frameworks are outside scope.

## Decisions

The existing UAR runtime retains ownership of run admission and projections. It validates A2UI operation order and catalog, folds legacy patches and artifacts during replay, and checks owner, interaction, and surface generation when admitting actions. Durable receipts handle retry and recovery. The runtime and bundled renderer adopt the final A2UI version while the interoperability fixture uses the published PEM 5.0.0 headless consumer.

The source build pins a security-maintained PEM 5.0.0 successor. The exact recursive tree and image passed the existing release audit and the signed release records their identities.

## Risks

An interim compatibility patch may arrive before the final surface artifact. The replay fixture verifies the final dialect without dropping the earlier updates. A synthetic provider must issue distinct tool-call IDs for its separate operations on the same surface.
