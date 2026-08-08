## ADDED Requirements

### Requirement: Navigation has one typed destination inventory
The frontend SHALL declare shell destinations once and SHALL derive rail groups, compact tabs, Configure-sheet rows, breadcrumbs, palette commands, and active-route state from that inventory. Static route matching MUST resolve specialized work destinations before the broader Configure route family.

#### Scenario: Work destinations share one definition
- **WHEN** the shell renders desktop navigation, compact navigation, breadcrumbs, or command results
- **THEN** Chat, Knowledge, Agents, and Runs MUST use labels, icons, paths, and route identity from the shared destination inventory

#### Scenario: Admin-backed work route is active
- **WHEN** the current path is `/admin/knowledge`, `/admin/agents`, or `/admin/runs`
- **THEN** the corresponding work destination MUST be active
- **AND** the generic Configure destination MUST NOT claim that route

#### Scenario: Configure route is active
- **WHEN** the current path is a configured admin surface that is not projected as a work destination
- **THEN** the matching Configure item and compact Configure tab MUST identify the current location

### Requirement: One responsive shell re-stacks at the binding breakpoint
The application SHALL mount feature content once inside a single shell composition. Above 900 CSS pixels it SHALL expose a left rail that is 240px expanded or 60px collapsed. At or below 900 CSS pixels it SHALL replace the rail with a compact top bar and four bottom targets for Chat, Knowledge, Agents, and Configure.

#### Scenario: Expanded desktop rail
- **WHEN** the desktop rail is expanded
- **THEN** it MUST occupy 240px
- **AND** it MUST show the delivered wordmark without also showing the app icon

#### Scenario: Collapsed desktop rail
- **WHEN** the operator collapses the desktop rail
- **THEN** it MUST occupy 60px
- **AND** it MUST show the delivered app icon without the wordmark
- **AND** every icon-only destination MUST retain an accessible label

#### Scenario: Compact navigation
- **WHEN** the viewport is at or below 900 CSS pixels
- **THEN** the desktop rail MUST be non-rendered for layout and focus
- **AND** the top bar MUST show the 22px UAR mark
- **AND** each bottom target MUST provide at least a 44px interaction target

#### Scenario: Compact navigation retains complete route access
- **WHEN** a compact user needs a destination that is not one of the four bottom targets
- **THEN** the persistent header command trigger MUST remain available
- **AND** the shared command inventory MUST include Runs and About

### Requirement: Configure uses the shared compact sheet host
The compact Configure target SHALL select an id-based shell sheet rather than render desktop Configure links in the bottom bar. The shared host SHALL present the Configure destinations in a modal Base UI surface and SHALL close after route selection.

#### Scenario: Configure tab opens the hub
- **WHEN** a compact user activates Configure
- **THEN** the shared mobile sheet host MUST open a labeled Configure hub
- **AND** the hub MUST list Providers, MCP & tools, Skills, A2UI, and Runtime settings

#### Scenario: Configure route selection
- **WHEN** a user chooses a Configure item from the hub
- **THEN** React Router MUST navigate to its static application route
- **AND** the sheet MUST close

### Requirement: Route context is accessible before the working surface
The shell SHALL provide a keyboard skip link, one main-content landmark with a stable target, and a breadcrumb header derived from the current destination. The header SHALL expose command access, theme access, and persistent readiness text without using color as the sole status signal.

#### Scenario: Keyboard user skips shell navigation
- **WHEN** keyboard focus enters the application shell
- **THEN** a skip-to-content link MUST become visible on focus
- **AND** activating it MUST target the shell's main content landmark

#### Scenario: Breadcrumb reflects the route
- **WHEN** navigation changes between a work route and a Configure route
- **THEN** the breadcrumb MUST update from the shared destination inventory
- **AND** the current page MUST be exposed as the breadcrumb page

#### Scenario: Runtime health is unavailable
- **WHEN** no health response is available
- **THEN** the persistent readiness presentation MUST include an `Unreachable` text label
- **AND** it MUST NOT communicate failure only through its status color

### Requirement: The app command palette uses Base UI
The new shell command palette SHALL use the installed Base UI `Dialog` and `Autocomplete` primitives, SHALL filter the typed static destination inventory, SHALL auto-highlight keyboard results, and SHALL close after selection. It MUST NOT import `cmdk` or the legacy command wrapper.

#### Scenario: Global keyboard command opens palette
- **WHEN** the user presses Control+K or Meta+K outside an editable control
- **THEN** the command palette MUST open
- **AND** focus MUST move to an input with an accessible `Search commands` name

#### Scenario: Command navigates
- **WHEN** the user chooses a destination command
- **THEN** React Router MUST navigate to that destination using normal history
- **AND** the command palette MUST close

#### Scenario: Query has no match
- **WHEN** the command query matches no destination
- **THEN** the palette MUST expose a readable `No commands found` empty state

### Requirement: Delivered UAR brand assets own the shell identity
The frontend SHALL ship the delivered `docs/ui/logo/` assets under `frontend/public/brand/`, excluding operating-system metadata. React shell surfaces SHALL use a shared Slash Gate logo component instead of the retired KnowMe mark. The document head SHALL select delivered light and dark favicons using color-scheme media queries.

#### Scenario: Shell brand projection
- **WHEN** the shell renders in expanded, collapsed, or compact presentation
- **THEN** it MUST use the delivered UAR wordmark, app icon, or mark appropriate to that presentation
- **AND** it MUST NOT render the retired KnowMe identity

#### Scenario: Browser selects favicon by theme
- **WHEN** the browser prefers a light or dark color scheme
- **THEN** the matching delivered favicon MUST be available from `/brand/`

### Requirement: Shell presentation state follows the frontend layering contract
Rail collapse, command-palette visibility, and the active mobile sheet SHALL be transient Zustand state exposed through the existing shell hook. Shell components MUST call the hook and MUST NOT import Zustand stores, services, or transport modules directly. The shell MUST NOT change provider, AG-UI, persistence, or feature-store contracts.

#### Scenario: Sibling shell controls share state
- **WHEN** the header trigger, keyboard shortcut, compact tab, or sheet host changes shell presentation
- **THEN** sibling shell components MUST observe the same hook-projected state
- **AND** the store MUST contain only serializable ids and booleans rather than React nodes

#### Scenario: Existing feature route renders after migration
- **WHEN** a user navigates to `/threads`, `/admin/*`, or `/about`
- **THEN** the existing feature page MUST remain mounted in the shell working surface
- **AND** no provider payload, event schema, entity record, or service call contract MUST change

### Requirement: Shell interactions preserve Flat 2.0 and motion accessibility
New shell surfaces SHALL separate hierarchy with the existing surface ladder and spacing rather than visible borders, separator rules, gradients, backdrop blur, or shadows. Interactive state transitions SHALL use 200–320ms opacity/background or 20px translation and SHALL become effectively immediate under reduced-motion preference. Focus indicators SHALL use the permitted 3px ember treatment.

#### Scenario: Reduced motion is requested
- **WHEN** the user enables `prefers-reduced-motion: reduce`
- **THEN** rail, sheet, and palette transitions MUST not require animated movement to understand state change

#### Scenario: Keyboard focus enters a shell control
- **WHEN** a shell link, button, command item, or sheet control receives visible keyboard focus
- **THEN** it MUST expose the 3px ember focus treatment without adding a persistent component border
