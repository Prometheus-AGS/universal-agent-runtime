## Source evidence and scope
Current native UAR source237c8d31 and Boss2b9 operation c14-e473095a-65a3-4965-a465-b0c2fa6ed3f5 report422 during coding preset setup. Boss uarCodingTeamPackage.ts:40 emits flat host extensions; UarCodingTeamAdministrationAdapter.ts:91 emits flat private binding extension. Draft.2 common.schema.json $defs.extension requires required and value with additionalProperties false. That is an exact source mismatch; source evidence alone does not prove which native request returned422 or establish a successful repair.

## Contract
Canonical agent selection: {required:true,value:{version:1,tools:[...],servers:[...]}}. Canonical private binding selection: {required:true,value:{version:1,tools:[],servers:[...],workspacePath:trustedPath}}. The external envelope and inner payload are separately strict. required remains an envelope authority bit and is not defaulted or copied from the payload. No legacy flat acceptance is added.

## Shared reader repair
TeamHostSelection keeps its existing internal fields. serde(from=TeamHostExtension) parses strict required/value, with a strict TeamHostValue payload, then moves values into selection. Existing tools/servers default-empty and optional workspacePath behavior stay identical. All three existing serde_json readers consume the repaired type without independent unwrapping logic:
- compiler/collaboration/validation/projection.rs required-capability conversion diagnostics;
- compiler/collaboration/team_execution/resolution.rs exact member host tools/servers and teamHostWorkspaceRequired;
- runtime/team_execution/host.rs attach_host_context bound private workspace comparison.

valid() still requires required=true, version1 and nonempty IDs. Attachment still checks exact saved binding revision, canonical directory, allowed servers and existing host admission. No tool grant broadening occurs. Malformed/optional selections retain current unsupported/scope/context diagnostics. Root must change Boss producer and saved-binding comparator coherently before delivery build.

## Boundary
Current failed packaged operation authorizes scoped repair. Plan committed before code. No Cargo/compiler/check/build/test/review runs; only scoped formatting if needed. Commit/push hooks suppressed under parent instruction and are not passing evidence. Lead alone rebuilds after complete source and performs actual failed native-operation rerun. Future UF reconciliation is separate; this task changes only current U checkout, preserving other-owner dist/ packaging output.
