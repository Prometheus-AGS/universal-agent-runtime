# Local embedding lifecycle fixture inputs

`embedding_lifecycle_failover` consumes these documents while it drives the
production local embedding implementation. They deliberately contain no model
weights, credentials, endpoints, or personal configuration.

`incompatible-model.lock.json` pins a concrete public MiniLM ONNX export and
tokenizer inputs. `model-preparation.md` describes its exact local preparation
contract. The standalone `prepare_models.py` performs that preparation only
when explicitly invoked after approval; no download, digest check or inference
ran while authoring these files.

The fixture requires three existing local asset directories and the declared
incompatible model identity and dimension:

| Variable | Required space | Purpose |
| --- | --- | --- |
| `UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR` | primary | Initial in-process local inference member. |
| `UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR` | same `{backend, model, dimension}` as primary | Compatible in-process backup member. |
| `UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR` | different model but equal dimension | Query-refusal probe. |
| `UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_MODEL` | true identifier for the incompatible assets | Prevents a relabelled primary model from satisfying the probe. |
| `UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIMENSION` | same numeric dimension as primary | Requires an equal-dimension mismatch, not only a shape mismatch. |

The test never downloads or synthesizes model assets. A missing directory,
missing required model file, unreadable manifest, an incompatible fixture that
does not have the required equal dimension, or an incompatible fixture whose
asset manifest equals the primary manifest is a failed prerequisite, not a
skipped or passing case. Merely renaming BGE assets does not establish a second
model space. The actual `bg-small-en-v1.5.onnx` weight-file digest must also
differ from the primary fixture's digest.

`primary.txt` is ingested before the first backup transition. `backup.txt` is
ingested only while the compatible backup is actively serving. `recovery.txt`
is ingested after the first primary recovery. `second_backup.txt` and
`second_recovery.txt` prove that the retained backup can serve a second
primary-to-backup-to-primary cycle without a runtime restart. `query.txt` must
remain queryable across every generation. `incompatible-query.txt` drives the
refusal attempt and must leave the durable KB binding and all documents
unchanged.

The runtime-owned local failover implementation validates the actual asset
manifests and exposes sanitized member provenance. It must retire the active
primary member through its lifecycle scope and join accepted inference before
the compatible backup is activated. This fixture only supplies non-secret text
inputs and the directory-variable contract.
