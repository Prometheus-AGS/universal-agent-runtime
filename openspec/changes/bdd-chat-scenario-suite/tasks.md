## 1. Add new step definitions in tests/bdd.rs

- [ ] 1.1 Add a `When` step that sends a chat completion request with an explicit multi-message conversation (prior user + assistant turns followed by a new user message) to the completion endpoint, reusing `ensure_server_booted` and storing status + body in `World`
- [ ] 1.2 Add a `When` step that sends a structurally malformed chat request (JSON body omitting the `messages` field), storing the response status in `World`
- [ ] 1.3 Add a `Then` step asserting the response status is a 4xx client error (and explicitly not a 5xx)

## 2. Author the chat feature file

- [ ] 2.1 Create `tests/features/chat.feature` with `@api` scenarios for: single-turn non-streaming completion, multi-turn conversation, tool-call round trip, streaming deltas + completion, and malformed-request client error — reusing existing steps where the vocabulary already exists

## 3. Verify the suite compiles and passes

- [ ] 3.1 Run `cargo test --test bdd` (SKIP_FRONTEND_BUILD=1) and confirm all chat scenarios pass alongside the existing librefang/AG-UI scenarios
- [ ] 3.2 Run `cargo fmt --check` (or `cargo fmt`) and `cargo clippy --tests` on the touched test code to keep it warning-clean
