# 17. Relicense the runtime from AGPL-3.0-only to MIT

Date: 2026-08-07

## Status

Accepted — supersedes [ADR-0002](0002-dual-license-agpl-mit.md).

## Context

ADR-0002 (2026-07-13) kept the runtime under `AGPL-3.0-only` plus a commercial
dual-license, reasoning that copyleft preserved "the open-source moat" while
permissive SDKs met market expectations. That reasoning held while UAR was
understood primarily as a server that organizations deploy.

It stops holding once decentralized peer-to-peer operation (C-23…C-27) becomes
default scope. The target topology is phones, laptops, and small always-on
machines in homes and offices, discovering each other and executing work on each
other's behalf with no intermediate server. In that setting:

- **The copyleft trigger is a friction point at exactly the wrong moment.** AGPL
  §13 obligations attach to network-available modifications. A person running a
  node at home should not have to reason about whether serving a request to
  their own phone constitutes distribution.
- **The moat was not enforceable in the target topology anyway.** Copyleft is
  enforced against identifiable deployers; a mesh of individually-owned devices
  is not that.
- **A mixed-license mesh is an adoption tax.** flint-realtime-fabric, which owns
  the P2P transport UAR consumes, is MIT. An AGPL runtime on an MIT transport
  forces every adopter to reason about the boundary.

Verified before deciding: all human commits are by one author (Travis James,
under four git identities; the remaining committers are `dependabot[bot]` and a
CI machine account), there are **no AGPL dependencies** in `Cargo.lock`, and all
four submodules are MIT. Nothing blocked the relicense.

## Decision

- License **all code** in this repository — runtime, SDKs, and tools — under `MIT`.
- Keep documentation under `CC-BY-4.0`.
- **Delete `LICENSE-COMMERCIAL.md`.** A commercial exception exists to sell
  relief from copyleft; under MIT there is nothing to be relieved of.
- **Remove the CLA-lite clause** from `CONTRIBUTING.md`. It existed solely so
  commercial licensees could receive the same functionality as AGPL users.
- Do **not** rewrite git history. Contributions made before this date were made
  under the previous terms; the relicense applies going forward.

## Consequences

**The commercial moat moves from the license to the service.** This is the
substantive cost and it should not be understated: anyone may now run, modify,
and offer UAR as a hosted service with no obligation to Prometheus AGS. The
durable commercial surface becomes the *control plane* — cross-network node
discovery, mesh orchestration, fleet configuration, cross-organization trust
brokering, and observability — while the *data plane* (identity, pairing, LAN
discovery, credential verification, peer routing) is complete and free.

This is the Tailscale/Headscale shape: an identical data plane in both cases,
with all differentiation in the control plane.

It also makes a rule load-bearing that would otherwise be a courtesy: **every
paid feature must be a convenience over something the open core can already do
manually.** Under MIT, a crippled free tier can be forked whole in an afternoon,
so the open core has to be genuinely complete for the commercial layer to have
any durability at all.

Secondary consequences:

- The Rust SDK's `embedded` feature no longer propagates a copyleft obligation.
- `tools/license-check.sh` now enforces MIT across **every** workspace crate.
  The previous version checked only the root manifest and the SDKs, and passed
  while `uar-jwt-proxy` and `mcp-server-fetch` were still AGPL-3.0-only.
- We continue to treat ourselves as a *manufacturer* under the EU Cyber
  Resilience Act (`SECURITY.md`). The relicense changed the software's terms,
  not our security obligations to the people running it.
