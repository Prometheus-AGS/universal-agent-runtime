# Behavioral controls — `emit-rag-retrieval-events`

This change introduces no fail-closed authentication or authorization guard.
The following contrast controls were nevertheless observed for the two places
where a false positive would overstate retrieval behavior.

## No retrieval hit, no citation event

Command:

```bash
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::rag::citation_stream::tests::empty_matches_produce_empty_stream -- --exact --test-threads=1
```

Observed output, exit 0:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 605 filtered out; finished in 0.00s
```

The assertion requires both an empty marker set and no normalized event.

## Verification guard excludes an unrelated chunk when enabled

Command:

```bash
cargo test --quiet --locked -p universal-agent-runtime --no-default-features --features server-full --lib uar::rag::pipeline::tests::drop_uncorroborated_filters_when_enabled -- --exact --test-threads=1
```

Observed output, exit 0:

```text
running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 605 filtered out; finished in 0.00s
```

The backend returns one related and one unrelated match. The assertion requires
only the related chunk to survive when `drop_uncorroborated` is enabled.
