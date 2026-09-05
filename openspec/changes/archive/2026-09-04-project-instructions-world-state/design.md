## Context

See [proposal.md](proposal.md) for motivation. Project instructions and world state cross the configuration, host runtime, context assembly, session, and governed file-read boundaries. The design must preserve capability inversion: only trusted host code reads files, records baselines, and commits state; contributors expose immutable prompt fragments.

## Goals / Non-Goals

**Goals:**

- Discover project instructions deterministically within an operator-trusted root.
- Represent environment, time, permissions, and project instructions as host-authority fragments with stable identities.
- Send a full snapshot when no trustworthy baseline exists and merge-patch deltas otherwise.
- Keep the same behavior across legacy and typed turn assembly.

**Non-Goals:**

- Remote instruction retrieval or model-controlled trust configuration.
- Replacing enforced policy with project instructions.
- Persisting process environment variables in world state.

## Decisions

### Host-owned discovery with an explicit trust allowlist

`ProjectInstructionsConfig` supplies filename candidates, project-root markers, and absolute trusted workspace roots. Discovery canonicalizes paths, selects the most specific trusted root containing the working directory, stops at the nearest configured marker, and loads files root-to-cwd. An override file replaces its base file in the same directory. The alternative—trusting the request working directory implicitly—was rejected because instruction files are an injection boundary.

Subtree instructions are admitted only after the governed read tool successfully reads a file in that subtree. This avoids scanning unrelated descendants and ties scope expansion to an observed host action.

### Immutable snapshots and host-committed baselines

`WorldStateRuntime` captures trusted host inputs into `WorldStateSnapshot` and gives context assembly an immutable contributor. The contributor cannot mutate the session. The runtime commits the selected `WorldStateUpdate` only after assembly chooses the execution path. This prevents shadow or failed assembly paths from advancing the baseline.

### Stable sections with RFC 7386 deltas

Environment, current time, permissions, and project instructions use stable section ids. The first contribution and every post-rewrite contribution render all sections. Later contributions generate RFC 7386 merge patches and omit unchanged sections. Arrays are replaced atomically; explicit null object values are rejected because RFC 7386 reserves null for deletion. A substituted `Clock` and an integer time bucket make granularity deterministic.

The alternative—diffing rendered text—was rejected because textual diffs are unstable, harder for agents to interpret, and cannot express section removal cleanly.

### Host authority remains below policy authority

Every world-state fragment uses `Authority::Host`, marker escaping, and the fixed `WorldState` prompt section. Project instruction contents never become `System` or `Policy` fragments. This preserves the existing authority ordering even when a file imitates policy syntax.

## Risks / Trade-offs

- **Trusted files can still contain harmful instructions** → They are opt-in, constrained to canonical trusted roots, escaped inside host markers, and remain subordinate to enforced policy.
- **A history reducer can remove the messages needed to interpret a delta** → The baseline records exact messages and falls back to a full render when they are absent or a rewrite is reported.
- **Large instruction trees consume context** → Discovery loads only the root-to-cwd chain plus subtrees actually accessed; world-state token reservation accounts for a possible full resend.
- **RFC 7386 cannot represent an explicit-null object member as a value** → Snapshot sections omit nullable members, and generation returns a typed error if that invariant is violated.

## Migration Plan

The feature is additive. Existing deployments default to no trusted workspaces, so project files are not read until the operator adds an absolute root. Rollback consists of removing trusted workspace entries or reverting the runtime wiring; no stored schema migration is required.
