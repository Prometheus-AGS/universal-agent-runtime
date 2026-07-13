## 1. Open letter (consent or remove)
- [x] 1.1 Draft the open-letter template (operator review before send). See `docs/legal/sdk-relicense-open-letter.md`.
- [x] 1.2 Identify all past SDK contributors from git log across `sdks/{python,rust,typescript}/`. Result: only the operator's own identities (Travis James) plus a CI automation account (`Ubuntu <azureuser@...>`) — **no third-party contributors found**. See findings in the open-letter doc.
- [ ] 1.3 Send the open letter; collect consent or removal request. **BLOCKED on operator**: per the 1.2 finding, this may reduce to operator self-authorization rather than outreach — operator to confirm, then this is either a no-op or an explicit send action (send-on-behalf-of-user requires explicit chat permission per policy; not auto-sent).
- [ ] 1.4 For non-responsive contributors past the consent window, mark their contributions for removal in §3. N/A if 1.3 resolves as self-authorization; otherwise pending the 30-day window in the letter.

## 2. License files
- [x] 2.1 Create `sdks/python/LICENSE` (MIT).
- [x] 2.2 Create `sdks/rust/LICENSE-MIT`; keep `LICENSE-AGPL` for the dual-license carve-out.
- [x] 2.3 Create `sdks/typescript/LICENSE` (MIT).
- [x] 2.4 `LICENSE-COMMERCIAL.md` reviewed; left unchanged (no pricing bands added) per operator instruction 2026-07-13 to defer commercial pricing/terms content for now. File still exists with its original generic contact-path text; see task 6.1.
- [x] 2.5 Add `LICENSE-CC-BY-4.0.md` for documentation (or inline the CC-BY notice in `docs/`).

## 3. Manifest updates
- [x] 3.1 `sdks/python/pyproject.toml` → `license = {text = "MIT"}`. Already set.
- [x] 3.2 `sdks/rust/Cargo.toml` → `license = "MIT OR AGPL-3.0"` (consumer chooses); `description` notes the dual-license.
- [x] 3.3 `sdks/typescript/package.json` → `"license": "MIT"`. Already set.
- [x] 3.4 Root `Cargo.toml` stays `license = "AGPL-3.0-only"`; add a comment that the SDKs are MIT-licensed.

## 4. CONTRIBUTING + README
- [x] 4.1 `CONTRIBUTING.md` — add the CLA-lite forward-going clause.
- [x] 4.2 `README.md` — update the license section; add the SDK/runtime license split.
- [x] 4.3 `docs/product-support-matrix.json` — update `license` per bundle; added a `sdks[]` block with per-SDK license.

## 5. CI guard
- [x] 5.1 `tools/license-check.sh` — assert every `Cargo.toml`, `pyproject.toml`, `package.json` has a `license` field; assert the file matches.
- [x] 5.2 Add the guard to `.github/workflows/ci.yml` as a new step (`Check license declarations`, in the `Check & Lint` job).

## 6. Operator authorization
- [ ] 6.1 Operator publishes the `LICENSE-COMMERCIAL.md` with public pricing bands. **DEFERRED** (operator instruction 2026-07-13: remove/omit commercial pricing info for now). No pricing bands exist in the file; this task is not blocking Change 1's close and can be picked up as a future change when the operator wants to publish pricing.
- [ ] 6.2 Operator sends the open letter to SDK contributors. **Likely N/A** per the 1.2 finding — operator to confirm no third-party contributors exist, at which point this task closes without a send.
- [ ] 6.3 Operator reviews and merges the PR (no auto-merge on a license-flip change). **BLOCKED on operator.**
