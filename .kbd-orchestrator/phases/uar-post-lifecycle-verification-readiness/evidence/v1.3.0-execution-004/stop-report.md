# Packet004 stopped at first failed gate

A4 was signed with the named SSH agent and adopted as contract f99c0e44b0a1994560453211c6528b247d6ed88030602d0f0a94470c644eea72, binding revision6, plan112. Inventory d1002cebf542eed02aa63f2d403a768942b30fc5c46217529baef49dfebb5051 and sealed manifest 42dfcd72a54bbd53dcaeab456c9da0ffb0624dea0eb97f3e0aa31788d8095530 bound 32082 inputs. No implementation edits followed approval.

The uncomfortable result: repair003 resolved the observed executable mismatch, but the real main acceptance still failed. This run does not certify UAR1.3.0. All three repairs are consumed; no fourth repair or automatic retry is authorized.

## Commands and observed results

The exact approved wrapper is retained in ../../v1.3.0-repair-003-verification-packet.md and the adopted contract. Its main-boundary receipts are in ../main-boundary-v1.3.0-004/.

- Owned fixture mkdir and Docker Compose up: exit0.
- `cargo build --locked --no-default-features --features server-full --bin universal-agent-runtime`: PASS, exit0, 3.8570973739842884 seconds.
- Exclusive retained executable SHA256: 7be53484dcc48e988982cc5f02a6d2c572f67a7715bf372a8bfae7d39127203d. Both actual boot path/hash comparisons matched. Two starts were observed; the manifest expected three, so this does not establish the whole identity obligation.
- `cargo test --locked --no-default-features --features server-full --test post_lifecycle_main -- --test-threads=1`: FAIL, exit101, 101.81448849398294 seconds; test body reported 0 passed, 1 failed. Both commands used CARGO_TARGET_DIR=target/uar-post-lifecycle-main through the approved runner. Wrapper exited1; outcome BLOCKED.

## Scenario receipts

| Case | Actual result | Recorded reason |
|---|---|---|
| LIAB-01 | PASS | Direct real-store collaborator controls; not provider-dispatch proof |
| LIAB-02 | BLOCKED | request_transport_failed |
| ERROR-01 | BLOCKED | request_transport_failed |
| ERROR-02 | BLOCKED | request_transport_failed |
| COUNT-01 | BLOCKED | recorded final body differs from validated serialization; trusted builder scope; standalone exact-fit unsupported |
| HISTORY-01 | FAIL | create_one returned404 |
| KB-01 | FAIL | create_kb_ab returned404 |
| CAS-01 | FAIL | discriminator assertions failed; required ordering/fault controls not all observed; see cas-store-collaborator.json |
| BTHREE-01 | NOT_EXERCISED | Original archive absent; historical disposition only |
| IDENTITY-01 | PASS in driver | Both observed launches matched; overall acceptance still requires successful teardown and runner summary |

Totals: 2 PASS, 4 BLOCKED, 3 FAIL, 1 NOT_EXERCISED. The expected409 conflict control succeeded and is not a failed response. Exact transport, HTTP404 and serialization root causes were not investigated after the failed gate; no speculative backend diagnosis or repair is claimed. The CAS receipt additionally records unrelated adapter contention classified FAIL_UNRELATED_CONFLICT_REPORTED_AS_CAS_MISS; interpreting and correcting this requires new authority, not a waiver.

## Cleanup and stop

Both main children and the peer received the approved stop and were awaited with exit0. Exact-owned row cleanup reported PASS for12 rows with absence observed; raw/store scope released. No namespace/database deletion occurred. Teardown overall remained BLOCKED: two HTTP cleanup mismatches and processCountMismatch. Docker Compose stop --timeout30 store and down --timeout30 both exited0 and removed the owned container/network; disk data and evidence were retained. Full command receipts are in fixture-cleanup.json. No additional service/process probes ran.

Issue426 integration tests, all six supported-profile builds, independent Rust review and spec verification were NOT_RUN under first-failure stop. All14 obligations remain; acceptance tasks5.1/5.2 remain pending. No canonical completion, OpenSpec archive, phase reflection or publication occurred. Historical identity/authority blockers were specifically resolved; a current packet004 runtime blocker was recorded. Verification is CLOSED.

## Allowance and changes

Original epoch and240-active-minute/240-host-dispatch limits are unchanged. Only the signed5213.007-second human wait is excluded; other waits are conservatively included. budget-at-stop.json records actual unique dispatches and the active-time upper bound at the bookkeeping cutoff. Repairs3/3, remaining0. Pre-epoch effort remains UNKNOWN.

Only derivative execution receipts, canonical blocker transitions and append-only session memory were written after the gate. No production, fixture or contract source changes and no unrequested guards were added. Frozen source and previous evidence remain intact. Phase completion and later obligations remain unverified because the first main gate failed.
