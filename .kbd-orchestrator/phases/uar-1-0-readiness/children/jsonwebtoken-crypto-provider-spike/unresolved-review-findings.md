# Unresolved review findings after two rounds

The adversarial-review protocol caps revision at two rounds. Both analyze and decision reviews returned `PASS`; no `CRITICAL` finding remains. The second round still produced warning-level scope/maintenance items, resolved or carried as follows:

- Exactly-one-provider feature unification: resolved in `analysis.md` and `decision.md` with an exact `cargo tree -e features` parent check and a future-update trigger.
- Ambiguous “six changes” and Tier 2 reference: resolved by naming all six OpenSpec changes and quoting the pinned command.
- Version 10 comparison: resolved with `cargo info jsonwebtoken@10.4.0` evidence in `research-evidence.md`; it reports 11.0.0 latest, exposes the same provider choice, and cannot remove the transitive 11.0.0 used by liter-llm.
- Embedded/mobile prior decisions: carried explicitly. The earlier cargo-gating assessment documented already-working Android/iOS routes and warned against speculative broad dependency gating. This spike gives embedded/mobile no verdict; the unconditional manifest feature must be rechecked by any phase certifying those profiles.
- Cross-target evidence: narrowed. Upstream AWS-LC platform support is recorded, but UAR was built only on aarch64 macOS. No cross-target UAR verdict is made.
- Passive monitoring: converted to update ownership for locked dependency/provider/release-target changes. Repository policy forbids adding routine development checks to GitHub Actions, and the child scope forbids creating CI or implementation changes.
- Native build prerequisites: added explicitly—ordinary non-FIPS AWS-LC requires a C/C++ compiler, not CMake/bindgen/Go.
- Algorithm equality and `jsonwebtoken` currency: exact pinned-source and `cargo info` commands are recorded in `research-evidence.md`.

## Plan review

- Round 1 blocked because re-evaluation triggers, exact commands, and receipt paths were not acceptance criteria. All were added.
- Round 2 blocked because the provider-disabled scratch command did not state how the provider was disabled or how to interpret results. The plan now requires it before A0 or in a scratch checkout with `aws_lc_rs` removed, requires the observed missing-provider failure, and requires the identical post-A0 test to pass.
- Round 2 warned that the contract's clippy command does not carry the server-full feature arguments. The command remains `cargo clippy -p universal-agent-runtime` because that is verbatim from the execution contract and the user explicitly prohibited `--all-targets`; the spike cannot rewrite parent tier discipline.
- Round 2 suggested promoting wrong-secret test creation from an aside. It is now an explicit acceptance criterion with error-not-panic semantics.
- The two-round cap is exhausted. These corrections are recorded but not represented as independently re-vetted.
