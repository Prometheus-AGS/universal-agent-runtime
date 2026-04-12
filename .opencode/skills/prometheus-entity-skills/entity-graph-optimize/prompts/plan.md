# Plan — Audit checklist and priorities

## P0 — Correctness / architecture

- [ ] Component files importing `useGraphStore`
- [ ] Components calling `fetch` for entity APIs (should be hooks → engine/store)
- [ ] Missing `registerSchema` for types used in CRUD cascade paths

## P1 — Performance

- [ ] Broad `useStore` selectors returning whole `entities[type]`
- [ ] Unstable `queryKey` arrays (inline objects without memo)
- [ ] `useGQLList` / `useEntityList` mounted in loops without virtualization
- [ ] Realtime `flushInterval` set to 0 in production without justification

## P2 — Hygiene

- [ ] Missing JSDoc on exported app hooks
- [ ] Duplicate descriptor/config definitions
- [ ] Error boundaries swallowing graph errors silently

## P3 — Memory

- [ ] Unbounded `upsertEntity` growth for unbounded feeds
- [ ] Stale list ids pointing to removed entities

## Prioritization

Sort fixes by user-visible impact × risk. Prefer **measure → change → measure**.

## Next

`execute.md`
