# Validation report: document-and-deploy-native-services

Observed at `2026-08-23T17:05:34Z`.

## Final gate

| Check | Result | Observed evidence |
|---|---|---|
| Schema | PASS | Strict OpenSpec validation and JSON, Node, Python, and PowerShell parsing passed. |
| File integrity | PASS | Both catalog submodules are clean; the manifest output exists and is non-empty. |
| Constraints | PASS | Catalog provenance, pricing conversion, exact-match config migration, secret safety, and platform limits passed. |
| Consistency | PASS | Gitlinks, snapshot metadata, README count, OpenSpec verification, and retained evidence agree. |

No blocking findings remain.

## Corrective loop

The first history-blind validation failed for three concrete reasons:

1. The Unix provider parser ignored its unquoted-ID capture group and could add
   a duplicate Alibaba provider.
2. The README reported 269 providers while the generated catalog had 316.
3. Current `models.dev` HEAD contains two Eden AI filenames that differ only by
   case and therefore cannot produce a clean checkout on the supported default
   macOS filesystem.

The final artifact recognizes unquoted provider IDs, reports 316 providers, and
pins `models.dev` at clean Qwen-containing revision `f97df19af`. The actual
offline catalog source is `liter-llm` revision `788877f7`, version 1.18.1.

## Deterministic observations

- Two catalog refreshes produced SHA-256
  `c4704316b380e40c9b2d093eb4c1704a2574d4a13ecc0d5b5d1943bc5ded1bb6`.
- Alibaba `qwen3.8-max` maps to 1,000,000 context, 131,072 output,
  $2/$6 per-million input/output pricing, and the source capability flags.
- The runtime source diff contains no Qwen catalog overlay.
- An unquoted, operator-owned Alibaba fixture remained byte-identical, retained
  its custom model, and contained one Alibaba provider after bootstrap.
- Eight loaded credential values were checked; zero occurrences were found in
  the intended artifacts and retained evidence.
- The installed LaunchAgent remained healthy and ready. No inference request
  was made during this QA pass.

## Scope limits

This report covers the `server-full` release build and observed macOS
LaunchAgent deployment. Linux systemd and Windows SCM results remain template,
parser, and compile evidence only. It makes no aggregate or cross-profile
readiness claim.
