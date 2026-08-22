# Goals — perform-the-soak-run-candidate-tag-external-installs-and-ga-promotion > fix-container-rust-toolchain-pin-consistency

- Make the production Docker backend build consume the dated RUST_TOOLCHAIN pin instead of floating nightly
- Prove the pinned ARM64 toolchain passes locked diskann-wide while floating nightly reproduces the observed E0283 failure
- Commit a replacement immutable candidate and return the parent to a full certification restart
