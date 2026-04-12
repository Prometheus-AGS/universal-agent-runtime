# Reflect — Typecheck and schema registration

## Checklist

- [ ] `prisma validate` succeeds
- [ ] Every FK in Prisma schema has a matching relation entry **or** intentional omission documented
- [ ] `registerSchema` runs before hooks mount in app entry
- [ ] List/detail responses match `readListResponse` expectations (`items` | `data` | array)
- [ ] Mutations trigger `cascadeInvalidation` (automatic on CRUD success paths in library)

## Manual tests

1. Load list → IDs in graph → open detail without full reload.
2. Update entity in one view → other view updates (reactivity).
3. Change filter in view → new `queryKey` → correct refetch.

## Security spot-check

- Attempt injection via `where` JSON — server must reject unknown fields.

## Next

`persist.md`
