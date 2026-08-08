## ADDED Requirements

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
