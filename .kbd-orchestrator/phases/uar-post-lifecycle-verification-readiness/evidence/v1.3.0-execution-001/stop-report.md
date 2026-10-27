# UAR 1.3.0 verification 001 — stopped at first failure

The amendment was signed through the operator-authorized local SSH agent using the registered public key. Actual adoption succeeded for contract d00c3c5bd3057ad70ce7324efb6d4476066271081f87833ed811f2929e521551, plan109, binding revision3. Direct user F/V approval is retained in operator-approval.json. The sealed manifest hash and source inventory are in execution.json.

Fixture A directory preparation and B pinned store startup exited0. Main launcher C accepted all32060 bound inputs. Its exact server-full binary build passed, exit0,753.8417172789923s. Its declared post_lifecycle_main Cargo test failed during compilation, exit101,757.9160010199994s. Runner exited1 and outcome.json is BLOCKED. No runtime case or teardown receipt was produced. This is not a product PASS.

Observed compiler errors:
- E0382 twice in tools/seed-minimax-verification/main.rs:225 and239: record_id is used after moving it into bind at212 and232. Cargo included this binary while preparing the integration target.
- E0277 three times in tests/support/post_lifecycle_store.rs:93,197,253: the SHA-256 digest output does not implement LowerHex for the current dependency.
- E0624 twice at tests/support/post_lifecycle_store.rs:188 and tests/post_lifecycle_main.rs:257: integration code calls pub(crate) permits_durable_work.

Warnings also remain: patched SurrealDB unused OnceLockExt; IngestionAdmission missing Debug; sidecar AsyncFd::new deprecation. They did not cause the exit101.

Per operator instruction, no repairs or retries were made. Approved owned store stop D and down E both exited0, removing the owned container/network and preserving disk data. These commands do not certify runtime teardown. Issue426, all six profiles, independent Rust/code reviews, screening, seven spec validations, canonical5.1/5.2 completion, archives, closure and reflection are NOT_RUN. Gate closed pending corrective source work, refreshed whole-phase inventory and explicit exact failed/invalidated gate instruction. The published source/tag/image remain as previously delivered; no publication changed.

The uncomfortable fact: the runtime binary compiled, but the approved integration command could not compile. No acceptance conclusion follows from publication or the successful binary build. Tool-call consumption requires an audit before any finite-budget closure; this receipt makes no closure claim.
