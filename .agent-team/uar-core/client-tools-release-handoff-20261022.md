# Caller-tools release continuation — 2026-10-22

Execute finite CT1–CT7 contract in `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/client-tools-exit-contract.md`.
User explicitly authorized local all-five functional test/fix/retest and publication; latest instruction says execute, fix remaining failures and publish. Unrelated verification CLOSED. Do not ask for repeated micro-approval or expand scope.

## Current facts

- All59 active-phase implementation items complete (baseline13 + caller46); canonical6/6 changes complete. Other run-wide phases remain outside this contract.
- Native source accepted by build015 + gate017. All95 core,60 safety,5 cancellation; ordinary cold, warm/cold actual local team graph, warm/cold budgets, settled mixed-host/client crash recovery allfive passed. Actual host effects never repeated; exact completed retries add no provider calls. Cleanup confirmed, intentional candidate SIGKILL separate.
- Gate017 exit2, status extended_passed_remaining_release_gates, is intentional broader excluded certification status. No scoped failure or unexpected exit. Do not rewrite the runner to hide2.
- Native proof: `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/client-tools-local-gate-017-receipt.json`.
- Normal enabled policy and commitlint hooks passed; source commit `c065bb0798f5bbbd4a878a0c41c04cb4e8a75ad6` pushed main and annotated `v1.2.2`. Remote peeled tag verified samecommit.
- Publication dispatched once: `gh workflow run deploy.yml --ref v1.2.2`; run39863098430, https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39863098430 . Both Linux platform jobs in buildandpush; not yet published. No local runtime/build active.
- Deployment wait began22:11:33UTC, excluded from active no-progress repair budget. Current progress JSON in target/client-tools-v1.2.2-deploy-progress.json.
- Existing1.2.0/v1.2.0 index dec5284e97b78ff65a66ef19eb43632154e23ea946f05a1d241cf979dd09566d, source035bbdc702ba20daaa1add7294b35f5783ed6e0e. Existing1.2.1/v1.2.1 index a012df13ba86668410c30b92625c018950a5c3764bb83d5254146a3295845ea4, source dd896d5a66eda825f0076c1f35ba05f2dea6324c. Never move these or v1.2.2 source tag. Code repair after published source needs new patch version.
- #425 original frozen1.2.0 handoff already delivered in onecomment and closed; do not add duplicatecomment. No Kubo edits.

## Finish, no further plan

1. Wait actual run success for both platform builds and merge. Read failed deployment diagnostics only if failure. Do not put product tests in GitHub Actions.
2. Verify new Docker1.2.2/v1.2.2 exactsame canonical OCI index, amd64/arm64 and attestation descriptors; old fourDocker tags unchanged. Capture source/run/index/platform receipt.
3. Pull exact canonical index locally. Record actual Linux amd64 platform/config identity. Do not claim ARM functional acceptance.
4. Run existing supervisor INSIDE exact published image with minimal fixture bundle already prepared at `target/client-tools-image-bundle-v1.2.2`. It contains ONLY scripts/release/client_tools_local_gate.py, client_tools_acceptance.py, tests/fixtures/client_tools_provider.py, policies/, empty target/. Current copies include13.28 correction. No repo configs/credentials/private HOME mounted.

Command after replacing CANONICAL_DIGEST with observed new digest:

`docker run --rm --name uar-client-tools-v122-39863098430 --network none --no-healthcheck --platform linux/amd64 --user 501:20 --mount type=bind,source=/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/client-tools-image-bundle-v1.2.2,target=/workspace --workdir /workspace --entrypoint python3 ghcr.io/prometheus-ags/universal-agent-runtime@CANONICAL_DIGEST scripts/release/client_tools_local_gate.py --binary /usr/local/bin/universal-agent-runtime --evidence target/client-tools-published-image-001 --extended`

Docker run flags and imagetools inspect --raw/--format were read via CLIhelp. Record exact expanded command/digest before execution. All fixture/candidate/driver run inside isolated container loopback, no external traffic. Owned normal and intentional-crash lifecycle receipts retained in host bundle target.

5. Require all scoped reports + confirmed cleanup/expected exits, not merely process exitcode. Runner2 retains broader excluded certification exactly like native gate017.
6. Write final publication/provenance/image-gate receipts and update docs/releases/v1.2.2-kubo.md, v1.2.2-publication.md, CHANGELOG, finite CT receipt and scoped reflection. Native and actual Linuxamd64 containerproof with SurrealKV/controlledpeer only; PG compile, ARM build, paid providers/arbitrary graphs/bundledassets not certified.
7. Commit documentation/receipts separately with enabled hooks (`pnpm_config_verify_deps_before_run=false git commit -F <ownedmessagefile>`), push main [skip ci]. No retag/rebuild for receipt-only edits. Create GitHub release v1.2.2 with actual index/source/run and attach final contract/proof receipts. No need a PR. Stop when CT1–CT7 complete.

## Checkout preservation

Preexisting modified .codex/config.toml excluded from staging.254 unrelated untracked paths/private estate wikis preserved. Never git add -A. Current uncommitted edits are publication receipt/draftdocs/editorial docs changes; they do not change production source. Stage explicit paths only. Temp commit message .agent-team/uar-core/v1.2.2-commit-message.txt is owned and untracked, do not publish it. All code writers/source critics completed with handoffs in OpenSpec/.agent-team. No new features, testsuites or verification framework.

