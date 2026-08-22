# Universal Agent Runtime — Converged Specification

**Status:** Authoritative. Supersedes conflicting claims in any other `docs/` file.
**Version:** UAR 1.0.0 · **Written:** 2026-08-07
**Method:** triangulation across the runtime's own source, three consumer applications, and
three partner projects. See §10 for how this was derived and what it cannot prove.

---

## 0. Why this document exists

No accurate specification existed. `docs/` held 140 markdown files, some dated, some
contradicting the code. The code was authoritative for *what exists* but silent on *what is
intended*. Neither source alone could answer "is UAR done?"

This spec is derived from three independent sources, reconciled:

| Source | Authoritative for |
|---|---|
| **UAR source** | what exists and works |
| **Consumers** — KnowMe, BossFang, San Saba | what is actually required and planned |
| **Partners** — flint-gate, flint-realtime-fabric, flint-forge | what is deliberately *not* UAR's |

Every capability below cites evidence. Claims verified against source are marked **[V]**.

---

## 1. What UAR is

An embeddable agent runtime that executes agents, routes models, manages knowledge and
memory, governs tools, and streams structured events — **on mobile, desktop, and cloud from
one codebase**, with nodes able to find and serve each other **peer-to-peer**.

**License: MIT** (relicensed from `AGPL-3.0-only` on 2026-08-07; see
[ADR-0017](adr/0017-relicense-runtime-to-mit.md)). This is architecturally load-bearing rather
than administrative. AGPL's copyleft was the commercial moat, and it created a friction point
exactly where C-23/C-24 need none: a person running a node at home should not have to reason
about whether serving a request to their own phone constitutes network distribution. It also
removed a mixed-license boundary — flint-realtime-fabric, which owns the peer transport UAR
consumes, is MIT.

**Consequence, stated plainly:** anyone may now run, modify, and offer UAR as a service with
no obligation. The durable commercial surface moves to the *control plane* — cross-network
node discovery, mesh orchestration, fleet configuration, cross-organization trust brokering,
observability — while the *data plane* (identity, pairing, LAN discovery, credential
verification, peer routing) is complete and free. This is the Tailscale/Headscale shape.

That makes one rule commercially load-bearing rather than a courtesy: **every paid feature
must be a convenience over something the open core can already do manually.** Under MIT a
crippled free tier can be forked whole in an afternoon, so the open core has to be genuinely
complete for the commercial layer to have any durability at all.

### 1.1 The portability constraint is the defining architectural fact

**[V] UAR has zero hard dependencies on any partner project.** `grep` over `src/` and
`Cargo.toml` finds 15 `flint-*` references, all doc comments describing ported patterns; no
Cargo entry for any partner.

This is deliberate and load-bearing. It means:

- UAR must function standalone on-device.
- UAR may **not** delegate anything it requires to a partner unavailable on-device.
- Partner integration is **additive**, never a prerequisite.

**Corollary (added after adversarial review):** delegation of *authority* does not delegate
the *contract*. Where a partner owns a capability, UAR still owes the consumption contract
and a defined behavior when the partner is configured but unreachable — the routine case on
mobile. §5 records these.

### 1.2 Build profiles — capability varies by profile

**[V]** From `Cargo.toml`:

| Profile | Composition |
|---|---|
| `minimal` (default) | `server`, `surreal-backend` |
| `server-full` | `minimal` + a2a-transport, local-models, cedar-governance, response-quality, document-intelligence, telemetry, api-docs, admin-ui, wasm-runtime |
| `desktop-full` | `server-full` + tauri |
| `embedded-mobile` | `host-persistence` |

**[V] Routes are not individually feature-gated, but implementations are** —
`response-quality`, `telemetry`, `wasm-runtime`, and three storage backends sit behind
`#[cfg(feature)]` in `server.rs`. **The same route can exist with different behavior per
profile.** Any completeness claim must name the profile it was measured on.

---

## 2. Interface surfaces — there are three, not one

A route-only view of UAR is structurally incomplete: two of three target platforms never
touch HTTP.

### 2.1 HTTP API — **[V] 124 resolved paths**

Resolved by composing `server.rs` `.nest()` prefixes with each nested router's own routes.
Registration forms: `.route()` ×202, `.nest()` ×30, `.nest_service()` ×3, `.merge()` ×3,
`.route_service()` ×2, `.fallback_service()` ×1.

`/api/uar/*` (82 of 124):

| Sub-capability | Routes | | Sub-capability | Routes |
|---|---|---|---|---|
| skills | 13 | | actors | 4 |
| settings | 12 | | discovery | 3 |
| agents | 12 | | auth | 3 |
| compiler | 10 | | credentials | 2 |
| runs | 6 | | user, sync, route, resolve-model, mcp | 1 each |
| providers | 6 | | | |
| a2ui | 6 | | | |

### 2.2 Rust library API — a real contract surface

**[V] `docs/compatibility-policy.md:16` states the Rust embedding/library API is *not* a
public compatibility contract for 1.0.** KnowMe contradicts this in practice: it links UAR
as a path dependency and imports ~30 distinct UAR Rust paths, pinning four feature flags
(`embedded-mobile`, `in-memory-backend`, `surreal-backend` in the root crate; `http-client`
in `sdks/rust`).

**This spec records the conflict rather than resolving it — see OPEN-4 (§8).** For
completeness measurement, the Rust API is treated as a contract surface, because a consumer
depends on it in production.

### 2.3 Protocol contracts

**[V]** `src/uar/a2ui/protocol.rs`:
- A2UI profile `uar.a2ui/1` over A2UI `v0.9.1`, catalog `urn:uar:a2ui:catalog:1`
- Base catalog `https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json`

