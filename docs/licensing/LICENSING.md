# UAR Licensing Model

UAR is **MIT** licensed. That is the whole model.

- **Code** — runtime, SDKs, everything in this repository: `MIT` (see `LICENSE`)
- **Documentation** — `CC-BY-4.0` (see `LICENSE-CC-BY-4.0.md`)

You may use, modify, embed, redistribute, and offer UAR as a hosted service,
commercially or otherwise, with no obligation beyond preserving the copyright
notice. There is no commercial license to purchase and no copyleft trigger.

## Why this model

UAR is runtime-layer infrastructure, and it is designed to run **decentralized**
— on phones, laptops, and small always-on machines in homes and offices that
find and serve each other peer-to-peer. A license that makes people think before
running a node works against that directly. MIT removes the question.

## What changed, and what it cost

Until 2026-08-07 the runtime was `AGPL-3.0-only` with a separate commercial
exception. The copyleft was the commercial moat: network-deployed modifications
had to be offered back, and organizations who could not accept that bought their
way out.

**MIT removes that moat deliberately.** Anyone may now run and offer UAR as a
service with no obligation to Prometheus AGS. This is a real trade, made on
purpose: adoption of a decentralized runtime matters more than a licensing toll,
and a toll is unenforceable in the topology we are aiming at anyway.

## Where Prometheus AGS's commercial offering sits now

Not in the software — in **coordination**. The peer-to-peer data plane is fully
open and complete: any node can derive its identity, pair with another node,
verify credentials, and route work to a peer **using only MIT code and open
protocols, with no paid service and no account**.

What Prometheus AGS sells is the control plane around a fleet of nodes:

| Open (MIT), always | Commercial service |
|---|---|
| Node identity, pairing, LAN discovery | Finding your nodes across networks |
| Credential issuance and verification | Cross-organization trust brokering |
| Routing work to a paired peer | Mesh orchestration and fleet configuration |
| Per-node logs | Observability across the mesh |

The rule we hold ourselves to: **every paid feature must be a convenience over
something the open core can already do manually.** A directory service saves you
from exchanging addresses by hand; it is never the only way to pair. If a paid
feature ever becomes the *only* path to a capability, we have broken this rule.

## Contributor expectations

By contributing you agree your contribution is MIT. No CLA, no copyright
assignment, no dual-licensing clause. See `CONTRIBUTING.md`.

## Trademark note

Code licensing does not grant trademark rights. See `TRADEMARKS.md`.

## Disclaimer

This document is informational and not legal advice.
