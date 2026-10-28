# Vendored pinned Rust sources

Unpublished direct dependencies and explicitly retained production source exports
are stored here. Parking Lot retains its standalone export at the published
release commit; changing its delivery mechanism is separate from this version
refresh. Vendored trees omit VCS metadata, tests, tooling and agent state.

| Package | Upstream | Commit | License |
|---|---|---|---|
| `surreal-memory` | `https://github.com/Prometheus-AGS/surreal-memory-server.git` | `9e36b1cd8b83a657366eb1e3ef8cd837178336de` | MIT (declared in crate manifest) |
| `prometheus-parking-lot-core` (with facade and sibling source) | `https://github.com/Prometheus-AGS/prometheus-parking-lot-rs` | `53682521cb2a7a49c06e046c826890d9e11b6bfe` | MIT (`prometheus-parking-lot-rs/LICENSE-MIT`) |
| `sycophancy-core` | `https://github.com/Know-Me-Tools/sycophancy-correction-skill.git` | `01150389c10169816fbd4cc4ef4145fbe052ad90` | MIT (declared in workspace manifest) |

The two upstream snapshots that declare MIT without shipping a license file are
covered by the canonical MIT text in `LICENSE-MIT`; the provenance table
preserves that this record was added during vendoring.

The `surreal-memory` snapshot intentionally preserves UAR's standalone manifest
adaptations while copying the four changed implementation files byte-for-byte
from the recorded upstream commit. Its manifest exact-pins `surrealdb`,
`surrealdb-types`, and `hf-hub` while retaining the empty, disabled `palace`
compatibility feature because UAR does not ship the upstream workspace's
`mempalace-core` dependency.

The lifecycle integration advances the memory export by the upstream storage changes at `9e36b1cd8b83a657366eb1e3ef8cd837178336de`, through its parent 3a767608d53481ebc3c9085b8f5b4b82316ad8cb from the prior pin. It introduces opt-in host-managed connection ownership. Existing standalone manifest adaptations are preserved; upstream source is copied byte-for-byte. No upstream push or verification is implied.

The in-tree Liter checkout additionally carries a local UAR serialization
accessor in `crates/liter-llm/src/client/mod.rs`:
`DefaultClient::chat_stream_request_body` delegates to the same request
preparation as managed streaming. UAR uses its actual serialized bytes for
budget receipts instead of reconstructing an approximate wire representation
or bypassing managed transport through the raw client. The base dependency pin
does not identify these local bytes; the whole-phase source inventory must
include them. This is an unpublished local patch, with compilation and runtime
verification pending under the closed operator gate.

The Parking Lot export for issue #426 records the official published `v0.4.0`
tag target `53682521cb2a7a49c06e046c826890d9e11b6bfe`, annotated tag object
`943c62d860e8971847334e9196ad34881658fbcb`. The release was published on
2026-10-25 at15:18:35Z. The earlier export used the untagged same-version
`f24382acd7ba818204e32f4449e9c9c37c52558d` snapshot, which had replaced0.3.0
`4b747350c6883497813bed33623a9937ad7a5894`.

The official GitHub comparison from that earlier snapshot to the release target
contains14 commits changing only upstream release/assurance scripts, workflows
and an OpenSpec plan. No facade/core/sibling implementation, manifest, lockfile
or license path exported here changes. The existing production export bytes
therefore already match the released source; this refresh changes its provenance
pin, not UAR's dependency graph or the standalone manifest adaptations. The
GitHub API reports the annotated tag signature valid; this is upstream metadata,
not UAR integration acceptance. Sources:
<https://github.com/Prometheus-AGS/prometheus-parking-lot-rs/releases/tag/v0.4.0>,
<https://github.com/Prometheus-AGS/prometheus-parking-lot-rs/compare/f24382acd7ba818204e32f4449e9c9c37c52558d...53682521cb2a7a49c06e046c826890d9e11b6bfe>.

The facade, seven sibling crates, target-specific sources, PostgreSQL migrations,
required README files and shipped licenses are preserved. Implementation source
is copied byte-for-byte from the exact commit archive; the only adaptations are
manifests. Package metadata, dependency requirements, features and lints are
materialized into standalone manifests, including upstream's Rust 1.95 minimum.
Sibling path dependencies are relative to each consuming crate. Dependency
requirements, optional flags, target conditions and lint values retain upstream
semantics. The facade omits explicit development-only benchmark/example targets
whose files are not exported. The upstream workspace declaration and nested
workspace-owner pointers remain removed; UAR excludes the facade and all seven
nested packages from its workspace. Reintroducing inherited workspace ownership
would repeat the deployment's prior multiple-workspace-root failure.

UAR consumes only `prometheus-parking-lot-core` with its default empty feature
set. Its dependency requirements are unchanged from the prior core snapshot;
the UAR lockfile changes only that local package's version to `0.4.0`. Optional
facade backends, the Tokio runtime and the standalone FFI source are retained but
are not dependencies of UAR. No Cargo dependency-resolution command was needed.
The 0.4.0 dispatch batch adds `rejected` descriptors; UAR's ingestion owner must
settle these outside the dispatch lock. UAR does not use the upstream durable
mailboxes, so the snapshot's PostgreSQL protocol-3/redb schema-3 migration is not
a UAR ingestion-store migration.

Tests, benches, examples, VCS metadata, workflows, tooling and agent state are
omitted. Implementation files containing inline tests are preserved unmodified.
Refresh from the exact official commit archive or git blobs, export the same
production paths, and reapply these standalone manifest adaptations. No upstream
modification, publication or verification is implied.