**[V]** AG-UI profile `uar.agui/1` (`server.rs`).

---

## 3. Capabilities UAR owns

Every entry is required by ≥1 consumer or implemented with a user-facing decision.

| # | Capability | Evidence | Consumers |
|---|---|---|---|
| C-01 | **Agent execution and run lifecycle** — create, stream, cancel, resume | `/api/uar/runs/*` | KnowMe (embedded + HTTP), BossFang |
| C-02 | **AG-UI event streaming** — SSE, POST-initiated, `last_event_id` resume | `runs/{id}/stream` | KnowMe, San Saba (contract) |
| C-03 | **Model routing and provider registry** | `/api/uar/route`, `/resolve-model`, `providers/*` | KnowMe, BossFang |
| C-04 | **Credentials** — multi-tenant provider credential subsystem | `credentials/*`, `security/credentials/` | KnowMe |
| C-05 | **Knowledge bases and RAG** | `/api/knowledge/*`, `rag/` (3,924 lines) | KnowMe |
| C-06 | **Memory** — scoped, typed, with history | `/api/admin/memories/*`, `memory/` (3,134) | KnowMe |
| C-07 | **Skills** — catalog, activation, native registry | `skills/*` (13 routes) | KnowMe |
| C-08 | **Tools and MCP** — registry, per-tool source attribution, approval | `/api/tools`, `mcp/*` | KnowMe |
| C-09 | **Agent compiler** — compile, register, A2A discovery | `compiler/*` (10), `/a2a/*` (6) | KnowMe, BossFang (schema) |
| C-10 | **Settings** — the deployment configuration surface | `settings/*` (12 routes; `settings.rs` has 64 raw) | all |
| C-11 | **A2UI** — schema registry, per-run surfaces, action round-trip | `a2ui/*` (6) | San Saba (contract), KnowMe |
| C-12 | **Persistence** — 3 backends: in-memory, SurrealDB, Postgres | `persistence/`, `design_systems/store.rs` | KnowMe (surrealkv on-device) |
| C-13 | **Sessions and threads** — caller-supplied thread IDs | `/api/sessions/*`, `/threads` | KnowMe, San Saba |
| C-14 | **OpenAI-compatible surface** | `/v1/chat/completions`, `/v1/models`, `/v1/messages` | **BossFang (only live traffic)** |
| C-15 | **Agent descriptor schema** (`uar-agent-descriptor/v1`) | `compiler/ir.rs`, `a2a/types.rs` | **BossFang — most durable requirement** |
| C-16 | **Governance** — tool approval, run policy | `governance/`, 4 settings refs | San Saba (hard requirement) |
| C-17 | **Security** — auth, API keys, credential encryption | `security/`, 5 settings refs | all |
| C-18 | **File processing / document intelligence** | `file_processing/`, 2 settings refs | KnowMe |
| C-19 | **Evals** | `eval/` (1,936 lines), 1 settings ref | none — see §6 |
| C-20 | **Health, readiness, metrics** | `/healthz`, `/readyz`, `/metrics`, `/ping` | BossFang, San Saba (probes) |
| C-21 | **Tenant isolation** — tenant-scoped runs, memory, KBs, tools, credentials | **[V] PARTIAL: exists for credentials only** (`runtime/manager.rs:144` "multi-tenant credential service"; `None ⇒ single-tenant"). **[V] No tenant awareness in `memory/`, `rag/`, or run state** | San Saba (blocking) |
| C-22 | **Scheduled / event-initiated runs** — cron, webhook, batch | **[V] ABSENT** — every run in C-01 is caller-initiated | San Saba (documented backlog) |

| C-23 | **Peer reachability** — a mobile/desktop instance is addressable from outside without binding a listening port, over an authenticated relationship between devices one user or org owns. **Default scope, on by default, disabled only by runtime configuration** | **[V] ABSENT** — no `iroh`, `libp2p`, `str0m`, `webrtc`, or `quinn` dependency. See §3.3 | operator intent |
| C-24 | **Peer mesh** — peer discovery, CRDT state convergence, and capability-aware routing of work to a peer, with no intermediate server and no token cost. **Default scope, on by default** | **[V] PARTIAL** — remote *invocation* exists (`api/a2a/client.rs:107`); discovery, transport, and peer-aware routing do not. See §3.3 | operator intent |

| C-25 | **Node decentralized identity** — every node has a self-sovereign DID derived from its own key, with no registry and no issuer. Optional but encouraged | **[V] BUILT** — `frf-did` derives `did:key` from an iroh endpoint key; verified against the W3C published test vector. `did:web` supported for nodes that control a domain | operator intent |
| C-26 | **DID resolution and credential verification** — resolve a peer's DID and verify what it presents, **offline** | **[V] BUILT** — `frf-did` (`did:key` offline, `did:web` over HTTPS) + `frf-wallet` signature verification | operator intent |
| C-27 | **Credential wallet and owner→node delegation** — an owner issues a capability credential to a node's DID; the node presents it when pairing | **[V] BUILT** — `frf-wallet`, W3C VC 2.0 data model, Ed25519-signed. 20 tests including forged-issuer, stolen-credential, capability-escalation, and expiry-extension cases | operator intent |

> **C-25/C-26/C-27 are the first capabilities in this document whose evidence is shipped,
> tested code rather than a survey finding.** They were built in the fabric on 2026-08-07 and
> are cited above at the level this spec's own ladder calls L1–L2 (present and wired). They are
> **not** L3: no UAR code consumes them yet, and no two devices have completed a real pairing.
> The distinction matters — see §3.4.

> **C-23/C-24 were added by operator direction, and that is itself a finding.** Neither was
> demanded by any of the three consumers, implemented in code, or recorded in any doc. Six
> independent surveys could not see them. See §10's revised limits: **operator intent is a
> fourth necessary source**, and triangulation across code + consumers + partners does not
> substitute for it.
>
> C-23 also corrects an error of mine. I had treated "no `server` feature ⇒ no routes ⇒
> unreachable" as settled. That conflates **binding a port** with **being addressable**.

> **C-21 and C-22 were added after adversarial review.** Both are first-class runtime
> capabilities that the route-and-module denominator missed: C-21 is a data-model invariant
> (invisible at the route layer), C-22 is an absent capability (nothing to enumerate). The
> earlier draft demoted C-21 to a gap footnote (GAP-03) covering only the A2A task store,
> which understated it — the same hole exists for memory, knowledge, and run state.
>
> **C-02 is downgraded to PARTIAL** against San Saba: the streaming capability exists, but
> the *contracted* event union is not satisfiable today (GAP-09).

### 3.1 Internal — no user-facing decision

**[V]** Zero routes *and* zero settings references: `guardrails`, `quality`, `prompt_cache`,
`orchestrator`, `telemetry`.

These are libraries, not capabilities. **They are not UI gaps.** (`telemetry` is a build
profile flag, not a user setting.)

> Method note: "has no route" was initially used as the internal test and was **wrong**.
> `security`, `governance`, `file_processing`, and `eval` have no dedicated routes but do
> carry user decisions via settings keys, so they are capabilities (C-16–C-19). The correct
> test is *"is there a user-facing decision"*, not *"is there a route."*

### 3.3 Decentralized peer operation — C-23 / C-24

**The intent.** UAR nodes owned by the same person or organization discover each other,
authenticate, sync state over CRDT, and **execute work on each other's behalf** — with no
intermediate server and no token cost. A phone asks for a model too large to run locally; a
Mac Studio at home running a quantized model serves the request over P2P. Target hardware
includes Apple silicon desktops and Linux AI boxes in homes and offices.

**This is default scope, not an optional add-on.** No Cargo feature gates it. The peer
subsystem is compiled into every profile — `minimal`, `server-full`, `desktop-full`,
`embedded-mobile` — and is **on by default**, disabled only through runtime configuration
(`PeerConfig { enabled: bool }`, defaulting to true). A build-time flag would mean the code is
absent from default binaries, never exercised by default tests, and adopters would have to know
to opt in at build time. Decentralized operation is the product direction, so it ships in the
default build.

*Note the deliberate divergence from house style:* the ~16 existing `enabled: bool` config
flags use bare `#[serde(default)]`, which yields **false**. C-23/C-24 use a named default
function yielding **true**. That inconsistency is intentional and recorded here so a future
reader does not "fix" it.

**Why C-23 exists at all — a correction.** An earlier reading treated "no `server` feature ⇒
no routes ⇒ unreachable" as settled. That conflates **binding a listening port** with **being
addressable**. A P2P transport gives the second without the first: the device dials out,
establishes an authenticated session, and serves requests over it. §12.1 established that
`embedded-mobile` has *zero* HTTP routes; C-23 is how that profile is reachable anyway.

**Open protocols only.** Operator constraint: no proprietary or custom protocol, and no direct
binding to fabric internals.

| Concern | Protocol |
|---|---|
| Transport (primary) | **iroh** — QUIC; node identity *is* the public key |
| Transport (later) | **WebRTC** data channels, for browser reach |
| State convergence | **Loro** CRDT |
| Identity | **JWT/JWKS** — the same open methods flint-gate already emits |
| Agent invocation | **A2A JSON-RPC** — already implemented, reused unchanged |

**Discovery is LAN + explicit pairing, not a DHT.** mDNS/local discovery plus manually-paired
remote nodes. "Gossip" in this spec means capability/state gossip **among already-paired
peers**, not peer *finding*. The spec deliberately does not claim DHT-scale decentralization —
a smaller, defensible requirement with the smallest attack surface.

**What already exists — reuse, do not rebuild.** The gap is narrower than "build a P2P system":

| Asset | Location |
|---|---|
| `A2AClient::send_message` / `get_task` — **remote invocation already works** | **[V]** `src/uar/api/a2a/client.rs:107,114`; constructed at `runtime/graph/nodes/agent_node.rs:105` |
| `federated_agent_registry` — a real federation seam | **[V]** `src/lib.rs:113`, wired `server.rs:649` |
| Agent cards, A2A discovery, task store | `src/uar/api/a2a/` |
| `local-models` — what a desktop node serves | `Cargo.toml:136` |

Missing: peer *discovery*, peer *transport*, and *routing that can choose a peer* (GAP-10/11/12).

**Trust boundary — non-negotiable, and default-on makes it stricter.** A device reachable from
outside has a different security posture than one that only dials out. With no build-time flag
standing between a default build and peer reachability, the auth prerequisite becomes
load-bearing rather than advisory:

- **Same JWT/JWKS verification as HTTP.** GAP-02's verifier is a **hard prerequisite**, not
  parallel work. P2P must not become a path *around* authentication.
- **Tenant/owner claims enforced per peer** (C-21). A peer mesh without tenant scoping is a
  cross-tenant execution vector.
- **Fail closed** on unverifiable peer identity, exactly as for an absent `tenant_id`.
- Peer relationships are **explicit and user-owned** — never open discovery.
- **Default-on ≠ auth-optional.** "On by default" means the subsystem is compiled, initialized,
  and configurable by default. Until the verifier exists, `PeerConfig::default()` must resolve
  to *enabled-but-unable-to-establish* — no verified identity, no session — enforced in code
  and covered by a test, since there is no compile-time gate to rely on.

### 3.4 Decentralized identity — C-25 / C-26 / C-27

**The load-bearing discovery: a node's DID is a pure function of the key it already has.**

**[V]** `iroh_base::EndpointId` is a type alias for `PublicKey`, a 32-byte Ed25519 key
exposed via `as_bytes()`. A `did:key` for Ed25519 is that *same* key under the `0xed01`
multicodec prefix, base58btc-encoded with a `z` multibase tag — the familiar `z6Mk…` form.

```
iroh SecretKey ──► EndpointId (32-byte Ed25519 public key)
                       │
                       ├──► QUIC handshake proves possession
                       └──► did:key:z6Mk…   (deterministic, offline, no network)
```

So decentralized identity here costs **no new key material, no registry, and no network
access**. That is what makes it viable on a machine in a house with no uplink, and why
"optional but encouraged" is nearly free rather than a parallel identity stack.

**A DID is a claim until it is checked.** The handshake proves the *key*, not any DID a peer
asserts. `frf_did::did_matches_endpoint` must be called before trusting an asserted DID; a
mismatch is an impersonation attempt, not a formatting problem. This is covered by test
`rejects_a_did_belonging_to_another_key`.

**Method choice (D-P4):**

| | `did:key` | `did:web` |
|---|---|---|
| Infrastructure | none | a domain + HTTPS |
| Works offline | **yes** | no |
| **Key rotation** | **never** | yes |

`did:key` is the default because it fits a personally-owned device. **Its inability to rotate
is a real limitation, recorded rather than buried:** if a node's key leaks, that identity is
permanently dead and must be re-paired everywhere. `did:web` is the mitigation for nodes with
a domain, at the cost of being only as decentralized as that domain. `did:peer` was considered
— it matches pairwise/LAN semantics well — but has thinner tooling and also cannot be updated
in place, so it does not solve rotation either.

**Owner→node delegation (D-P6).** The owner's DID issues a W3C VC 2.0 credential to the node's
DID granting enumerated capabilities. The node presents it when pairing. No central authority,
per-node revocation, and fully offline verification. Rejected alternative: a shared org key on
every node — no per-node revocation, and one compromised device compromises the estate.

`SignedCredential::verify_delegation` is the **only** entry point that returns a positive
authorization answer, and it checks six things in order: signature valid; signer *is* the
stated issuer; issuer is the owner the verifier expects; subject is *this* node; unexpired;
capability actually granted. Bundling them is deliberate — a valid signature alone authorizes
nothing, and a correctly-signed credential issued to a *different* device is a replay.

**SPIFFE was evaluated and rejected on evidence, not preference.** It is the incumbent for
workload identity, but requires dedicated infrastructure — attestation nodes, registration
servers, certificate authorities. That disqualifies it for a standalone machine in a home,
which is the target topology.

**Evidence level — stated precisely.** C-25/26/27 are **L1–L2** (present and wired), not L3.
The crates compile, 37 tests pass across `frf-did` and `frf-wallet`, clippy is clean under
`pedantic` with `unwrap_used`/`expect_used` denied, and the `did:key` derivation is verified
against the **W3C specification's own published test vector** rather than this
implementation's output. But **no UAR code consumes them yet and no two devices have paired**.
Per this document's own ladder, that is not done.

---

## 4. UI exposure requirements

The web application exists so developers can configure a runtime for deployment and validate
agents without writing their own harness. Each capability is classified by what the UI owes:

| Class | Obligation |
|---|---|
| **INTERACTIVE** | user must be able to exercise it |
| **DISCOVERABLE** | UI must tell the user it exists and how to reach it, without calling it |
| **OPERATIONAL** | for orchestrators and probes; no UI expected |

**[V] DISCOVERABLE is an observed pattern, not an invention**: `/v1/chat/completions` is
listed in `runtime-console-page.tsx:429` as "OpenAI-compatible chat and model catalog
surface" — displayed, never called. A developer must know UAR speaks OpenAI-compat to point
their own SDK at it. That is squarely "help developers determine HOW to do their work."

| Capability | Class |
|---|---|
| C-01–C-13, C-16–C-19 | INTERACTIVE |
| C-14 OpenAI-compat, C-15 descriptor schema, `/a2a/*`, `/mcp/uar` | DISCOVERABLE |
| C-20 health/metrics, `/.well-known/security.txt` | OPERATIONAL |

**Guard against misuse (tightened after adversarial review).** The first version of this
guard said "DISCOVERABLE only if its primary consumer is a machine." The judge showed that
is unenforceable, because it records **who calls** the capability and ignores **who
configures it** — `/a2a/*` is machine-consumed, but *deciding which agents are exposed* is a
human act that would carry no UI obligation.

The enforceable form is **two independent questions per capability**:

1. **Who invokes it?** machine → not INTERACTIVE for invocation.
2. **Does any human decision govern its behavior?** — enablement, exposure, credentials,
   policy, limits. **If yes, that decision is INTERACTIVE regardless of who invokes it.**

A capability is DISCOVERABLE **only when both** are machine-side. `/a2a/*` therefore splits:
discovery endpoints are DISCOVERABLE; *which agents are published* is INTERACTIVE.

---

## 5. Partner boundaries — what UAR does NOT own

Each delegation names the owner **and** UAR's remaining consumption contract. A "not UAR"
verdict without a counterpart is how phantom work becomes dropped work.

| Capability | Owner | UAR still owes |
|---|---|---|
| RS256/JWKS token **issuance** | **flint-gate** [V] mints RS256, `kid: sansaba-ssr-rs256-v1`, publishes `/.well-known/jwks.json` | **A JWKS verifier** — fetch, `kid` selection, cache, rotation, `iss`/`aud` pinning, **and fail-closed when the JWKS is unreachable OR returns an empty key set** (see OPEN-3: that may be the production reality). **[V] Absent today**: `jsonwebtoken 10.2.0` present, zero JWKS code |
| Tenant claim **origin** | **flint-gate** [V] mints `tenant_id` from Kratos `metadata_public.tenant_id`; forwards `X-Flint-Tenant-Id` | **Extract from verified claims and FAIL CLOSED when absent** — Gate renders it empty if Kratos lacks it. **[V] Zero `x-flint-*` consumption today** |
| **Presence** | flint-realtime-fabric (`frf-domain/src/presence.rs`) | nothing — do not build |
| **CRDT cross-device sync** | flint-realtime-fabric (Loro + redb) | nothing — do not build |
| **Multi-client fan-out** | flint-realtime-fabric (`publisher.rs`) | **Own run history + `last_event_id` resume regardless.** The fabric's agent bus uses `try_send` and *silently drops events on a full channel* — it is not a durable substitute |
| **A2UI authoring / design-time** | flint-forge | **The accept/store/serve side** — an ingestion interface and catalog format, plus catalog seeding (GAP-06) and an owner for `surfaces/assemble` (GAP-08). *"Forge owns authoring" does not create the pipe.* |
| **RLS-backed data APIs** *(Forge's own stores)* | flint-forge | **Tenant isolation of UAR's OWN stores is UAR's and undelegatable** — runs, memory, knowledge bases, A2A tasks, credentials. See C-21 and GAP-03. Forge's RLS covers Forge's data, not UAR's |
| **P2P transport** (iroh/QUIC session establishment, WebRTC data channels) — C-23 | **flint-realtime-fabric**, which must gain an **iroh transport and a Rust SDK client** it does not have today | **Peer-aware routing, remote execution, and identity verification stay UAR's.** UAR consumes the SDK over open protocols (iroh, WebRTC, Loro, JWT/JWKS, A2A JSON-RPC) — **no direct binding to fabric internals and no custom protocol** |

> **The P2P delegation is conditional, unlike the rows above it.** Every other partner
> capability is delegated to something that exists. This one is delegated to work not yet
> done: **[V] the fabric's own `IMPLEMENTATION-PLAN.md:200` marks P2P CRDT sync `live: no`**,
> its str0m crate is media-only (no data channels), and `frf-crdt` exposes only
> `export_updates_since` / `merge_into_store` / `apply_delta` — a delta-sync primitive, not
> an RPC transport. Recorded here so the dependency is visible rather than assumed.
>
> **Status update 2026-08-07 — the transport dependency is now partially discharged.** The
> fabric gained three crates: `frf-p2p` (iroh 1.0.3 transport, pairing, fail-closed session
> establishment), `frf-did` (C-25/C-26), and `frf-wallet` (C-27). **[V]** All compile, 47
> tests pass across the three, clippy clean. What remains for C-23 is the *accept* loop and
> mDNS wiring; what remains for C-24 is peer-aware routing in UAR (GAP-12).
>
> Note the earlier claim in this spec that the fabric needs "a Rust SDK client it does not
> have today" was **wrong**: `frf-sdk-rust` already exists (651 lines, tonic/gRPC with auth
> interception and a reconnect policy). The accurate statement is narrower — the existing SDK
> requires a *reachable gateway URL*, which is precisely what a peer path must not need. That
> is a gap in transport shape, not a missing SDK.

> **This dependency is now on UAR's critical path, not beside it.** Because C-23/C-24 are
> default scope and on by default (§3.3), the fabric's iroh transport and Rust SDK client are
> **release-blocking for every profile**, not optional work behind a build flag. That is a
> material change to the fabric's priority, and it is the direct consequence of the default-on
> decision rather than an independent judgment.

---

## 6. Capabilities with no consumer demand

Implemented, no surveyed consumer requires them. **Recorded, not condemned** — absence of
demand from three consumers is not proof of uselessness.

| Capability | Note |
|---|---|
| C-19 Evals | UAR ships `evals/*.yaml`; zero consumer references |
| gRPC transport (`:50051`) | exposed in San Saba's deployment; no client found |
| Run checkpoints / resume | SDK exposes `/runs/{id}/checkpoints`, `/resume`; no consumer call site |
| Direct tool execution | `/api/tools/{name}/execute`; no consumer call site |
| Knowledge document upload | SDK exposes it; KnowMe manages KBs but never uploads |

---

## 7. Known gaps

| # | Gap | Evidence | Impact |
|---|---|---|---|
| **GAP-01** | `GET /.well-known/uar-runtime` **absent** | **[V]** UAR has `/.well-known/uar-config`; of BossFang's three probed tokens only `a2ui.registry` exists (`openai.chat.completions`, `ag-ui.stream.agui_spec` absent) | **BossFang pods cannot reach Ready.** Hard production dependency. May be a rename of `uar-config` — verify before scoping as new work |
| **GAP-02** | No JWKS/RS256 verifier | **[V]** `src/config.rs:284` shared `jwt_secret`; no JWKS code | Blocks San Saba adoption |
| **GAP-03** | A2A task store not tenant-partitioned | **[V]** `a2a/task_store.rs:16` — flat `RwLock<HashMap<String, Task>>` | Blocks San Saba; security property; **undelegatable** |
| **GAP-04** | Rust API used as a contract while policy says it is not | **[V]** `compatibility-policy.md:16` vs KnowMe's ~30 imports | Every internal refactor risks breaking KnowMe |
| **GAP-05** | `register_builtins` called from two server paths but not the embedded runtime | **[V]** `src/server.rs:454`, `src/server.rs:517`; `src/embedded.rs` | Built-ins are absent when the embedded runtime starts against a fresh database |
| **GAP-06** | A2UI catalog is 5 hardcoded builtins | **[V]** `builtin_{form,confirm,select,text_input,display}` vs the 55-definition a2ui.org catalog | Consumers expect a real catalog |
| **GAP-07** | `@prometheus-ags/a2ui-*` appear unpublished | **[V]** `a2ui-core@0.10.4`, `a2ui-react@0.10.1`, `a2ui-uar@0.1.0` all have `"main": "./src/index.ts"` — raw TS, no build output | San Saba hand-authored a mirror believing these did not exist. **Packaging problem, not a missing implementation** |
| **GAP-08** | `/a2ui/v1/surfaces/assemble` orphaned | Deferred Change 19 → Change 20; done in neither | Consumers depend on Forge for it |
| **GAP-09a** | **AG-UI vocabulary mismatch** (naming) | **[V]** San Saba's closed union requires `VECTOR_CITATION`, `MEMORY_HIT`, `SKILL_SELECTED`; UAR emits the same concepts as `uar.*` CUSTOM (`uar.citation.added`, `uar.memory.recall`, `uar.skill.activated`) | Genuinely a naming mismatch — the information is present. **But** San Saba's union is a compile-time exhaustive switch, so an unmapped event is a **build failure**, and any standard AG-UI client dispatching on the typed union sees nothing. Protocol-conformance issue |
| **GAP-09b** | **AG-UI missing capability** | **[V]** `REASONING_ENCRYPTED_VALUE` (opaque/encrypted reasoning passthrough) and `ACTIVITY_SNAPSHOT`/`ACTIVITY_DELTA` are absent everywhere — no `uar.*` equivalent | A real capability gap, not a naming one |
| **GAP-10** | **No P2P transport in UAR** | **[V]** No `iroh`, `libp2p`, `str0m`, `webrtc`, or `quinn` dependency in `Cargo.toml` | Blocks C-23. A device that cannot bind a port cannot be reached at all |
| **GAP-11** | **The fabric's P2P path is not live** | **[V]** fabric `IMPLEMENTATION-PLAN.md:200` marks CRDT-sync P2P `live: no`; its str0m is media-only; `frf-crdt` exposes 3 delta-sync functions and no transport | Blocks C-23. The delegated owner has not built the thing being delegated (see §5) |
| **GAP-12** | **The router cannot express "a peer"** | **[V]** `src/uar/llm/router.rs:4-8` — `RouteTarget` is `{ Fast, Smart, Reasoning }`, model *tiers* only, with no notion of *where* a model runs; `route()` dispatches on prompt heuristics (`needs_reasoning`) | Blocks C-24, and it is **the highest-leverage single change**: without a peer variant carrying node identity and capability, the "phone asks the Mac Studio" scenario cannot be expressed at all |

> **GAP-10/11/12 are smaller than "build a P2P system," and the spec should not overstate
> them.** Remote *invocation* already works: **[V]** `A2AClient::send_message`/`get_task`
> exist (`src/uar/api/a2a/client.rs:107,114`) and are constructed at
> `runtime/graph/nodes/agent_node.rs:105`; `federated_agent_registry`
> (`src/lib.rs:113`, wired `server.rs:649`) is a real federation seam. What is missing is
> peer *discovery*, peer *transport*, and *routing that can choose a peer*.
>
> **Sequencing constraint — REVISED 2026-08-07 after C-25/26/27 shipped.** An earlier
> revision said GAP-02's JWKS verifier was a hard prerequisite for peer reachability. That was
> written when flint-gate was assumed to be the only issuer. With DID/VC the peer path does
> **not** depend on flint-gate at all:
>
> | Path | Verifier | Requires network? |
> |---|---|---|
> | **Peer ↔ peer (default)** | DID resolution + VC verification (`frf-did`, `frf-wallet`) | **No** — `did:key` is offline; the iroh handshake proves key possession |
> | HTTP API | JWKS verifier (GAP-02) | Yes — flint-gate |
>
> These are now **parallel tracks, not a chain.** GAP-02 remains a real gap for the HTTP
> surface and still blocks San Saba, but it is no longer on C-23's critical path. This is a
> strict improvement: a home node with no uplink can still authenticate a peer.
>
> **What does NOT change:** because C-23/C-24 are default scope and on by default (§3.3),
> there is no build-time flag between a default build and peer reachability, so fail-closed is
> the whole guarantee. **[V] Enforced and tested**: `frf-p2p`'s default verifier is
> `DenyAllVerifier`, which authenticates nobody
> (`deny_all_verifier_refuses_every_token`), and a default-configured transport establishes no
> session (`default_transport_cannot_establish_a_session`). Default-on ≠ auth-optional.

> **GAP-09 was split after adversarial review.** The judge argued the whole mismatch was
> being excused as "vocabulary." That was right for `REASONING_ENCRYPTED_VALUE` and
> `ACTIVITY_*` — now GAP-09b.
>
> **One judge claim was wrong and is rejected:** that `TEXT_MESSAGE_CHUNK`/`TOOL_CALL_CHUNK`
> prove UAR lacks token-level streaming. **[V] `adapters.rs:92-97`** maps
> `NormalizedEvent::ChatDelta { text_delta }` into `TEXT_MESSAGE_CONTENT` with a `delta`
> field. UAR *does* stream incremental deltas; it carries them on a differently-named event.
> That belongs in GAP-09a, not 09b.

---

## 8. OPEN — blocking full spec freeze

Adversarial review required these be marked open rather than resolved. Each changes the
denominator's boundaries.

### OPEN-1 · Forge operates a competing AG-UI run stream
flint-forge implements `/agents/v1/:run_id/surfaces/assemble` and per-run SSE emitting
`a2ui:surface`. **Agent runs and per-run event streams are UAR's core identity.** Neither
repo's docs acknowledge the overlap. **Product decision required** — not a technical finding.

### OPEN-2 · Is `frf-gateway` UAR, or does it front UAR?
flint-gate's production overlay mints tokens with `aud: ["frf-gateway"]` for site
`sansaba-runtime`. If `frf-gateway` *is* UAR, GAP-02's verifier must accept that audience.
If it fronts UAR, the trust boundary is different. Not determinable from the surveyed repos.

### OPEN-3 · Empty JWKS with a file-based key — **PARTIALLY RESOLVED**
flint-gate's SSR overlay sets `database.url: ""` with a file PEM, while `jwks_handler`
returns an empty key set when no DB is configured. On its face that deployment **mints RS256
tokens but publishes an empty JWKS**.

**Reclassified after adversarial review.** The judge was right that this is largely a
determination, not an open question — the evidence is already in hand. Two consequences,
both now actionable:

- **A likely flint-gate defect**, to raise with that project.
- **A firm UAR requirement, no longer contingent:** the GAP-02 verifier must **fail closed
  on an empty or unreachable key set**. That is now in §5's contract rather than waiting on
  this item.

**Genuinely still open:** whether another deployment overlay differs. That is a deployment
question, not spec-blocking.

### OPEN-4 · Is the Rust library API a contract? (GAP-04)
Either `compatibility-policy.md` is wrong, or KnowMe is knowingly outside the policy. This
spec cannot decide it; it affects every future refactor.

---

## 9. Not in scope

Recorded so a future reader can distinguish *absence* from *omission*:

- Identity provisioning, token issuance, user management → flint-gate
- Presence, multi-client fan-out, WebRTC **media** → flint-realtime-fabric
- **P2P transport mechanics** (iroh/QUIC session establishment, WebRTC data channels) →
  flint-realtime-fabric, *as a dependency UAR consumes over open protocols* — see §5 and C-23
- A2UI design-time authoring, RLS-backed data APIs → flint-forge
- Prometheus skill *package* content → prometheus-skill-system
- Application-level workflow orchestration → the consuming application

> **Decentralized peer operation is IN scope, is DEFAULT scope, and must not be read out of
> it.** An earlier revision of this list said "CRDT sync … → flint-realtime-fabric" without
> qualification, which would have excluded C-23/C-24 entirely. The correct split: the fabric
> owns the **transport**; UAR owns **peer-aware routing, remote execution, CRDT state
> ownership, and identity verification**. Delegating a transport is not delegating the
> capability — and per §3.3 it is not gated behind a build flag either.

---

## 10. How this was derived, and what it cannot prove

**Sources:** UAR source; consumer docs+code for KnowMe (43 docs), BossFang (42), San Saba
(35); partner surveys of flint-gate, flint-realtime-fabric, flint-forge. Six parallel
agent surveys, each reporting doc-vs-code corroboration and recency.

**Adversarially reviewed.** An isolated critic (MiniMax-M3) and judge (k3) attacked the
denominator before this document was written. They found:

- **Counting mounted routes as delivered capability** — the central flaw. GAP-05 is the
  live proof: 124 correct routes, one capability at 0%.
- **A2UI authoring exclusion was wrong** — corrected in §5.
- **"No push contract exists" was an overcorrection** from grep evidence — the honest claim
  is *"no push contract found in code."*
- **The portability test is monotonic** — it only ever produces "not UAR" verdicts. §1.1's
  corollary is the mitigation.

### What this spec CANNOT prove

1. **That a capability nobody built and no consumer demanded is missing.** Triangulation
   narrows this; it does not close it — **and C-23/C-24 are the proof that the hole is real,
   not theoretical.**

   Decentralized peer operation is a long-held product intent. It was **invisible to all six
   surveys**: no consumer demands it, no code implements it, no doc records it. Three sources
   agreeing on its absence produced no signal, because all three were silent for the same
   reason — it had never been built. It entered this spec only because the operator stated it.

   **Therefore operator intent is a fourth necessary source**, not a nice-to-have. Any earlier
   claim in this document that code + consumers + partners is a sufficient basis is hereby
   weakened: those three establish what *exists* and what is *currently demanded*, never what
   is *intended but unbuilt*. A capability can be central to the product's direction and score
   zero on every triangulated axis. **The method's blind spot is precisely the most
   forward-looking requirements**, which is the worst possible place for it.
2. **That any capability works.** This is a specification, not a verification. Route
   presence, module existence, and consumer demand are all recorded — behavior is not.
   **Verification must be per-profile** (`minimal`, `server-full`, `desktop-full`,
   `embedded-mobile`), because implementations are feature-gated even where routes are not.
3. **That consumer requirements are current.** Weighted by code corroboration and recency,
   but a stale consumer doc can still mislead.

### Revision rule

When this spec and another `docs/` file disagree, **this file wins** and the other is a
candidate for `docs/archived/`. When this spec and the *code* disagree, **the code wins** and
this file is wrong — file a correction.

---

## 11. Archiving authority — WITHHELD pending §12

An isolated judge was asked whether this document should be the basis for archiving ~140
files. The answer was **NO**, for a reason worth recording verbatim in effect:

> This is a **census of surfaces, not a verified contract**. §10 admits it cannot prove
> anything works, and GAP-05 proves route-presence evidence is actively misleading.
> Archiving against it will archive real requirements encoded nowhere else.

That reasoning is accepted. **This spec is authoritative for what UAR is expected to do. It
does not yet carry authority to archive a document**, because a doc describing a capability
that silently fails on one profile would be archived as "contradicted" when it is in fact
the only surviving record of a real requirement.

Archiving authority is granted only after §12 is populated.

## 12. Per-profile conformance — REQUIRED, NOT YET RUN

For each capability C-01…C-27, an executable check under each build profile:

| Profile | Status |
|---|---|
| `minimal` (default) | not run |
| `server-full` | not run |
| `desktop-full` | not run |
| **`embedded-mobile`** | **RUN 2026-08-07 — compiles; 1 capability FAIL; see §12.1. Predates C-23/C-24 — incomplete, must be re-run** |

> **Every profile must now also check C-23/C-24, including the one already certified.**
> Because P2P is default scope and on by default (§3.3), a profile that compiles without the
> peer subsystem is **non-conformant** — not merely reduced. This adds two checks to every
> profile:
>
> - **C-P6 · Peer subsystem compiled and initialized** on this profile, with no `cfg` gate.
> - **C-P7 · `PeerConfig::default()` is enabled, and a peer session is refused when identity
>   cannot be verified.** Default-on ≠ auth-optional; this must be asserted, not assumed.
> - **C-P8 · Node DID derivable on this profile** (C-25). `did:key` derivation is offline and
>   allocation-light, so it should hold on `embedded-mobile` — but "should" is exactly the kind
>   of claim §12 exists to stop this document from making.
>
> **[V] C-P7 is already satisfied in the fabric**, though not yet in UAR: `frf-p2p`'s default
> verifier is `DenyAllVerifier` and `default_transport_cannot_establish_a_session` asserts the
> refusal. That is fabric-side evidence; the UAR-side check remains outstanding because UAR
> does not consume these crates yet.
>
> `embedded-mobile`'s §12.1 result therefore **carries forward but is no longer complete**.
> Its GAP-05 finding stands; its scope predates C-23…C-27 and did not test for them.
> **This is the second time a §12 verdict has been invalidated by a scope change** — the
> first was the discovery that routes do not exist on this profile at all. Per-profile
> conformance is not a one-time gate; it re-opens whenever the denominator changes, and
> §11's archiving authority stays withheld until the re-run completes.

### 12.1 `embedded-mobile` — first profile checked

`cargo check --locked --no-default-features --features embedded-mobile` → **PASS**, clean,
2 warnings.

**The structural finding matters more than any individual check. [V]**
`embedded-mobile = ["host-persistence"]` and nothing else. It does **not** enable `server`,
and `src/lib.rs:40` gates `pub mod server` behind `#[cfg(feature = "server")]`.

> **The entire 124-route HTTP denominator does not exist on this profile.**

On the platform KnowMe ships to mobile, every capability reaches the consumer through the
**Rust library API** — the surface `compatibility-policy.md:16` declares is *not* a
contract. GAP-04 is therefore not a policy nicety; on mobile it is the whole contract.

| Check | Result |
|---|---|
| Compiles | **PASS** |
| **Builtin skills registered (GAP-05)** | **FAIL — confirmed** |
| SSRF guard intact | **PASS**, with a correction below |
| Persistence available | **PASS** |
| Host remediation path exists | **PASS** |

**GAP-05 mechanism, now precise:** `register_builtins` is called only at `server.rs:436`.
Because `server` is not compiled here, **that call site is absent from the binary** — a
compile-time exclusion, not a runtime ordering bug. An embedded host that does nothing extra
gets an **empty skill registry**: capability C-07 at 0% on this profile while all 124 routes
stay "present and correct" on others.

Severity is tempered by the remedy: `SkillService::register_builtins` is `pub`
(`skills/service.rs:167`) and the SDK exposes `Runtime::native_skills()`
(`sdks/rust/runtime.rs:128`) — KnowMe uses exactly this. **So GAP-05 restates as: builtins
are not registered by default on embedded profiles, the host must do it explicitly, and this
is undocumented.** Not "skills are broken on mobile."

**A correction to my own first reading.** The compile surfaced dead-code warnings for
`MAX_BODY_BYTES` and `MAX_REDIRECTS` in `fetch_guard.rs`, and my initial inference was that
SSRF protection might be inert here. **Wrong — checked before recording.** `web_fetch.rs`
enforces both independently: `redirect::Policy::none()` at :193 (commented as stopping
`169.254.169.254` redirects) and a `max_size_kb` cap at :239. The constants are vestigial
with no consumer on any profile, and `FetchDenial::TooManyRedirects` is never constructed —
worth deleting so the warning stops implying a gap.

**Not yet exercised on this profile:** memory (C-06), knowledge/RAG (C-05), model routing
(C-03), A2UI (C-11), tenant isolation (C-21). A compile check plus call-site tracing is not
a behavioral test; that needs an embedded harness that constructs a `Runtime` and drives it.

### 12.2 What this profile check proved about the method

1. **Per-profile measurement is a correctness requirement, not a refinement** — the
   denominator's unit (HTTP routes) does not exist on one of four target profiles.
2. **"Default-off with an undocumented opt-in" is a failure class** that neither route
   enumeration nor module inventory can detect. It surfaced only by tracing a call site
   against a feature gate.

**Why per-profile is non-negotiable: [V]** routes are not individually feature-gated, but
implementations are (`response-quality`, `telemetry`, `wasm-runtime`, and three storage
backends). GAP-05 is the standing proof — `register_builtins` is called only from
`server.rs`, so `embedded-mobile` boots with an empty skill registry while all 124 routes
remain present and correct.

A check must assert something **positive and observed** (a real response, a rendered
result), never the absence of an error. Non-route channels get the same treatment: the Rust
library API, SSE stream semantics (resume, replay, idempotency), published packages, and
`.well-known` documents.
