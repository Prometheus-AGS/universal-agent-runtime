# Vendored unpublished Rust sources

Only unpublished direct dependencies are stored here. Published dependencies
use exact crates.io versions in `Cargo.toml`. Vendored trees are pinned source
exports with VCS metadata, tests, documentation, and agent state removed.

| Package | Upstream | Commit | License |
|---|---|---|---|
| `surreal-memory` | `https://github.com/Prometheus-AGS/surreal-memory-server.git` | `9e36b1cd8b83a657366eb1e3ef8cd837178336de` | MIT (declared in crate manifest) |
| `prometheus_parking_lot` | `https://github.com/Prometheus-AGS/prometheus-parking-lot-rs` | `4b747350c6883497813bed33623a9937ad7a5894` | MIT (`prometheus-parking-lot-rs/LICENSE-MIT`) |
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

The Parking Lot export for issue #426 records upstream source snapshot
`4b747350c6883497813bed33623a9937ad7a5894` (manifest version `0.3.0`),
replacing `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0`. This is a pinned
source snapshot, not a claim that the version is published. The facade, seven
workspace crates, target-specific sources, PostgreSQL migrations, required
README files and shipped licenses are preserved. Package metadata, dependency requirements and features, and lints are materialized
into standalone manifests, including the upstream Rust 1.95 minimum.
UAR depends directly on the runtime-free core; its existing host owns worker
execution. The facade, Tokio runtime, optional PostgreSQL, embedded and Iggy
backends and the standalone FFI member are retained as upstream source.

Implementation source is copied byte-for-byte. The facade and seven nested
crate manifests are standalone: inherited package fields, dependencies and lints
are copied from the pinned upstream workspace into each consuming manifest.
Sibling path dependencies are relative to each crate; dependency requirements,
features, optional flags, target conditions, and lint values retain upstream
semantics. The vendored workspace declaration and explicit nested workspace-owner
pointers are removed. UAR excludes the facade and all seven nested packages from
its own workspace. This resolves the v1.3.0 deployment manifest failures: implicit
inheritance first selected UAR's workspace, while adding explicit nested owners
then produced multiple workspace roots. The facade drops its four explicit
benchmark targets and eight explicit example targets, whose development-only
files are not exported.
Tests, benches, examples, VCS metadata, workflow tooling, and agent state are
omitted; upstream implementation files containing inline tests are preserved
without rewriting. Refresh from the exact commit through GitHub git blobs,
export the same production paths, and reapply these documented manifest
adaptations. No upstream modification or verification is implied.
