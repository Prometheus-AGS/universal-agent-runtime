# Execute evidence: harness-model-profiles-routing 2.2

Task 2.2 adds exact, versioned endpoint request profiles at the Liter driver
boundary. Each profile binds the provider, endpoint fingerprint, qualified and
wire model names, model and settings revisions, request transform, complete
allowed final-field set, and optional output ceiling.

The supported synthetic OpenAI-compatible fixture proves that the complete
post-merge request sent over HTTP equals the validated request. Its captured
bytes match the redacted prepared-request SHA-256, serialized length, and exact
token count. Unsupported settings, endpoint or model rebinding, missing or
mismatched budget contracts, reserved-field collisions, Anthropic ceiling
expansion, and checked-add overflow all reject before network I/O.

## Verification

- Tier 0: `cargo check` passed with `server-full`.
- Tier 1: `model_path_resiliency` passed 18/18.
- Tier 1 regression: `prompt_assembly` passed 12/12.
- `cargo fmt --all -- --check` passed.
- Targeted `git diff --check` passed.
- Strict OpenSpec validation passed.
- Final isolated artifact review returned PASS with no findings after two
  blocking rounds were corrected.

An intermediate test compile exposed E0382 because the HTTP fixture moved
`base_url`. Cloning that test value corrected the failure before the recorded
green runs.

## Claim boundary

The production provider matrix still certifies zero dispatch profiles. This task
certifies only the deterministic synthetic OpenAI-compatible path whose
qualified and wire model names are identical and whose endpoint is explicitly
bound. Anthropic profiled dispatch remains unsupported until a complete final
wire counting contract exists. Production profile registration and canonical
entry-path wiring remain task 2.3. No live provider, dependency, secret, service,
database, deployment, commit, or push changed.

`ProfiledWireReceipt` is the only product surface added beyond the planned
profile type and validation path. It directly closes the observed requirement to
prove final-wire equivalence while withholding request content. Every guard maps
to an explicit exact-profile requirement or an isolated-review finding.
