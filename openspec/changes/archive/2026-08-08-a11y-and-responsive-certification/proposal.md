## Why

The UI migration cannot close while accessibility checks are suppressed for known token
contrast failures, the global focus indicator remains 2px instead of the binding 3px ember
contract, and the responsive result has not been certified across the required four widths
and two themes. This final change turns those open claims into repeatable WCAG 2.2 AA,
keyboard, reduced-motion, and responsive evidence.

## What Changes

- Add a deterministic Playwright certification suite for representative application
  surfaces at 320, 768, 1024, and 1440 CSS pixels in light and dark themes.
- Run axe plus explicit checks for landmarks, accessible names, keyboard operation,
  focus visibility, target size, live regions, non-colour status cues, reduced motion,
  and horizontal overflow.
- Remove Storybook accessibility suppressions by correcting the reproduced theme-token
  contrast failures in both themes.
- Establish the 3px ember focus indicator as the application-wide default and remove
  conflicting feature-level 1–2px overrides exposed by certification.
- Record the KnowMe §12 acceptance review, including honest classifications for checks
  that require a real backend or a separate Flutter implementation.

## Capabilities

### New Capabilities

- `frontend-accessibility-certification`: Defines the repeatable WCAG 2.2 AA, keyboard,
  reduced-motion, status, focus, and responsive certification contract for the React UI.

### Modified Capabilities

None.

## Impact

- Affects frontend theme tokens and focus styles, representative feature controls,
  Storybook accessibility coverage, and Playwright certification tests.
- Does not change runtime APIs, provider/model compatibility, persistence, or realtime
  entity flows; tests use deterministic transport mocks and observe the existing UI state.
- C-15 completion and the phase counter must be recorded through the canonical KBD
  `change transition` command before the OpenSpec change is archived.
