# Published amd64 acceptance follow-up

Operator request: run the full acceptance program against the published amd64 image, then update release receipts and #379. This is authorized release verification after the completed full implementation boundary; no new development phase or production change is planned.

1. Pull the immutable linux/amd64 child manifest from the accepted v1.1.0 index. Verify architecture and source/fixture equality with the tag.
2. Execute the unchanged full program with DOCKER_DEFAULT_PLATFORM=linux/amd64 and the child manifest digest. The host is OrbStack linux/aarch64; this is emulated amd64 execution, not native amd64 hardware qualification. Record the host, container/image architecture and exact command.
3. Retain the complete raw receipt, log, egress receipt and subordinate evidence. Report actual failure if any; never move the immutable tag or replace a failure with a pass.
4. Add uniquely named amd64 assets/checksums to the existing GitHub Release, update its notes and post #379. Preserve the original arm64 assets and their hashes.
5. Commit the follow-up receipts and documentation. Owned files: this plan, phase receipts and acceptance/status records, docs/releases/v1.1.0-kubo.md and append-only .prometheus history. No runtime or fixture edits are planned.

The uncomfortable limit: an amd64 image executed under translation does not prove behavior on native amd64 hardware. Existing startup-capture, infrastructure-projection and review-independence limits continue to apply.

## r1 infrastructure stop and complete r2 correction

r1 exited 1 after safe_default completed. Global DOCKER_DEFAULT_PLATFORM=linux/amd64 caused Docker to attempt a second platform of the pinned SurrealDB index; Docker reported cannot overwrite digest. This is the observed local error; its upstream root cause is not independently established. The failed receipt/log remain untouched.

r2 removes the global platform override. UAR stays pinned to its single amd64 child manifest. A disposable, network-disabled run without the override reported machine x86_64 and runtime ELF machine 62; Docker also reported the expected amd64/arm64 platform warning. Image metadata is linux/amd64. The companion remains its existing pinned arm64 image. Runtime/fixtures are unchanged. No production, fixture or documentation implementation item is pending; the complete full-program rerun is the requested acceptance gate.
