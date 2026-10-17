# V2 local05 observed repair — 2026-10-17

## Position and source ownership

The phase remains Execute, plan revision 63. R11 task 8.3 and L9 task 8.4 were completed before the first local05 build. Inventory07 was the explicit full-phase implementation boundary. The lead owns `.dockerignore`, `scripts/release/uar_1_1_1_runtime.py`, the renewed inventory and the next acceptance manifest. The `uar-core` team prepared private rc05 staging and install helpers without running them. No other agent should edit these files or start a build.

## Actual gate result

The first command `docker buildx build --load --platform linux/arm64 --tag uar-phase:1.1.1-rc20261017-05 --file Dockerfile .` exited 0 and produced `sha256:f1bf2fca8ab82b8376cdac9af768022a0adf589a1ea2da6a6483b9ba79bb2221` on `linux/arm64`. Its receipt is `evidence/gates/local-05-build.json`.

The six-group command `python3 scripts/release/uar_1_1_1_acceptance.py --manifest .kbd-orchestrator/phases/uar-outstanding-issues-v1-1-1/evidence/gates/local-05.json` rebuilt the image as `sha256:dde5986112fb9555ad6bcad4ab4c85b3b6ffc8151e56ebed1f291341f9f53616`. Binding's named real-host case passed, then installed-image binding failed with `restart changed durable accounting call/liability`. Streams entered its full byte-partition matrix. The lead sent SIGINT after finding a required binding repair and quadratic trace rereads. The command exited 130 and retained `evidence/acceptance/20261017-local-05/receipt.json` with status `interrupted`; streams and all later groups remain unverified. No local05 result certifies the source now being repaired.

## Observed cause and repair

Python import caches under `scripts/release/__pycache__/` appeared after the manual build. Git ignores them, so the source identity remained stable, but the Docker context did not. `.dockerignore` now excludes `**/__pycache__/` and `**/*.pyc` to stabilize the image input.

The binding fixture compared complete Surreal `/sql` response objects from separate queries. That assertion mixed stored `result` data with response metadata. It now requires a successful one-row query after restart and compares only the stored result. The local05 receipt does not prove whether the data changed; the next run must decide that.

The streams matrix read and JSON-decoded the entire growing provider trace twice for every byte split. It now records the pre-run byte offset and parses only rows appended by that case. It retains all three newline forms, BOM variants, bytewise and every interior split, terminal/event checks, exactly one native request and wire SHA-256 check. Source inspection only; the optimization is unverified until the next gate.

## Next boundary

Renew the complete 22-item implementation inventory with current per-file, source-tree and fixture-tree hashes, and record local05 as the failed/interrupted parent. Create a fresh candidate manifest and image tag. Run one serial production gate through the real image path. The private rc05 guest helpers are unrun and image-specific; do not reuse the rc05 image ID or identity for the next candidate. Fresh same-image native recovery staging remains required before recovery can pass. The 120-minute work checkpoint remains due near 15:11 UTC.