## Published-image result, 2026-10-22

The immutable1.2.2 image failed its local Linuxamd64 acceptance gate:94/95 core cases passed; `agui:reject:partial` setup returned `host_cleanup_unconfirmed`. Extended image stages did not run. This image is not functionally accepted. Native gate017 does not certify this failed delivered-image run. Preserve both1.2.2 tags; any production repair ships as a new patch. Receipt: `.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/client-tools-published-image-001-receipt.json`.

## Current handoff:1.2.3 deployment build active

Source/tag: `c4a656251b643bd83c8f1407286480dbfac76708`, annotated `v1.2.3`, both pushed main/tag. Normal policy hook0.66s andcommitlint0.91s passed; first message-only attempt failed body-line length and was wrapped, no bypass. Docker1.2.3 was notfound beforedispatch, so no prior version overwritten.

Sole manual test-disabled deployment: https://github.com/Prometheus-AGS/universal-agent-runtime/actions/runs/39875057583, created2026-10-22T23:56:57Z. amd64/arm64 Build and push by digest active. Historical1.2.0alias and sidecars SKIPPED. External build wait is excluded from finite contract active no-progress budget. No local build/runtime currently active.

Current production correction: existing RunStreamState owns per-run acknowledgment mutex initialized inall3 constructors; shared confirm holds it before durable load through root/childjoins, settlementCAS andwinnerread. Cold alreadysettledrow works; coldpendingstillfails. Fixed-stage diagnostics returnoriginalerror. Cancellation maydropguardwhile ScopedPersistence retains alreadyacceptedCAS; no crossprocess/unknown-outcome exactlyonce claim. Original discarded innererror unproven; exact callcorrelation proved two AGUIconfirmers coulddisagree. See singlewriter writerhandoff andfresh isolatedRustcritic report.

All61 current phase source items complete, canonicalrevision266/plan97;22 root-run incomplete changes belong other phases. Build0171.2.3 PASS9m09; nativegate019 EXIT2 all9scopedgroupsPASS:95core60safety5cancel, ordinarycold, graphwarm/cold, budgetwarm/cold, settledmixedcrashrecovery allfive; cleanup/expectedexitsconfirmed. Exit2 keeps broadercertification excluded by CT1–CT7. Exact image proof stillPENDING. Reviewed source committed; onlypublication/evidence docs dirty. Private .codex/config.toml andunrelated254wiki/Literstate untouched/untracked. Do not stageall.

Next: poll39875057583 sparsely and communicate atmost60s intervals. On success verify paired1.2.3/v1.2.3 exactindex+2platforms+attests+SLSAvcsrevisionc4a andall6oldtagsunchanged. Previousfrozenreceipts in docs/releases/v1.2.2-registry-provenance.json. Recordexactindexcommand beforeexecution; minimalownedbundle ready at target/client-tools-image-bundle-v1.2.3 with3existingPythonfiles+policies+target, no userconfigmount.

Exactimage gate pattern (substitute newlyverified index/runcontainername): `docker run --rm --name uar-client-tools-v123-39875057583 --network none --no-healthcheck --platform linux/amd64 --user 501:20 --mount type=bind,source=/Users/gqadonis/Projects/prometheus/universal-agent-runtime/target/client-tools-image-bundle-v1.2.3,target=/workspace --workdir /workspace --entrypoint python3 ghcr.io/prometheus-ags/universal-agent-runtime@sha256:NEW_INDEX scripts/release/client_tools_local_gate.py --binary /usr/local/bin/universal-agent-runtime --evidence target/client-tools-published-image-002 --extended`. Retainlocalresult beneathbundle/target. Passingfullgate exits2 bydesign withall9groups andactualcleanupconfirmed; do notbroadenorfixexit2.

Onlyafterexactnewimage passes create GitHubreleasev1.2.3 withactualdigest/source/run/acceptance limitations, attachfinalcontract/provenance/summaryreceipt; finaldocs/recordscommitmainwithenabledhooks+[skip ci], no source/tag/image changes. NoGitHubrelease1.2.3 yet. Do notclaim1.2.2accepted; do notretagoldversions. NoKuboedits orduplicate#425comment (#425frozenhandoffalreadyclosed). ARM image build !=ARMfunctionalacceptance; PGcompiled !=PGfunctionalacceptance; controlledpeer !=paidprovider certification. Userexplicitlyauthorizes scopedlocaltest/fix/retest/publish; unrelatedlockCLOSED, no newpermissionmicroloops. Stopatfinitecontractsuccess.

## FINAL — scoped contract complete

Run39875057583SUCCESS; paired1.2.3/v1.2.3 index7fa7973fdcc0968d1fd0fa3a497f9c5a51daa97a653a48f01a2a1507fdbe3195, bothplatforms/attests/sourcec4a andall6oldtagsverified. ExactLinuxamd64 imagegate002EXIT2 all9finitegroupsPASS,95core60safety5cancel, ownedcleanup/expectedexitsconfirmed. GitHubreleasev1.2.3publishedwith7safeassets. Finalrecords-onlycommit/pushmain completeshandoff; no source/tag/image mutation afterward. STOPcaller-toolfeaturework. ARMimagebuiltbutnotruntimecertified; PGcompilednotfunctional; controlledpeerdoesnotcertifypaidproviders. NoKuboedits. Prior pendinginstructions in this handoff are superseded by this finalreceipt.
