## ADDED Requirements

### Requirement: Tailwind uses the CSS-first Vite integration

The frontend SHALL use exact `tailwindcss` 4.3.3 and `@tailwindcss/vite` 4.3.3 packages, register the Tailwind Vite plugin, and import Tailwind through the shared CSS token source without a JavaScript Tailwind config or PostCSS config.

#### Scenario: Frontend tooling is inspected

- **WHEN** a contributor inspects the frontend build configuration
- **THEN** Tailwind is configured through the Vite plugin and `frontend/src/shared/theme/tokens.css`, and neither deleted legacy config file exists

### Requirement: The design token ladder is CSS-first and stable

The frontend SHALL define the KnowMe-aligned complete-color surface, text, brand, status, run-phase, typography, radius, easing, and duration tokens in `frontend/src/shared/theme/tokens.css` using Tailwind CSS-first theme directives.

#### Scenario: A downstream surface selects design roles

- **WHEN** a downstream UI change needs canvas, chrome, surface, raised, text, brand, status, or run-phase styling
- **THEN** the corresponding stable CSS theme token is available without adding JavaScript Tailwind configuration

### Requirement: Theme and legacy utility behavior is preserved during staging

The CSS-first foundation SHALL preserve dark, light, high-contrast, current semantic utility aliases, explicit A2UI source coverage, and live animation utility behavior while C-05 and C-14a retain ownership of legacy HSL-channel call-site conversion.

#### Scenario: Existing frontend source is compiled during the staged migration

- **WHEN** existing source still uses semantic utilities or legacy HSL-channel variables
- **THEN** C-02 supplies compatible generated utilities and variables without rewriting those deferred call sites

#### Scenario: Reduced-motion user loads the frontend

- **WHEN** the user prefers reduced motion
- **THEN** the token foundation preserves the reduced-motion duration override

### Requirement: Deleted configuration references do not dangle

All live frontend and CI configuration SHALL stop referencing `frontend/tailwind.config.ts` and `frontend/postcss.config.js`; Storybook visual regression SHALL trigger on the shared token source, and the component generator SHALL use the Tailwind 4 empty config-path value.

#### Scenario: Token-only change is proposed

- **WHEN** a pull request or main-branch push changes `frontend/src/shared/theme/tokens.css`
- **THEN** the Storybook visual-regression workflow is eligible to run

#### Scenario: Component generator reads frontend configuration

- **WHEN** the component generator reads `frontend/components.json`
- **THEN** it finds an empty Tailwind config path and the current CSS entry path
