# Tasks: base-ui-icon-migration

## Task 1 — Establish full icon mapping
- [ ] For each file in the 16-file list, grep for the specific Radix icon names used
- [ ] Confirm each maps to a lucide-react equivalent (use the mapping table in proposal.md)
- [ ] For any Radix icon not in the mapping table, find the closest lucide-react match before proceeding

## Task 2 — Migrate ui/dialog.tsx and ui/sheet.tsx
- [ ] Replace `Cross2Icon` → `X` from `lucide-react`
- [ ] Remove `@radix-ui/react-icons` import line
- [ ] Run `pnpm typecheck` — no errors in these files

## Task 3 — Migrate ui/accordion.tsx
- [ ] Replace `ChevronDownIcon` → `ChevronDown` from `lucide-react`
- [ ] Remove `@radix-ui/react-icons` import line

## Task 4 — Migrate ui/dropdown-menu.tsx, ui/context-menu.tsx, ui/menubar.tsx
- [ ] Replace `ChevronRightIcon`, `CheckIcon`, `DotFilledIcon`, `DotIcon` with lucide-react equivalents
- [ ] For `DotFilledIcon` used as radio indicator: use `Circle` with `fill-current` class or a small filled circle span
- [ ] Remove `@radix-ui/react-icons` import lines

## Task 5 — Migrate ui/navigation-menu.tsx
- [ ] Replace `ChevronDownIcon` → `ChevronDown`
- [ ] Remove `@radix-ui/react-icons` import

## Task 6 — Migrate ui/breadcrumb.tsx
- [ ] Replace `ChevronRightIcon` → `ChevronRight`
- [ ] Remove `@radix-ui/react-icons` import

## Task 7 — Migrate ui/radio-group.tsx
- [ ] Replace `DotFilledIcon` with a filled circle approach using lucide-react `Circle`
- [ ] Verify visual appearance matches original radio indicator

## Task 8 — Migrate ui/checkbox.tsx
- [ ] Replace `CheckIcon` → `Check`
- [ ] Remove `@radix-ui/react-icons` import

## Task 9 — Migrate ui/sidebar.tsx, ui/pagination.tsx, ui/carousel.tsx, ui/resizable.tsx, ui/command.tsx, ui/input-otp.tsx
- [ ] Process each file: identify icons used, replace with lucide-react equivalent, remove import
- [ ] `sidebar.tsx`: likely uses `ChevronRight`, `MoreHorizontal`, `PanelLeft` — verify during migration
- [ ] `carousel.tsx`: likely uses `ChevronLeft`, `ChevronRight` — confirm
- [ ] `resizable.tsx`: likely uses `GripVertical` (already lucide) — confirm no radix-icons usage

## Task 10 — Remove @radix-ui/react-icons from package.json
- [ ] Run: `pnpm remove @radix-ui/react-icons`
- [ ] Verify `pnpm install` succeeds
- [ ] Run: `grep -r "@radix-ui/react-icons" frontend/src/` — must return zero results

## Task 11 — Full TypeScript check
- [ ] Run: `pnpm typecheck` from `frontend/`
- [ ] Zero errors expected at this point (both primitives and icons migrated)

## Completion Gate
All tasks checked. Zero `@radix-ui` imports remain in own code. Commit: `feat: replace @radix-ui/react-icons with lucide-react across all ui components`
