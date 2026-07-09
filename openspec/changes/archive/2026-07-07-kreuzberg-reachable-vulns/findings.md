# Findings: kreuzberg-reachable-vulns

## Task 1.1/1.2 — upstream fix availability (2026-07-07)

Checked every `kreuzberg` tag from the current pin (`v4.9.8`) through the
newest available tag (`v5.0.0-rc.35`, a pre-release) by cloning each
candidate and inspecting its own `Cargo.lock`:

| kreuzberg tag | `lopdf` | `quick-xml` resolutions | Fixes all 3 advisories? |
|---|---|---|---|
| `v4.9.8` (current pin) | 0.40.0 | 0.37.5, 0.39.4, 0.40.1 | No (baseline) |
| `v4.9.9` (latest stable 4.9.x) | 0.41.0 | 0.37.5, 0.39.4, 0.40.1 | No — lopdf still < 0.42.0; quick-xml unchanged |
| `v4.10.0-rc.15` (latest 4.10 pre-release) | 0.40.0 (regressed) | 0.37.5, 0.39.2 | No |
| `v5.0.0-rc.35` (latest overall, major pre-release) | **0.42.0 ✅** | 0.37.5, 0.39.4, 0.40.1 | No — quick-xml still unfixed even here |

**Patched versions required** (per `RUSTSEC-2026-0187`, `-0194`, `-0195` in
the local advisory-db): `lopdf >= 0.42.0`, `quick-xml >= 0.41.0`.

**Conclusion: no upstream kreuzberg tag/commit — stable or pre-release —
resolves all 3 advisories.** Even the newest available tag (`v5.0.0-rc.35`,
an unstable major-version release candidate we should not pin to per D-D)
only fixes `lopdf` and leaves `quick-xml` vulnerable. A clean upstream bump
is **not available**; per plan.md, the fallback is a `[patch.crates-io]`
override in UAR's own `Cargo.toml`.

## Why a straightforward `[patch.crates-io]` override doesn't work either

`cargo tree -i` against UAR's own workspace shows the 3 quick-xml
resolutions kreuzberg reaches come from **3 different manifests we don't
control**, each with a different semver requirement:

