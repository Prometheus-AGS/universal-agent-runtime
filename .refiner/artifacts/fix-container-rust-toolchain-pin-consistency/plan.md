# Validation plan — `fix-container-rust-toolchain-pin-consistency`

1. Validate the Docker/repository/effective pin agreement and shell syntax.
2. Reconcile each fail-closed claim with its recorded nonzero negative control.
3. Check the clean positive and negative locked ARM64 fixture receipts use
   identical inputs and distinct dated toolchains.
4. Check the full clean production-image receipt names the implementation SHA,
   `linux/arm64`, dated argument, compiled dependency, image digest, and exit 0.
5. Strictly validate the OpenSpec change and audit only the child-permitted diff.
6. Validate the refiner constraints and manifest schemas, referenced output,
   non-empty files, log/decision consistency, and convergence state.
7. Persist the result without editing product code or asserting that the parent
   10,800-second operational-resilience certification passed.
