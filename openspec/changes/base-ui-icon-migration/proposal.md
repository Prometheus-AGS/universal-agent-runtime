# Change: Base UI Icon Migration — Replace @radix-ui/react-icons with lucide-react

## Why

After the package swap in `base-ui-foundation`, `@radix-ui/react-icons` remains as the last
Radix dependency in application code (used in 16 files across `src/components/ui/` and
`src/components/`). `lucide-react` is already installed in `package.json` and provides
equivalent icons for every Radix icon used in this project.

## What Changes

Replace all `@radix-ui/react-icons` imports with `lucide-react` equivalents across all 16 affected files.

### Icon Mapping

| Radix Icon | lucide-react Equivalent |
|---|---|
| `Cross2Icon` | `X` |
| `ChevronRightIcon` | `ChevronRight` |
| `ChevronDownIcon` | `ChevronDown` |
| `ChevronLeftIcon` | `ChevronLeft` |
| `DotFilledIcon` | `Circle` (small, filled via className) |
| `CheckIcon` | `Check` |
| `DotsHorizontalIcon` | `MoreHorizontal` |
| `DragHandleDots2Icon` | `GripVertical` |
| `MixerHorizontalIcon` | `SlidersHorizontal` |
| `EnterFullScreenIcon` | `Maximize2` |
| `ExitFullScreenIcon` | `Minimize2` |
| `PlusIcon` | `Plus` |
| `MinusIcon` | `Minus` |

### Files to Migrate

- `frontend/src/components/ui/pagination.tsx`
- `frontend/src/components/ui/resizable.tsx`
- `frontend/src/components/ui/accordion.tsx`
- `frontend/src/components/ui/input-otp.tsx`
- `frontend/src/components/ui/command.tsx`
- `frontend/src/components/ui/navigation-menu.tsx`
- `frontend/src/components/ui/sheet.tsx`
- `frontend/src/components/ui/dialog.tsx`
- `frontend/src/components/ui/carousel.tsx`
- `frontend/src/components/ui/breadcrumb.tsx`
- `frontend/src/components/ui/radio-group.tsx`
- `frontend/src/components/ui/sidebar.tsx`
- `frontend/src/components/ui/dropdown-menu.tsx`
- `frontend/src/components/ui/context-menu.tsx`
- `frontend/src/components/ui/menubar.tsx`
- `frontend/src/components/ui/checkbox.tsx`

## Acceptance Criteria

- [ ] `grep -r "@radix-ui/react-icons" frontend/src/` returns zero results
- [ ] `@radix-ui/react-icons` removed from `frontend/package.json`
- [ ] All previously-using icons now import from `lucide-react`
- [ ] `pnpm typecheck` passes with zero icon-related errors
- [ ] Visual spot-check: dialog close button renders X icon, accordion chevron renders correctly