- `quick-xml@0.37.5` — required by `biblib v0.4.2` (kreuzberg's dependency)
- `quick-xml@0.39.4` — required by `calamine v0.35.0` (kreuzberg's
  dependency) **and independently** by `opendal`/`reqsign-aws-v4` (via
  `liter-llm`, an unrelated S3/HTTP-signing path, not kreuzberg document
  parsing at all)
- `quick-xml@0.40.1` — required directly by `kreuzberg`'s own `Cargo.toml`
  (`quick-xml = "0.40.1"`, i.e. `^0.40.1`)

`lopdf@0.40.0` is required directly by `kreuzberg` (`lopdf = "0.40.0"`,
i.e. `^0.40.0`).

Because these are all pre-1.0 crates, Cargo treats each minor version as a
separate semver-compatible bucket (`^0.37`, `^0.39`, `^0.40` are mutually
incompatible). A `[patch.crates-io]` entry pointing `quick-xml` at
`>= 0.41.0` would **not** satisfy any of the three callers' `^0.37`/`^0.39`/
`^0.40.1` requirements, so Cargo would refuse to use it for them — this
isn't a "point the pin at a newer version" fix. The only mechanism that
actually works is publishing/vendoring **forked builds of the exact pinned
versions** (`quick-xml` 0.37.5, 0.39.4, and 0.40.1; `lopdf` 0.40.0) with the
upstream security-fix commits cherry-picked in, keeping the same declared
version number so semver resolution still matches. That's 4 separate forks
(1 lopdf + 3 quick-xml, since biblib/calamine/kreuzberg each need their own
patched build at their own pinned version), not the "one heavier but
single move" plan.md anticipated.

## Reachability re-check needed before committing to that effort

Given the fork effort is bigger than planned, the next step (before writing
4 forks) is to check whether the *specific* vulnerable API surface is
actually exercised by kreuzberg/biblib/calamine's own code, per each
advisory's stated affected code path:

- `RUSTSEC-2026-0194`/`-0195` only trigger via `.attributes()` iterated
  **with the default duplicate-check enabled**, or via `NsReader`
  (namespace resolution). A consumer using `.attributes().with_checks(false)`
  and a plain `Reader` (no `NsReader`) is **not affected** by either CVE.
- `RUSTSEC-2026-0187` triggers via `lopdf::Document::load_mem`/`load*` on
  deeply nested PDF arrays/dictionaries (~10,000+ levels) — this one has no
  opt-out; any `.load()` call on untrusted input is reachable if the parser
  itself isn't patched.

This narrows the real question to: does `biblib`/`calamine`/kreuzberg's own
XML glue code call `NsReader` or the checked `.attributes()` path? If not,
the 2 quick-xml DoS CVEs may not be practically exploitable here even
though the vulnerable crate version is present in the tree — which would
be a legitimate, honest "lower urgency than the CVSS score suggests"
outcome, consistent with how this project has treated similar findings
elsewhere in this phase (see change #2's ammonia/rsa/crossbeam-epoch
reachability check).

## Reachability re-check results (source inspection, 2026-07-07)

Inspected the actual cached crate sources
(`~/.cargo/registry/src/.../{biblib-0.4.2,calamine-0.35.0,quick-xml-*}`) and
a clone of `kreuzberg@v4.9.8` for the exact vulnerable API surface:

| Advisory | Vulnerable API | Found in kreuzberg? | Found in biblib? | Found in calamine? | Verdict |
|---|---|---|---|---|---|
| RUSTSEC-2026-0195 (quick-xml, unbounded namespace alloc) | `NsReader` | No | No | No | **NOT REACHABLE** — nothing in the dependency chain uses `NsReader` at all. Disclosed as no-action-needed. |
| RUSTSEC-2026-0194 (quick-xml, quadratic attribute check) | `.attributes()` with default checks (no `.with_checks(false)`) | Yes — 12 files (`docbook.rs`, `extraction/xml.rs`, `jats/*`, `epub/content.rs`, `docx/*`, `fictionbook.rs`) all call `.attributes()`, none call `.with_checks(false)` | Yes — `endnote_xml/parse.rs` ×2 | Yes — `ods.rs`, `xlsx/cells_reader.rs`, `xlsb/mod.rs`, `xlsx/mod.rs` | **REACHABLE** — every XML-based format kreuzberg parses (docx, epub, jats, fictionbook, docbook, xlsx, ods, xlsb, endnote) uses the default (checked) attribute iterator. A single crafted tag with many attributes can pin a CPU core. |
| RUSTSEC-2026-0187 (lopdf, stack overflow via deep nesting) | `lopdf::Document::load_mem`/`load*` | Yes — called directly on user-supplied bytes in `extractors/pdf/mod.rs` (×2), `pdf/images.rs`, `pdf/embedded_files.rs`, `pdf/bookmarks.rs`, with no pre-parse depth/size guard | n/a | n/a | **REACHABLE** — confirms assessment.md's "most concretely reachable finding" framing; a crafted ~21KB PDF can SIGABRT the process. |

**Net result: 2 of the 3 advisories are confirmed reachable (lopdf stack
overflow, quick-xml quadratic attribute DoS); 1 (quick-xml namespace-alloc
DoS) is confirmed not reachable and needs no fix.** Both reachable ones
share a property that matters for remediation: neither can be disabled via
a consumer-side toggle (lopdf has no depth-limit config; kreuzberg's
`.attributes()` calls can't be swapped to `.with_checks(false)` without
patching kreuzberg/biblib/calamine's own source). A resource-bounding
compensating control (subprocess isolation with wall-clock/memory limits,
or a pre-parse file-size/structure sanity check) would mitigate the actual
blast radius of *both* remaining reachable issues in one general mechanism,
without requiring the 4-crate fork-and-patch effort — worth weighing against
the fork effort given it only needs to happen once instead of being
re-applied on every future kreuzberg bump.
