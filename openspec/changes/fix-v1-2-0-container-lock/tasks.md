## Implementation

- [x] Reconcile Cargo.lock against committed submodules without unrelated updates.
- [x] Document the locked container dependency contract and failure cause.
- [x] Match pinned Liter 2.3.0 Network struct variant after CI exposed E0164.

## Verification and delivery

- [x] Independent artifact review of the lock delta.
- [x] Validate locked production resolution and deployment policy locally.
- [x] Commit and push the scoped lock repair to main (ce241cf5).
- [x] Commit and push the observed compiler repair (75bf899b).
- [x] Observe both native GHCR builds and merged manifest publication (run 39690429050).
