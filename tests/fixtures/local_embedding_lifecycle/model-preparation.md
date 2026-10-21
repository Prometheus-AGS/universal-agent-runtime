# Pinned incompatible local embedding input

The negative fixture uses the official
[`sentence-transformers/all-MiniLM-L6-v2` ONNX export](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/tree/1110a243fdf4706b3f48f1d95db1a4f5529b4d41/onnx)
at revision `1110a243fdf4706b3f48f1d95db1a4f5529b4d41`.
`incompatible-model.lock.json` records immutable URLs, byte sizes, and identities
for the five assets used by the runtime's local FastEmbed path. All identities
come from the public
[revision metadata](https://huggingface.co/api/models/sentence-transformers/all-MiniLM-L6-v2/revision/1110a243fdf4706b3f48f1d95db1a4f5529b4d41?blobs=true).
The ONNX asset has an authoritative LFS SHA-256. The four regular JSON files
have Git blob SHA-1 identities, which include the Git object header; they are
not raw-content SHA-1 or SHA-256 values. The ONNX `upstreamGitBlobSha1` identifies
its LFS pointer, so use its `integrity` SHA-256 to check downloaded weight bytes.

The pinned [model config](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/raw/1110a243fdf4706b3f48f1d95db1a4f5529b4d41/config.json)
declares `hidden_size: 384`, and its
[pooling config](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/raw/1110a243fdf4706b3f48f1d95db1a4f5529b4d41/1_Pooling/config.json)
declares `word_embedding_dimension: 384` with mean pooling. UAR's current
user-defined FastEmbed path uses CLS pooling. This fixture deliberately applies
that runtime path to these real different weights; it makes no claim to reproduce
the canonical Sentence Transformers MiniLM embedding quality. A real local
inference producing 384 values remains a prerequisite for accepting the negative
case. Config metadata alone does not prove ONNX loadability or output shape.

## Deferred asset preparation

**Verification is CLOSED. No preparation, weight download, hash check, model
inference, test, build, or acceptance command below was run while authoring this
pin.** These commands are a proposed later preparation step. The lead must include
them in the whole-phase inventory and obtain the operator's explicit instruction
before executing them. This document grants no exception to the lock.

The primary and compatible backup use the existing declared local BGE assets.
The fixture copies primary assets into its private directory and treats the
configured backup as read-only, so both input variables may identify the
packaged source directory while the live instances still own distinct engines.
Choose a new, nonexistent incompatible directory outside the checkout whose
parent already exists. The following is an exact later command proposal from
the UAR repository, not authorization or evidence of execution:

```text
python3 tests/fixtures/local_embedding_lifecycle/prepare_models.py --primary-dir /Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models --incompatible-dir /Volumes/T7/uar-local-embedding-minilm-1110a243fdf4
```

After that separately approved preparation succeeds, the exact proposed real
local inference and durable lifecycle target is:

```text
UAR_EMBEDDING_LIFECYCLE_PRIMARY_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_BACKUP_DIR=/Volumes/T7/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIR=/Volumes/T7/uar-local-embedding-minilm-1110a243fdf4 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_MODEL=sentence-transformers/all-MiniLM-L6-v2 UAR_EMBEDDING_LIFECYCLE_INCOMPATIBLE_DIMENSION=384 cargo test --locked --no-default-features --features server-full --test embedding_lifecycle_failover -- --test-threads=1
```

The preparation helper is standalone source; it is not imported by the fixture
or executed automatically. It reads the committed pin, compares the primary
weight digest, downloads only public immutable asset URLs and validates each
declared size and SHA-256 or Git-blob identity. It starts no inference and makes
no hosted model request. An occupied output path is an error, not permission to
overwrite it. No model asset is downloaded by the test itself.

This helper downloads public immutable artifact URLs without new credentials or
hosted inference. It installs `onnx/model.onnx` as
`bg-small-en-v1.5.onnx` because that is the runtime's required local filename;
the model identifier remains `sentence-transformers/all-MiniLM-L6-v2`. The
identity and size checks enforce the external artifact boundary and reject an
LFS pointer or an error page masquerading as weights. Exclusive directory and
file creation avoid overwriting existing local model assets. A failed download
or identity check leaves an incomplete directory: do not use it as a fixture.
Choose a new directory for a subsequent explicitly authorized attempt.

The runtime and fixture own asset-manifest generation and real inference.
Preparation does not create a substitute manifest or invent vector outputs.
Keep the prepared directory outside version control; this source fixture stores
only the pin and preparation instructions, with no personal configuration writes.

## Required later evidence

The exact prepared weight SHA-256 must differ from the primary's actual weight
digest, and both runtime members must report their true different model IDs with
the same dimension. The approved production integration gate must load these
assets and run real local inference, then exercise incompatible-query refusal
while preserving the durable KB binding and documents. An ONNX load or output
shape failure is a failed fixture prerequisite, never a passing refusal result.

Metadata was retrieved from the official Hugging Face public API and the two
small config sources above. No ONNX bytes were fetched and no preparation helper,
hash validation, model inference, test, build, or independent acceptance was run.
The pin is complete as source input; runtime compatibility, actual 384-value
output, primary-weight inequality, and lifecycle behavior remain unverified.
