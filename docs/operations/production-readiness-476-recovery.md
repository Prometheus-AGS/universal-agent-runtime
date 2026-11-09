# Production readiness recovery checkpoint

This branch preserves recovered source for issue #476, including the embedded SurrealDB 4.0 child and the pending integration of RC.2. It is a work-in-progress source backup. Qualification is incomplete.

The repaired Linux amd64 image `sha256:9e86de9e34733dbc7430f033109428afa0223bb0323e36e341c4f2d0d241353e` passed the existing embedded seed, clean stop and reopen Cucumber scenarios (three scenarios and ten steps). The unsupported shared-store configuration was refused. The source inventory used for that image was `72336ddcc93aec82ca077dc3afa8c52e99d6e56d4a8ee67d365286c2754405a7`. That inventory precedes this checkpoint commit.

The pinned published RC.2 image started and produced a real stopped seed after one protected checkpoint continuation. The independent tool journal showed a single approved effect and no replay. The planned guest reboot then reformatted the isolated Docker disk. The candidate image and build cache were lost. The stopped E1 store, independent controller journal, test receipts, frozen build inputs and canonical KBD state are retained in a private recovery backup. No second continuation, claim-history readback, migration or rollback result is claimed.

Resume by rebuilding the frozen candidate and exporting its OCI image outside the VM before further recovery exercises. Reuse the preserved E1 store and reboot evidence to finish E2, then run the existing SDK parity and rollback drivers and the remaining lifecycle acceptance. Close the child only after those results are recorded. Parent qualification remains pending.
