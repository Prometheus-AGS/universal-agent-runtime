## 1. Fake EventSource

- [ ] 1.1 Author a small `FakeEventSource` class with `addEventListener` + `dispatch` helpers.

## 2. Test body

- [ ] 2.1 `vi.stubGlobal("EventSource", FakeEventSource)`.
- [ ] 2.2 Cover `create`, `update`, `delete` event-name mapping.
- [ ] 2.3 Cover unsubscribe — after unsubscribe, dispatch does not invoke handler.
- [ ] 2.4 Cover status callback — `onopen` triggers `connected`.

## 3. Sanity

- [ ] 3.1 Manually break the event-name mapping; confirm the test fails.

## 4. Verification

- [ ] 4.1 `pnpm --filter ./frontend test` green.
