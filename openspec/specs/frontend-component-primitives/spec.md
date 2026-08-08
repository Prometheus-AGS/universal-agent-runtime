# frontend-component-primitives Specification

## Purpose
TBD - created by archiving change base-ui-foundation. Update Purpose after archive.
## Requirements
### Requirement: Production primitives use the Base UI foundation
UAR's interactive production primitive wrappers SHALL use `@base-ui/react` as their
headless implementation foundation and SHALL expose those primitives through the local
`frontend/src/components/ui/` wrapper boundary.

#### Scenario: A product feature consumes an interactive primitive
- **WHEN** feature or page code needs an interactive primitive
- **THEN** it imports the local UI wrapper rather than importing a headless primitive
  package directly

#### Scenario: A production wrapper needs headless interaction behavior
- **WHEN** a wrapper under `frontend/src/components/ui/` implements headless interaction
  behavior
- **THEN** it uses the corresponding Base UI primitive and does not directly import a
  Radix UI primitive

### Requirement: Generator metadata preserves the selected primitive family
The frontend component generator configuration SHALL select the shadcn `base-vega` style
and the `neutral` base color so regeneration remains aligned with the Base UI foundation.

#### Scenario: UI wrappers are regenerated
- **WHEN** the shadcn generator reads `frontend/components.json`
- **THEN** it resolves the `base-vega` style and `neutral` base color

### Requirement: Primitive migration remains staged behind stable wrappers
UAR SHALL keep application-facing primitive imports stable while composition, icon, and
dependency-pruning follow-up changes are completed.

#### Scenario: A follow-up migration changes primitive internals
- **WHEN** composition, icon, or dependency cleanup changes the primitive implementation
- **THEN** existing feature and page imports continue resolving through the local UI
  wrapper boundary

### Requirement: Application composition uses render elements
UAR application-owned React source SHALL compose local buttons, links, and
third-party action primitives through their supported `render` element APIs and
SHALL NOT use `asChild`, Radix Slot imports, or direct `radix-ui` imports.

#### Scenario: An action primitive renders a local button
- **WHEN** an assistant action is displayed through the local Button or TooltipIconButton wrapper
- **THEN** the action primitive merges its behavior onto that render element without adding a wrapper element

#### Scenario: Application source is audited for legacy composition
- **WHEN** the composition source gate scans `frontend/src`
- **THEN** it finds no `asChild`, `@radix-ui/react-slot`, or direct `radix-ui` import syntax

### Requirement: Form wrappers delegate field semantics to Base UI
The stable local React Hook Form facade SHALL implement its item, label, control,
description, and message parts with Base UI Field primitives while preserving the
existing exported component names.

#### Scenario: A controlled field renders
- **WHEN** a consumer renders the local FormField, FormItem, FormLabel, FormControl, FormDescription, and FormMessage facade
- **THEN** the label is associated with the composed control and the description is exposed to that control through Base UI Field semantics

#### Scenario: External validation reports an error
- **WHEN** React Hook Form marks the controlled field invalid with an error message
- **THEN** Field.Root receives the external field state and Field.Error displays the message for the control

#### Scenario: A valid form is submitted
- **WHEN** the composed control contains a valid value and the form is submitted
- **THEN** React Hook Form receives the value through the unchanged Controller wiring

### Requirement: Regenerated Base UI wrappers remain stable
Button, Breadcrumb, Sidebar, and Select wrappers SHALL retain their existing Base
UI-native implementations and application-facing exports throughout the
composition migration.

#### Scenario: Existing wrapper composition is re-audited
- **WHEN** the migration verifies the regenerated local wrappers
- **THEN** their Base UI primitive, `useRender`, or Base UI `render` composition remains in place without an unnecessary regeneration

### Requirement: Application primitives use the Lucide icon family
UAR application-owned React primitives SHALL use `lucide-react` for generic
interface icons and SHALL NOT directly import `@radix-ui/react-icons`.

#### Scenario: A primitive renders a generic interface icon
- **WHEN** a local wrapper displays a close, chevron, check, menu, navigation, resize, search, plus, or minus icon
- **THEN** the icon is provided by `lucide-react` with the wrapper's accessible label and interaction semantics unchanged

#### Scenario: Application icon source is audited
- **WHEN** the icon-family source gate scans TypeScript and TSX under `frontend/src`
- **THEN** it finds no `@radix-ui/react-icons` or direct `radix-ui` import

### Requirement: The frontend dependency graph excludes unused Radix icons
The frontend manifest and the maintained root and frontend lockfiles SHALL NOT declare
`@radix-ui/react-icons` after all application-owned consumers have migrated.

#### Scenario: Dependencies are installed from the lockfile
- **WHEN** pnpm resolves the root and frontend workspaces with frozen lockfiles
- **THEN** both installs succeed without a direct `@radix-ui/react-icons` dependency

### Requirement: Product artwork remains distinct from interface icons
UAR SHALL preserve custom product, provider, and brand SVG artwork when no
generic interface-icon substitution is intended.

#### Scenario: The icon-family gate encounters custom artwork
- **WHEN** application source renders a product logo or provider mark as an inline SVG
- **THEN** that artwork remains allowed and is not replaced solely to satisfy the generic interface-icon family

### Requirement: Application command search is Base UI owned
UAR SHALL implement the stable local `Command*` facade with Base UI primitives and SHALL NOT retain `cmdk` or another Radix-backed application command implementation.

#### Scenario: A feature filters an action list
- **WHEN** an operator types in an agent, model, skill, tool, or knowledge-base command search
- **THEN** matching items remain keyboard and pointer activatable through the unchanged local wrapper API

#### Scenario: A repeated-add command remains open
- **WHEN** an operator selects an item from a command search whose host remains open
- **THEN** the search remains an action filter rather than persisting the selected item as a form value

### Requirement: Third-party primitive ownership is auditable
UAR SHALL document Radix packages retained through supported third-party dependencies and SHALL distinguish them from application-owned source imports and direct dependency declarations.

#### Scenario: The dependency graph is audited
- **WHEN** pnpm explains a retained Radix package
- **THEN** the receipt identifies its supported third-party owner and application source remains free of direct Radix imports

#### Scenario: Entity-management ownership is audited
- **WHEN** the Prometheus Entity Management package metadata and Radix graph are inspected
- **THEN** the receipt records whether that package introduces a Radix dependency without inferring ownership from unrelated packages

