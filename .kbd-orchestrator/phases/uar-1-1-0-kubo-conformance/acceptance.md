# Final published acceptance

## amd64 follow-up — 2026-10-15

PASSED. The unchanged full program exited 0 against published child manifest sha256:94f78a7ed17197fc045baec514ed6ec30d3cf23433e8c26340ccfddb080fac90; result passed, full_acceptance true. All 12 shipped projections, 4 MCP/skills/AG-UI profiles, 12 elicitation decisions, 4 false approvals and 8 A2A cases passed, including the 300-second default capture. Raw result: receipts/uar-v1.1.0-amd64-acceptance.json; command, architecture proof, attempts and hashes: receipts/uar-v1.1.0-amd64-execution.json. No runtime or fixture source changed.

This was amd64 emulation on arm64 OrbStack, not native amd64 hardware execution. UAR image metadata was linux/amd64 and its executable was x86-64 ELF machine 62. The SurrealDB companion and grpcurl client were arm64. Native amd64 hardware remains unverified. Existing capture-window, infrastructure-projection and review-independence limits remain. The initial Docker companion-pull stop is retained as an incomplete attempt; the subsequent full run passed without changing assertions or timeouts.

Six supplemental Release asset hashes matched local files; the six original arm64 assets and hashes stayed unchanged. SHA256SUMS-amd64 covers the supplemental JSON receipts. Tag/source/index digests are unchanged.

## Original arm64 result

PASSED. The complete program exited0 against the pulled immutable digest sha256:064f5004fb3ba76506abce5cf81285b2a8aa3e54fa5a22e2c2bfc97b1211e43b, result passed and full_acceptance true. All12shippedconfigurationprojections,4MCP/skills/AGUIprofiles,12elicitationdecisions,4falseapprovalscenarios and8A2Atuplespassed. Source andclientfixturesmatchedv1.1.0; no UARsource/binarypatches. See receipts/published-acceptance.json and publication.json, release-notes.md and issue-handoff.md. StableGitHubReleasecreatedand#379handoffposted.

Limits: installed execution linux/arm64; nativeamd64built butnotexecutedlocally. Egresscapture excludes documentedinterfacecontrolframes, zerodrops/application/unclassifiedframes, launch-to-firstcapturegapnotobserved. Twelveprofiles are source-backedconfigurationprojections, not infrastructure rollout tests. NativeRusttests authoredbutunrun; realinstalledimageprogram waschosenacceptance. No deferredKubo requirement.

## Historical acceptance record

# UAR v1.1.0 release acceptance

Acceptance incomplete. The entire-phase implementation boundary is now recorded in `implementation-complete.md`; all remaining code, fixtures and documentation are authored before verification resumes. No tag, registry image, GitHub Release or OG-4 success exists.

## Observed results and provenance

The latest local candidate is `sha256:2bf3ce5fea8aeb739f98080a5393772aaa4833fae9ff27e292da433b5fc4e00f` (image ID, not registry manifest). Production build r8 exited 0; Cargo reported release completion in 8m17s with zero warnings. `receipts/candidate-build-r8.json` retains the command, metadata and log hash. Later fixture/documentation edits have not been executed.

| Gate | Actual retained result | Image and receipt |
|---|---|---|
| Default startup | Passed: loopback listeners, required authentication, no-model response, 300-second packet capture, discovery, CORS and effective revisions | r5 image88efe49c; `candidate-defaults-passed-surreal-fixture-failed.json`, safe_default |
| Shipped configurations | Passed: twelve source-backed projections healthy with anonymous effective-settings and agent-list 401 | r5 image88efe49c; `candidate-profiles-passed-mcp-restart-failed.json`, shipped_profiles |
| MCP, skills and AG-UI | Passed the former four profiles and twelve elicitation decisions, exact credential/session/schema, isolation, restart, revocation and skill durability/failure checks | r5 image88efe49c; `candidate-mcp-passed-a2a-get-failed.json`, profiles. Newly authored explicit AG-UI false approval has not run |
| A2A | Enforced JSON-RPC approve and deny passed, including native activation ineligible under limited credentials; first gRPC case then failed fixture parsing of omitted totalSize | r8 image2bf3ce5f; `candidate-a2a-r19-failed.json` and two r19 tuple receipts |
| Published digest | Not run; image not published | Pending |

Each pass belongs only to its recorded source/image. No partial receipt establishes complete acceptance. Chronological failures and corrections remain in `execution.md` and receipts.

## Commands and observed outcomes

