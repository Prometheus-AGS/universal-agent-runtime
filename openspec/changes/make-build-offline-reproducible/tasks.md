## 1. Inputs
- [ ] 1.1 Inventory all build/runtime network and Git source dependencies.
- [ ] 1.2 Publish or vendor release-grade Git dependencies with license/source records.
- [ ] 1.3 Commit versioned provider/model snapshots with source URL/date/digest.
## 2. Build behavior
- [ ] 2.1 Make `build.rs` consume only committed inputs for release bundles.
- [ ] 2.2 Add explicit catalog/model refresh maintainer commands and diff review.
- [ ] 2.3 Add clean-source `cargo build --locked --offline` CI.
## 3. Verify
- [ ] 3.1 Rebuild twice in isolated environments and compare declared reproducible outputs.
- [ ] 3.2 Verify source archive contains every required input and license.
- [ ] 3.3 Document procedure and validate OpenSpec.
