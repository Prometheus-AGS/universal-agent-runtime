# Integrated desktop release sources

The operator approved consuming merged UAR PR #368 and connected fork updates on 2026-10-10.

- Full skill pack: `9c776b291cdf942e90ce240550781c7ca461dcf6`, including historical Cadence publication adoption.
- models.dev: `4a0154da2410bca18e97be812fecb9e8f85466e8`.
- Entity management: `34a211563fe7f06118996a40cc398c4c3af953ef`.
- Rust filesystem: `01ae76b99db4b818458efe2dc5d76e70fa5eadb4`.
- liter-llm: retain `a6047386cc9fae4258b4a8577f6a016022095586`, including the three release repairs ahead of main. The existing provider snapshot derives from its unchanged schemas; a models.dev pointer does not relabel that snapshot.

Operator-owned versions.toml and unrelated pnpm-lock.yaml changes remain untouched. Native server-full packages must record their actual final source commit. Source selection and compatibility advertisement are not installed qualification.