- `docker buildx build --platform linux/arm64 --load --tag uar:v1.1.0-candidate --metadata-file /tmp/uar-v1.1.0-candidate-build-r8.json --progress plain .`: exit 0; release8m17s, zero warnings.
- `python3 scripts/release/uar_1_1_acceptance.py --image uar:v1.1.0-candidate --local-tag --evidence /tmp/uar-v1.1.0-candidate-evidence-r3`: exit 1 after defaults passed; disposable storage fixture corrected subsequently.
- `python3 scripts/release/uar_1_1_acceptance.py --image uar:v1.1.0-candidate --local-tag --gate profiles --gate mcp --gate a2a --evidence /tmp/uar-v1.1.0-candidate-evidence-r4`: exit 1 after twelve profiles passed; restart fixture corrected subsequently.
- `python3 scripts/release/uar_1_1_acceptance.py --image uar:v1.1.0-candidate --local-tag --gate mcp --gate a2a --evidence /tmp/uar-v1.1.0-candidate-evidence-r12`: exit 1 after all former MCP profiles passed; A2A persistence correction required.
- `python3 scripts/release/uar_1_1_acceptance.py --image uar:v1.1.0-candidate --local-tag --gate a2a --evidence /tmp/uar-v1.1.0-candidate-evidence-r19`: exit 1 after enforced JSON-RPC approve and deny passed; gRPC fixture implementation remains open.

Six mandatory local GitHub Actions policy validations passed around build.rs edits. No workflow was edited and no product checks ran in GitHub Actions. A runtime teammate's premature scoped diff check is recorded as non-acceptance evidence.

## Full phase completion boundary

The runtime owner delivered every A2A client branch from the actual pinned contracts together. The trust owner delivered explicit AG-UI approval denial and zero-call assertions. The lead completed current documentation and the full source inventory. Earlier per-fix declarations are superseded; implementation-complete.md records the entire phase closure.

After that boundary, run the complete serialized local candidate program and the required source/spec gates. Commit the bounded source and evidence, publish the immutable tag and multi-platform image through the existing deployment workflow, record manifest and architecture digests plus workflow commit, pull the registry digest and run the full program without source patches. Only a passing complete published-image receipt permits the release success claim.

## Review, scope and limits

The isolated artifact review and one corrective review addressed the recorded C1/W1/W2/W3 source findings. Its report is `.agent-team/uar-core/reviews/uar-1-1-0-kubo-conformance/artifact-review.md`; the same-model-family limitation is retained. Later installed-image defects produced the no-model startup, raw-byte Surreal CAS and native activation registration repairs. Source review does not certify runtime behavior.

The file-by-file source inventory is `implementation-inventory.md`; requirement coverage is `requirements-inventory.md`. Guards implement operator roles, persisted native-agent restrictions, policy revocation and checked durable skill import. No unrelated production feature was added. Acceptance relays forward real peer bytes and never replace UAR execution or approval.

The local runtime is linux/arm64. Native test sources were authored but not executed; the selected completion gate is real installed-image integration. The 300-second egress observation requires zero drops and no application/unclassified frames, excludes explicitly documented interface-control traffic, and does not observe the launch-to-first-capture gap. Selected deployment listeners and remote databases follow the approved default-versus-deployment distinction.

The uncomfortable result is that historical partial passes coexist with unfinished fixture code and no published artifact. Neither build success nor a tag can close that gap.

## r20 current-image results and completed error-path implementation

The complete r20 program exited1 after current-image defaults, all twelve shipped profiles and all four MCP/skills/AG-UI profiles passed, including four explicit approval-denial scenarios. Receipt: receipts/candidate-r20-incomplete.json. The first A2A anonymous probe failed because it expected protocol headers on outer middleware401. The complete fixture error path and related documentation are now authored, all owners report no implementation item, and implementation-complete.md records the updated entire-phase boundary before any rerun. Only affected A2A acceptance remains locally; the published digest will receive the full program.

## Local acceptance complete

A2A r22 exited0 with all eight tuples passed. Current runtime image2bf3ce5f now has passing defaults,12 shipped profiles,4 MCP/skills/AG-UI profiles with12 elicitation decisions and4 explicit approval denials, and8 A2A cases. `receipts/candidate-local-acceptance.json` composes exact r20/r22 evidence and preserves the failed/partial raw receipts. Runtime source did not change between these gates. `git diff --check` exited0 with no output; managed OpenSpec validation exited0 and reported the change valid. Published digest acceptance remains pending.

## Integrated release candidate

Origin/main PR384 was merged cleanly in722ce006. Production r9 built imageebae4c36 with zero warnings (release10m46s). All4MCP/skills/AGUI profiles passedr23; all8A2A tuples passedr24 after pacing the client below the observedHTTP429 limiter. Exact combined receipts are indexed by candidate-integrated-acceptance.json. Default/profile passes remain tied to earlierimage2bf3ce5f; the published image receives the complete program to cover the integrated artifact.
