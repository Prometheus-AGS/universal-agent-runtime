## Implementation

- [x] Reconcile Cargo.lock against committed submodules without unrelated updates.
- [x] Document the locked container dependency contract and failure cause.

## Verification and delivery

- [x] Independent artifact review of the lock delta.
- [x] Validate locked production resolution and deployment policy locally.
- [ ] Commit and push the scoped repair to main.
- [ ] Observe both native GHCR builds and merged manifest publication.
