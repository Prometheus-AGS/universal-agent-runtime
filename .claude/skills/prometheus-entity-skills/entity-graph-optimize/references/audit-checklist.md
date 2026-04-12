# Complete audit checklist — entity graph projects

## Architecture (CLAUDE.md)

- [ ] Components use only hooks for entity data
- [ ] Hooks do not call external APIs directly (REST/GraphQL) except through library client/engine modules
- [ ] No duplicate entity copies in parallel React state

## Graph integrity

- [ ] Lists store **ids**, not row snapshots, in list state (library default — verify no overrides)
- [ ] `normalize` / descriptors produce stable `id` fields
- [ ] Patches used only for UI augmentation (`_selected`, etc.)

## Engine

- [ ] Subscriber cleanup on unmount for custom hooks
- [ ] `staleTime` appropriate to data freshness needs
- [ ] Focus/reconnect revalidation acceptable for UX

## CRUD

- [ ] `registerSchema` complete for FK relationships
- [ ] Mutations call success paths that trigger `cascadeInvalidation` (library handles when using `useEntityCRUD` correctly)

## GraphQL (if used)

- [ ] Descriptors cover all entity types returned
- [ ] `cacheKey` uniqueness
- [ ] Subscription error handling reconnect

## Prisma / REST (if used)

- [ ] Server validates `where` JSON
- [ ] Response shapes match `readListResponse`

## Realtime

- [ ] `RealtimeManager` singleton lifecycle matches app (one instance)
- [ ] Adapter `normalize` functions cheap (no heavy sync work per message)

## Security

- [ ] No secrets in client bundle
- [ ] Auth headers not logged

## Tooling

- [ ] `pnpm run typecheck` passes
- [ ] Lint rules for import boundaries (optional but recommended)
