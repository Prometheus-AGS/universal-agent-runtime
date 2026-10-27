# Knowledge index regeneration on Mac

Scenario KNOWLEDGE-REGENERATION-01 changes the active embedding space for a disposable KB from FastEmbed bge-small-en-v1.5 (384 dimensions) to OpenAI text-embedding-3-small (1536). It requires the incompatible old index to refuse retrieval, regenerates from the retained document through REST, and retrieves its original private fact from the new index. No document reupload or model-label-only substitution is accepted.

Run only:

```sh
node node_modules/@cucumber/cucumber/bin/cucumber.js --config scripts/behaviors/knowledge-regeneration.cucumber.mjs
```

Use the retained UAR_BASE_URL and UAR_WORKSPACE_ID. For this Mac fixture, run `node scripts/manual-host/knowledge-regeneration-macos.mjs`: the separate operator launcher provisions the already-configured OpenAI endpoint/key, invokes only the Cucumber command above with a credential-free environment, then restores the exact original host configuration. The BDD never reads credentials or calls host provisioning. Its cleanup deletes the disposable KB and restores the original embedding settings; the host leaves UAR running. Separate $10 embedding caps remain; Cohere is unused. One tiny ASCII document plus query has a conservative input-byte cost estimate; actual billing usage is not exposed by UAR.

Scope: one document, one cross-backend/dimension model change, one regeneration and retrieval. This does not certify all migration sizes, crash recovery, or storage backends. Earlier passing scenarios are not rerun.

Result: PASS on Mac, 2026-10-26T19:40:34.981660-05:00. First live attempt: `1 scenario (1 passed); 6 steps (6 passed); 0m 7.328s`. Host launcher also exited0 after restoration. One document retained its ID through generation `d3a7dbaf-8ca9-4af7-acd2-9b13431f0210`; retrieval returned its private seal and coefficient73 with score0.8107894. The old index correctly returned409. Disposable KB deleted; original embedding settings and exact host configuration restored; UAR PID97354 stays running.

[PASS receipt](knowledge-regeneration-01-macos-pass.json). No production change, build, earlier-scenario rerun or live failure. Source review corrected the initial credential placement, aggregate timeout and failed-start cleanup design before execution. Failed-start recovery itself was not exercised. OpenAI billing usage is not returned by UAR; the recorded two-input estimate is not an actual charge. Cohere unused; existing separate $10 caps retained.
