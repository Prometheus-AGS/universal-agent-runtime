# Decisions — `ship-skill-pack-install-path`

## Iteration 1 — 2026-08-18T17:51:31Z

- **Decision:** continue to independent review before convergence.
- **Iteration:** 1 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** UAR already detects versioned installed plugins; the minimum
  missing product surface is a verified producer for that layout, not another
  loader or runtime fetch path.
- **Uncomfortable result:** the first implementation staged inside the loader's
  scanned two-level root. A concurrent startup could have observed a complete-
  looking staging directory before activation. Staging now occurs as a sibling
  under the cache prefix and is cleaned on every exit.

## Iteration 2 — 2026-08-18T17:59:54Z

- **Decision:** converge after correcting the two critic findings and passing
  independent critic and judge review.
- **Iteration:** 2 of 5.
- **Blocking violations remaining locally:** 0.
- **Rationale:** the runtime scans `$HOME/.config/uar/skills`, so the installer
  now uses that exact default. The pinned pack contains 311 manifests, but UAR's
  established default loader policy deliberately excludes `skills/imported/`;
  the acceptance contract and test now require the exact 147-skill default
  inventory instead of calling the filtered result every manifest.
- **Uncomfortable result:** the initial API assertion compared two views of the
  same filtered discovery result and only required more than 100 rows. It could
  pass while omitting most copied manifests. The corrected assertion pins the
  expected default count before comparing exact API IDs.
- **Independent review:** critic PASS with only the known commit-exclusion
  warning; judge PASS with no blocker.
