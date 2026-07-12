## 1. Inputs
- [x] 1.1 Inventory all build/runtime network and Git source dependencies.
- [x] 1.2 Publish or vendor release-grade Git dependencies with license/source records.
- [x] 1.3 Commit versioned provider/model snapshots with source URL/date/digest.
## 2. Build behavior
- [x] 2.1 Make `build.rs` consume only committed inputs for release bundles.
- [x] 2.2 Add explicit catalog/model refresh maintainer commands and diff review.
- [x] 2.3 Add clean-source `cargo build --locked --offline` CI.
## 3. Verify
- [x] 3.1 Rebuild twice in isolated environments and compare declared reproducible outputs.
- [x] 3.2 Verify source archive contains every required input and license.
- [x] 3.3 Document procedure and validate OpenSpec.
