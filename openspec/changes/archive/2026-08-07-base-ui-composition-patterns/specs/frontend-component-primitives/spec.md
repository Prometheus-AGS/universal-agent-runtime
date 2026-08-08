## ADDED Requirements

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
