## 1. Contract and workflow checkpoint

- [ ] 1.1 Record the repository source receipt for C01 `afc.convergence-contract.v1` and `afc.identity-state-action.v1` `1.0.0`, I1 merge `79414bb7e134dad45330008999ee1dca999d44af` and implementation `676f995c73dc7dacdce71c929ffac88ac3618bb0`, C02 `afc.governed-effect/1` `1.1.0` at `59df10a102c5d46da9db5983a73fbcf20ca09590`, and initiative tasks C03.1-C03.3; verify each revision resolves and the receipt names this repository change without moving unrelated KBD state.
- [ ] 1.2 Claim one writer for each ordered surface in design section 9 and record that review, verification, and integration roles remain dormant until production implementation is complete; verify no claimed file overlaps another active assignment.

## 2. Draft.2 collaboration profile and schemas

- [ ] 2.1 Publish `docs/agents/collaboration/v0.1.0-draft.2/` with the 15 exact normative headings, independent version domains, compatibility rules, and immutable source pins; verify draft.1 files and `$id` values remain unchanged.
- [ ] 2.2 Define the draft.2 `common`, top-level discriminator, AgentDefinition, TeamDefinition, WorkflowDefinition, PackageManifest, private DeploymentBinding, private RepresentationGrant, ConversionReport, EffectiveBindingReceipt, and DeploymentBindingTemplate schemas; verify every `$id`, `profile`, `kind`, required member, `additionalProperties` rule, and package-kind exclusion agrees with the spec delta.
- [ ] 2.3 Add valid and invalid draft.2 schema fixtures for all document kinds, canonical SkillRef, unknown optional/required extensions, immutable digest/lock conflicts, private-state exclusion, and sanitized templates; verify the fixture inventory covers every schema branch without running the phase integration gate yet.

## 3. Authoritative runtime validation and evidence types

- [ ] 3.1 Extend `src/uar/domain/collaboration.rs` with draft.2 document identity, full SkillRef, RepresentationGrant, ConversionReport, EffectiveBindingReceipt, DeploymentBindingTemplate, and migration receipt types; verify their serialized field names and revision/immutability classes match the published schemas by static comparison.
- [ ] 3.2 Replace duplicated shape decisions in `src/uar/compiler/collaboration/validation/**` with the registered draft.2 schema/discriminator pipeline while retaining relational, digest, graph, capability, and authority checks as semantic validation; verify callers cannot supply a kind that disagrees with the document.
- [ ] 3.3 Produce RFC 6901 field diagnostics with source identity/profile, target profile or harness, disposition, stable reason code, redacted message, and optional effective-binding reference; verify unknown required semantics block activation and unknown optional values remain present in the immutable record.
- [ ] 3.4 Implement structured authority/recognized-secret exclusion for every portable extensible value with redacted diagnostic paths and no secret logging; verify the planned fixtures cover extensions, source descriptors, legacy sections, skill config, contracts, and free text.

## 4. Legacy Agent Markdown and lossless canonical conversion

- [ ] 4.1 Update `src/uar/compiler/{ir.rs,parser.rs,storage.rs}` to preserve source metadata identity, exact source headings, authored presence, complete v1.1/v2 IR, original authoring sections, and explicit rename mappings; verify the adapter accepts only the fixed Agent heading vocabulary and does not create Team, Workflow, Binding, or Grant Markdown kinds.
- [ ] 4.2 Normalize legacy skill entries into the canonical SkillRef superset `id`, `version`, `digest`, `required`, `config`, `entrypoint`, and `requiredTools`; verify omitted, null, empty, explicit-default, and required values remain distinguishable in the conversion report and stored source record.
- [ ] 4.3 Add explicit draft.1-to-draft.2 migration without rewriting draft.1 identities or relabeling source digests; verify every normalized definition records its source profile/digest and target profile/digest in an immutable migration receipt.
- [ ] 4.4 Update AgentArtifact projection to retain the canonical definition reference and pass full SkillRef plus all five v2 requirements into private binding; verify projection storage alone never produces an `exact` or supported runtime disposition.

## 5. Effective binding and private RepresentationGrant boundary

- [ ] 5.1 Resolve exact skill version/digest/config/entrypoint/tool requirements and all five v2 fields during DeploymentBinding preflight; verify each required field either appears in an EffectiveBindingReceipt with its enforcing runtime component or yields `required-unsupported` before admission.
- [ ] 5.2 Persist EffectiveBindingReceipt with the private binding revision and make ordinary-agent admission consume that receipt; verify requested/effective definitions, skills, models, runtime capabilities, and current policy revision are all bound to the admitted run.
- [ ] 5.3 Add private RepresentationGrant storage, revision history, lookup, and DeploymentBinding reference validation using C02 grant ID, issuer, subject, revision, status, validity, and constraint digest; verify there is no grant issuance, consent workflow, human impersonation, delegated approval, or representation action path.
- [ ] 5.4 Revalidate current policy and RepresentationGrant state after waits and before governed effects; verify a stale revision, revoked/expired status, or changed constraint digest cannot reuse an earlier binding or approval decision.

## 6. Import, export, and administration

- [ ] 6.1 Add canonical package export/reimport with exact definition digests, byte digests, optional extension values, migration reports, and immutable lock closure; verify a mutable alias change cannot alter an installed binding and a required unsupported target export is refused.
- [ ] 6.2 Add explicit DeploymentBindingTemplate export that removes private owner, workspace, runtime, credential, connection, decision, secret, and grant values and reports every required rebind; verify templates cannot be installed or activated as DeploymentBinding records.
- [ ] 6.3 Extend existing collaboration REST and MCP administration with typed conversion reports, binding receipts, package export, and sanitized-template export; verify authentication and owner/workspace scoping remain on every private operation and no new service or port is added.
- [ ] 6.4 Add `collaboration_definition_packages_v2`, `collaboration_deployment_bindings_v2`, `collaboration_conversion_reports_v1`, and `collaboration_representation_grant_refs_v1` only with their implemented behavior, retaining v1 advertisements while supported; verify capability output lists exact profiles, schema IDs, export classes, and activation limits without team-execution or C17 claims.

## 7. Complete-phase integration gate and receipts

- [ ] 7.1 Finish the integration fixture set before executing checks: legacy v1.1, explicit/non-explicit v2 defaults, full SkillRef, canonical Agent/Team/Workflow package, immutable conflict, unknown optional/required semantics, private values in every extensible location, stale grant/policy after wait, sanitized template, required-unsupported target, and downgrade refusal; verify each fixture maps to a spec scenario.
- [ ] 7.2 After all production tasks 2.1-6.4 are complete, run one repository integration gate over legacy and canonical import → schema/semantic validation → atomic catalog persistence → retrieval → private binding/grant resolution → ordinary agent admission and execution → package export/reimport, recording exact source/profile/schema/package/policy/grant revisions and observed outcomes.
- [ ] 7.3 If the final gate fails, fix only the observed failures and rerun only that failed gate; verify the final acceptance receipt distinguishes schema validity, catalog persistence, effective runtime semantics, private-authority exclusion, and ordinary execution evidence.
- [ ] 7.4 Run the repository-required OpenSpec validation for `afc-c03-lossless-definition-bindings`, complete the source and acceptance receipts, and link their exact revisions and outcomes back to initiative C03.1-C03.3; verify downstream D-MINI/full-pack/BossFang adoption remains pending rather than being reported as complete.
