# Vendored unpublished Rust sources

Only unpublished direct dependencies are stored here. Published dependencies
use exact crates.io versions in `Cargo.toml`. Vendored trees are pinned source
exports with VCS metadata, tests, documentation, and agent state removed.

| Package | Upstream | Commit | License |
|---|---|---|---|
| `surreal-memory` | `https://github.com/Prometheus-AGS/surreal-memory-server.git` | `432eaa1ebbef66fc02b9bb1a1e63cc2fdb2149e8` | MIT (declared in crate manifest) |
| `prometheus_parking_lot` | `https://github.com/Prometheus-AGS/prometheus-parking-lot-rs` | `ebb7c3ce02f7b925bc2e1b45c87ce8abf402b1f0` | MIT (`prometheus-parking-lot-rs/LICENSE-MIT`) |
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
