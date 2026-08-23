# Light Mode Specification

## Purpose

Define accessible light and dark theme selection, persistence, system preference detection, and complete design-token coverage.

## Requirements

### Requirement: Light/dark theme toggle
The UI SHALL provide a theme toggle allowing users to switch between light and dark modes.

#### Scenario: Toggle to light mode
- **WHEN** the user clicks the theme toggle while in dark mode
- **THEN** the UI switches to light mode with appropriate light color tokens

#### Scenario: Persist preference
- **WHEN** the user selects a theme preference
- **THEN** the preference is saved to localStorage and restored on next visit

#### Scenario: System preference detection
- **WHEN** no user preference is stored and the OS uses light mode
- **THEN** the UI defaults to light mode via `prefers-color-scheme` media query

### Requirement: Light mode has complete color tokens
The light mode color palette SHALL cover all CSS custom properties used by the design system.

#### Scenario: All components render correctly
- **WHEN** light mode is active
- **THEN** all UI components (cards, buttons, inputs, modals, chat bubbles) have legible contrast ratios meeting WCAG 2.1 AA (4.5:1 for text)
