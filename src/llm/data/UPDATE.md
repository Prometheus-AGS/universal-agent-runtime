# Updating `model_benchmarks.json`

This dataset is **hand-curated, not scraped**. Automating extraction from
public leaderboards was judged too risky to ship unattended: leaderboard
sites change their DOM/table structure often, model-name-to-catalog-id
mapping requires judgment calls (see the dropped `DeepSeek-V4-Pro-Max` /
`Mistral Medium 3.5` cases in `model_benchmarks.json`'s `_readme`), and a
silently-wrong automated scrape would be worse than a stale-but-correct
manual one.

## Process

1. Pick a benchmark + dimension (`coding` → SWE-bench Verified today;
   `agentic` and `context` are defined in `src/llm/benchmarks.rs` but have no
   populated source yet — see "Open dimensions" below).
2. Fetch the leaderboard directly (e.g. `https://llm-stats.com/benchmarks/swe-bench-verified`,
   `https://www.swebench.com/`) — don't trust a search-engine AI summary of
   the page, fetch and read the actual table. Search summaries have been
   observed to include suspicious/unverifiable claims (e.g. a "model
   suspended under export-control directive" claim that didn't check out).
3. For each model you want to add/refresh, confirm the **exact** catalog id
   it corresponds to by checking a real generated `provider_catalog.json`
   (built by `build.rs` into `$OUT_DIR`, or run
   `cargo build --lib` once and find it under
   `target/debug/build/universal-agent-runtime-*/out/provider_catalog.json`
   — pick the most recently modified one). Match on the provider's `id` field
   and the model's exact `id` string (case-sensitive — e.g. MiniMax models
   are `MiniMax-M3`, not `minimax-m3`).
4. If the leaderboard's model name doesn't unambiguously map to one catalog
   id (different naming, ambiguous suffix like "-Max"/"-Pro"), **drop it**
   rather than guess. A missing entry just means the router tiebreaker has
   no signal for that model (falls through to the next tiebreak); a wrong
   entry silently misattributes another model's score.
5. Add/update the entry with `benchmark`, `dimension`, `score`, `source_url`,
   `retrieved_date` (today, `YYYY-MM-DD`). Keep entries in provider-grouped
   order for readability (not required, just convention).
6. No rebuild step is required — `src/llm/benchmarks.rs` embeds this file via
   `include_str!` directly (not through `build.rs`/`OUT_DIR`), so editing it
   takes effect on the next normal compile.

## Open dimensions

- `agentic`: no benchmark chosen yet. Candidates from docs/uar-next-fable.md
  competitive-analysis section: GAIA, Terminal-Bench. Needs the same
  fetch-and-verify treatment as `coding` above before adding entries.
- `context`: no benchmark chosen yet. Candidates: MRCR, RULER — but
  docs/uar-next-fable.md §2.3 flags "effective vs advertised context" as one
  of the internally-contradictory dimensions in its source document, so be
  extra careful to cite a single primary benchmark run, not a secondary
  aggregator's summary of one.
