# Local embedding resource metadata

Phase: user-outcome implementation. Gate CLOSED. Only this document changed.
Read `prometheus-rust-workspace` first; used source inspection and filesystem
metadata only. No Rust/model constructor, import, inference, tokenizer parser,
probe, hash verification, download, build, test or service ran.

## Concrete existing closure root

Select this actual checkout directory as the `local_embedding_assets` tree:

`/Users/gqadonis/Projects/prometheus/universal-agent-runtime/src/uar/runtime/matching/models`

Observed regular, nonsymlink, mode644 file metadata:

| File | Bytes |
|---|---:|
| bg-small-en-v1.5.onnx | 34014426 |
| tokenizer.json | 711396 |
| config.json | 650 |
| special_tokens_map.json | 125 |
| tokenizer_config.json | 366 |
| MODEL-NOTICE.txt | 331 |

These are concrete existing bytes, not an empty tree or a proposed future
cache. File existence/size does not certify ONNX validity or successful inference.
The five runtime input files plus notice total34727294bytes. The producer's
approved existing-tree path can be the exact directory above, with actual
`tree_sha256` assembled during explicitly approved P0 metadata capture; no
current digest was invented or calculated in this task. Preserve the notice in
the same immutable snapshot tree. The closure group selects its tree ID and
thus contains all six actual files, not a bare ONNX path.

`catalog/SNAPSHOT.md` already records the packaged input identities below.
These are read historical source declarations, NOT freshly verified byte hashes:

| File | Declared SHA256 |
|---|---|
| bg-small-en-v1.5.onnx | 6c9c6101a956d62dfb5e7190c538226c0c5bb9cb27b651234b6df063ee7dbfe4 |
| config.json | f2c87ea17fdfa286f48d6e1506f2cdc838d303420dc55606190e67e5acd9e186 |
| special_tokens_map.json | b6d346be366a7d1d48332dbc9fdf3bf8960b5d879522b7799ddba59e76237ee3 |
| tokenizer.json | d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66 |
| tokenizer_config.json | 9261e7d79b44c8195c1cada2b453e55b00aeb81e907a6664974b4d7776172ab3 |

## Actual factory and configuration behavior

`src/uar/rag/embeddings/fastembed.rs` defines MODEL_FILES with exactly those five
runtime filenames. `FastEmbedBackend::new_with_scope` resolves the asset directory
and captures per-file and combined identities. Lazy initialization re-reads and
compares those bytes before `TextEmbedding::try_new_from_user_defined`, using
TokenizerFiles from the four JSON assets, CLS pooling and max length512.
No hf-hub/cache download occurs in that runtime factory.

The strict failover member path requires all five files in its explicitly
configured `models_dir` and does not consult environment/fallbacks. Legacy
nonstrict resolution tries `UAR_MODELS_DIR`, configured models_dir, `/app/models`,
then checkout-relative matching/models candidates. It reports the packaged
bge-small-en-v1.5 identity for legacy callers. Global config defaults backend
fastembed, model bge-small-en-v1.5, dimension384 and this checkout-relative models
directory. `build_local_backend` accepts only fastembed and requires local-models;
server-full includes local-models.

For the frozen fixture, set the actual local `models_dir` and any explicitly
approved UAR_MODELS_DIR reference to the copied snapshot directory, e.g. the
producer's `<artifacts>/snapshot/<selected-tree-id>/payload`. Do not rely on the
server's cwd or an ambient cache to select it. Strict primary/backup member
configuration, if exercised, must name its actual member directories/model
identities; a second copy/label is not evidence of a different model. The manager
pins the configured backend for matching; a global hosted backend does not turn
these local bytes into a local call automatically. The applicable case config
must explicitly select the intended local backend/algorithm.

Cargo.lock selects fastembed5.17.4; Cargo.toml pins ort2.0.0-rc.13. Its nearby prose
about fastembed major4 is stale; the actual declaration selects major5. No
manifest/source repair was made. ort-download-binaries is a build-time resource
fetch and belongs to the explicit native build exception/closure, not this model
asset download. Native ORT libraries belong to the native runtime closure and
are not replaced by an ONNX file.

## Existing user cache is separate and unnecessary for this factory

Metadata-only directory inspection found:
`/Users/gqadonis/.cache/huggingface/hub/models--BAAI--bge-small-en-v1.5`.
Its snapshots directory contains named revision
`5c38ec7c405ec4b44b94cc5a9bb96e735b38267a`, with symlinked config.json,
model.safetensors and tokenizer.json pointing into its blobs directory.
The cache also has refs named main and that revision; ref contents/model bytes
were not loaded. This is a BAAI safetensors cache, not the required five-file
packaged ONNX root. Its revision name is not asserted as the checkout ONNX
revision or current locked resource provenance. Do not include it by accident
or claim the FastEmbed factory uses it.

Metadata-only checks found no directories at ~/.cache/fastembed,
~/Library/Caches/fastembed, repository/.fastembed_cache or /app/models.
Their absence is irrelevant because the required packaged assets are present.
No profile, credential, entire cache dump or secret file was read.

## Source provenance and P0 disposition

The model notice names BAAI/bge-small-en-v1.5 and MIT terms:
https://huggingface.co/BAAI/bge-small-en-v1.5 . The more specific checked-in
refresh script and catalog snapshot identify Xenova's quantized ONNX variant:
https://huggingface.co/Xenova/bge-small-en-v1.5 . `scripts/refresh-local-model.sh`
uses resolve/main/onnx/model_quantized.onnx renamed to bg-small-en-v1.5.onnx and
resolve/main for the four JSON files. These URLs were read from repository
source, not newly fetched or used as current download authority. No immutable
upstream Xenova commit is recorded in those inspected provenance declarations.
The adopted checkout/source hashes, not a guessed cache revision or mutable
main, must bind the selected existing bytes for this fixture.

No asset download stage is needed for this local_embedding_assets closure:
all five exact factory inputs are present in the named checkout root. Therefore
P0 should snapshot/adopt those bytes and their notice, not invoke the refresh
script (which also requests tests and uses mutable main). If later approved
metadata finds a missing/mismatching file, stop P0; do not silently download or
substitute the BAAI safetensors cache. A replacement download would require a
separate immutable upstream revision, exact selected file digests and explicit
bounded download argv, none invented here because the absent-assets condition
was not observed.

The uncomfortable limitation is that existing ONNX metadata and declared source
hashes do not prove useful inference. They establish a fully authorable actual
resource tree, while real local matching behavior remains unverified until the
separately authorized genuine packet. Root owns integration of this tree into
actual artifact config, immutable source/closure capture and later V approval.
