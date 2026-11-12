# Offline upgrade of UAR embedded SurrealKV to 4.0

This procedure covers the Linux amd64 UAR OCI service and its dedicated embedded SurrealKV volumes. It targets the UAR artifact linked to the pinned SurrealDB 4.0.0 SDK and engine.

## Qualification scope

The SurrealDB upgrade phase passed five scenarios for the recorded Linux amd64 candidate behind a trusted gateway. The evidence covers fresh persistence, offline migration from the pinned 3.3.0 SurrealKV baseline, selected data and query parity, storage shutdown and reopen, and rollback after interruption or a recorded write disposition. The original snapshot stayed byte-for-byte unchanged.

The child evidence packet is identified by SHA-256 `2c8649980b135e69fdcc14bd99ee8ad6fbc9ce19f9c1e150795264839749a188`. Its scope is the recorded artifacts, configuration, and fixture data classes. Final release qualification remains tracked in the [release checklist](../releases/v1.4.0-final.md#finite-selected-obligations). The active standalone companion server now uses SurrealDB 4.0.0, pinned by digest in [versions.toml](../../versions.toml). The earlier embedded migration evidence applies to its recorded artifacts.

For current deployments, attach a fresh or already converted SurrealDB 4.0
SurrealKV store. The deployment templates preserve the existing volume and
datastore path. They do not perform a conversion. Keep legacy storage
offline and retain its recovery point before changing a volume assignment.

## Historical migration procedure

The following procedure records the earlier 3.3-to-4.0 qualification.
Current 4.0-only qualification uses fresh stores and 4.0 recovery points.

## Bind the source and recovery point

Use a store last written by the pinned UAR **3.3.0 embedded SurrealKV** baseline and closed by that same artifact. The upstream 3.3.0 source export identifies revision `238bfeb11f5725bebed370167656748df8067595`. The baseline uses the UAR lifecycle overlays for `surrealdb`, `surrealdb-core`, `surrealdb-engine-api`, `surrealdb-engine-local`, `surrealdb-kvs-surrealkv` and `surrealdb-kvs-rocksdb`, all at `3.3.0`. Bind the actual patch contents from the baseline checkout. The historical standalone SurrealDB 3.3 server image is a separate artifact.

Before attempting an upgrade, record an eligibility receipt with all of the following fields. Bind it to the exact snapshot and retain it with the recovery material. Stop if a field is unknown or the recorded source is outside the baseline.

| Field | Required binding |
| --- | --- |
| Baseline artifact | UAR source revision and source inventory, Linux amd64 OCI image ID or digest, runtime executable SHA-256, `Cargo.lock` SHA-256 and hashes of all applied 3.3.0 lifecycle overlay trees. |
| Candidate artifacts | UAR 4.0 image ID or digest, its source and `Cargo.lock` SHA-256, and the SHA-256 of the separately built migration runner from that same source and lockfile. |
| Effective settings | Nonsecret settings, namespace and database, UID/GID, volume and store identities and paths, plus memory and ingestion settings that determine the fixture scope. The default persistence endpoint is `surrealkv:///var/lib/uar/data/uar.db`; the shipped container runs as `65532:65532`. |
| Store layout and schema | For every enabled embedded store, record its owner, effective path, actual backend and format, schema definitions, applied migrations, indexes and applicable feature settings. The eligible disk layout is the 3.3 SurrealKV manifest format 1 under the recorded baseline's default embedded engine settings. Actual RocksDB, interrupted conversions and unknown formats are ineligible. A historical `rocksdb://` setting alone does not identify the disk format. |
| Recovery point | Write-freeze boundary, successful shutdown receipt, original volume and immutable backup identities, snapshot path and per-store inventory of relative directories and file paths, byte sizes and SHA-256 hashes. Record the artifact and distinct restore volume used to demonstrate 3.3 reopen. |
| Data coverage | Deterministic fixture IDs, owners, expected logical values, index/query observations and source and destination lineage for each class below. Record disabled classes and classes outside the supported scope. |

Freeze writes through the trusted gateway and stop every owner of each embedded store. Wait for successful shutdown before taking the backup. Inventory every enabled embedded store under the deployed settings, including memory when it uses its own path. Record shared store paths once. Take and retain a separate verified, immutable snapshot of each distinct store. Demonstrate that the pinned 3.3 artifact can reopen a fresh restore of the backup and read the recorded recovery point. Preserve the original and immutable backup, and keep the source paths unexposed to 4.0 startup. Keep store owners stopped and writes frozen until the selected cutover or recovery decision.

## Prove default startup refuses conversion

1. Restore the snapshots to separate disposable refusal volumes. Before starting the exact 4.0 UAR image, record the quiesced store inventory and hashes. Mount only the refusal copies. Keep the upgrade option unset and use the normal shipped configuration for startup. Record the specific refusal for an unconverted store and confirm the service never becomes ready. A launch, permission or unrelated startup failure does not count as this refusal.
2. After the refused process has exited, compare the quiesced copy to its own pre-start inventory. Require identical paths, byte lengths and hashes, including the legacy manifest and absence of conversion markers. Record the before and after format and file inventories. Use a fresh process running the pinned 3.3 artifact to reopen **that refusal copy** and confirm the recorded fixture. Repeat for each distinct store, using separately prepared other stores where needed so an earlier startup failure cannot mask the store under examination.

## Copy and convert each store offline

The migration runner is a source-built companion. It is built from the 4.0 candidate checkout and is separate from the shipping OCI image. The following build belongs to the approved qualification or operator procedure.

```sh
cargo build --locked --no-default-features --features server-full --example surrealql_store_runner
```

Use the bound runner artifact on a trusted Unix host. Keep the quiesced snapshot immutable and the parent of the new destination under exclusive operator control throughout the command. Run with the storage owner's permissions and ensure the candidate service will have access to the converted files. Supply UTF-8 filesystem paths with a pre-existing destination parent and enough space for the entire copy and engine conversion. Keep `?` out of the destination path because the SDK interprets it as the start of connection options. Use a distinct, previously absent destination for each store.

```sh
"$UAR_MIGRATION_RUNNER" --migrate-3.3 "$UAR_3_3_SNAPSHOT" "$UAR_4_DESTINATION"
```

The runner reads snapshot files and creates a private destination. It rejects overlapping directories, existing destinations, unsupported entries and hard-linked files. It verifies the copied legacy format before connecting the 4.0 engine. The only connection that opts into `experimental_upgrade_from_3x` is the typed embedded connection to the newly created copy. The conversion of that copy is one-way. The snapshot's exact 3.3.0 artifact, configuration and schema provenance come from the eligibility receipt.

Retain the command, exit status, stderr and stdout JSON lines. The `copy-start`, `copied` and `closed` stages identify the canonical snapshot and destination paths and give the file count where complete. Before copying, the runner creates `UAR_UPGRADE_INCOMPLETE` in the destination. It clears the marker only after the upgraded engine closes successfully. The migration and restore commands refuse a source carrying this marker, even if its legacy manifest was copied before interruption. Only a successful exit with `closed` permits starting the candidate on that destination. On interruption or any error after directory creation, quarantine the destination and record its identity and last completed stage. Restart from the immutable baseline with a different destination. Keep quarantined destinations out of all startup and restore configuration.

## Compare logical data on both artifacts

Compare the original and immutable snapshot inventories to their recorded baseline before and after the upgrade. Independently confirm original-store reopenability with the pinned 3.3 artifact after recording the unchanged bytes and while writes are still frozen. Preserve the immutable backup throughout.

Start the pinned 4.0 artifact on the converted destinations with the standard connection configuration. Use the trusted fixture gateway and separate synthetic owners. Record a baseline and candidate result for each applicable class, with stable IDs and expected owner visibility. Compare contractual ordering, including ties, and compare missing, null and other typed values where the API exposes the distinction. Confirm candidate-only writes remain readable after a fresh reopen. Record the scope of every comparison.

| Class | Comparison |
| --- | --- |
| AG-UI runs, contexts and events | Owner-scoped run and context IDs, event sequence and durable payloads, and refusal of cross-owner reads. |
| Checkpoints and history | Session, turn and checkpoint IDs, resume linkage, ordered history and retained values. |
| Canonical tool receipts | Run and receipt keys, occurrence linkage, outcome and idempotency behavior. |
| Knowledge and ingestion | Knowledge base, document, chunk and ingestion identities and states, links between them, owner visibility and index-backed result membership including an out-of-scope candidate. |
| Applicable embedded memory | Owner and scope, record ID, content and metadata, retention or expiry state and the supported retrieval path. Bind the memory store separately when it has its own embedded path. |

If any required comparison, independent reopen or refusal integrity observation fails, keep writes frozen and use the retained 3.3 recovery point. An engine close alone does not establish fixture parity or permission to cut over. Once the candidate accepts writes, account for the post-baseline delta and obtain the operator's recorded disposition before restoring service on a distinct 3.3 volume.

## Restore the 3.3 recovery point

Keep the gateway write freeze in place and stop the candidate before rollback. Use the retained immutable snapshot and a previously absent 3.3 restore destination. A converted or incomplete 4.0 directory cannot be the recovery source. Keep the candidate and its write inventory available for reconciliation.

The operator prepares a rollback plan with `kind` set to `uar.embedded-surrealdb-rollback/v1`, the canonical `snapshot` path, a nonempty `recovery_point`, and `writes_frozen` set to true. Set `candidate_write_receipt_sha256` to the digest of the actual candidate write inventory receipt, including an explicit empty inventory when the candidate accepted no writes. Copy its accepted write IDs into `accepted_writes`. Record exactly one disposition for each ID. Each `dispositions` entry has an `action` of `accept-loss` or `preserved-separately`, and a nonempty `record` referencing the operator decision or preservation receipt. The procedure relies on the trusted operator to establish the completeness of that inventory.

```sh
"$UAR_MIGRATION_RUNNER" --restore-3.3 "$UAR_3_3_SNAPSHOT" "$UAR_3_3_RESTORE" "$UAR_ROLLBACK_PLAN"
```

The command refuses a missing disposition before creating storage. The `restored-copy` receipt identifies the separate destination, the recovery point, every excluded write, and the accepted losses. Start the bound 3.3 artifact on the new copy, confirm the declared baseline through the trusted gateway, then restore service under the recorded disposition. If copying fails after directory creation, quarantine the incomplete copy and use a new path.

The child recovery fixture uses a bounded file size to interrupt a real copy, followed by separate 3.3 recovery. It then exercises an accepted write on the exact 4.0 SDK-backed store and checks refusal before disposition. This offline fixture complements the installed UAR acceptance observations. Use the parity manifest's `baseline.immutableSnapshot` as the synthetic recovery point and retain both runners and the generated receipt. Run only in the approved gate.

```sh
node scripts/behaviors/support/embedded-surrealdb-recovery.mjs "$UAR_EMBEDDED_PARITY_MANIFEST"
```
