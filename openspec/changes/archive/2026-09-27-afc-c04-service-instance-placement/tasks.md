## 1. Contract

- [x] 1.1 Specify stable identity, profile, workspace locality, ownership, endpoint roles, opaque references, and compatibility diagnostics without moving inventory or lifecycle ownership into UAR.
- [x] 1.2 Specify new placement, reattachment, unsupported migration, effective binding, and C03 deployment-binding enforcement.

## 2. Runtime implementation

- [x] 2.1 Add configured service-instance identity and descriptor construction aligned with A2A identity.
- [x] 2.2 Extend capability discovery and add structured compatibility evaluation.
- [x] 2.3 Enforce service placement on new runs and reattachment, expose effective binding in run inspection, and refuse migration.
- [x] 2.4 Validate C03 deployment bindings and effective receipts against the live instance.

## 3. Boundary integration

- [x] 3.1 Hand the additive capability and admission contract to The Boss and BossFang consumers.
- [x] 3.2 Run the shared C04 acceptance gate only after all repository slices are complete.
