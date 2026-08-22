# Artifact Judge — Round 1

Verdict: **BLOCK**

1. The planned image build preceded creation of the commit it claimed to bind,
   so the exact release-source claim was impossible.
2. The specification governed every Rust compilation while the contract
   recognized only the backend command.
3. The moving-nightly negative control was not locked to the observed
   incompatible channel and did not require `rustc -Vv`.

Non-blocking: the implementation is minimal, and the mismatch,
floating-selector, locked-dependency, and complete-image checks form a sound
layered evidence model once the blockers are resolved.
