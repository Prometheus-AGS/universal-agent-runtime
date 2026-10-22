# Budget error projection source review

Task 13.25, Execute. Bounded artifact review of bound Messages projection in `src/server.rs`, ACP caller-tool settled-error projection in `src/uar/api/acp/handler.rs`, and `budget-error-projection-repair.md`. Also inspected actual collected normalized-event serialization and host event redaction. Own report only.

## Disposition

No blocking source finding in the scoped projection repair.

Messages now retains the normalized Error code together with its public message. Actual settled Error uses that captured error; missing evidence and other unsuccessful statuses retain the generic actual status. Join failures and lost observation still fail independently. A captured error accompanying Done still fails through the existing post-settlement path. Staged caller requests and successful terminal completion are not released on these error branches.

ACP reads Error code/message from the same serialized normalized events collected for this run, and only prefers them when actual settlement is Error. Cancellation and other statuses retain their actual-status failure, missing Error evidence retains the generic fallback, and join/collection failures remain errors. The numeric RPC error contract is unchanged; no caller handoff or completed receipt is synthesized here.

These fields come from the existing public host event projection: Error.message is scrubbed after capture, while pre-protected-admission errors receive the existing reduced static message. The adapter does not newly inspect a raw provider error, private anyhow chain, credential, SQL, budget row or ledger payload. Error code is preserved as the existing typed control field. The changed spans do not alter budget admission, accounting, provider dispatch, resource ownership or refusal timing; this source review does not independently prove a pre-provider refusal run.

## Evidence and limits

Only source reads (`sed`, `rg`, `cat`, scoped `git diff`) and this report write ran. No tests, checks, builds, imports, formatting, compiler probes, runtime or network commands ran. No production files were edited. Global reviewer verification remains CLOSED; the lead owns its scoped local functional loop. Native Messages JSON/SSE and ACP public envelopes, budget assertions, compilation and cleanup remain unverified on this source. Same-family isolated source review is not cross-model adversarial evidence; no acceptance or publication claim is made.
