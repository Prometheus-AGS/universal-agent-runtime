## 1. Toolchain

- [x] 1.1 Upgrade the frontend to exact Tailwind 4.3.3 and `@tailwindcss/vite` 4.3.3, replace the legacy animation plugin with `tw-animate-css`, and preserve unrelated package/lockfile changes.
- [x] 1.2 Register the Tailwind Vite plugin and remove the legacy Tailwind and PostCSS config files.

## 2. CSS-First Token Foundation

- [x] 2.1 Create `frontend/src/shared/theme/tokens.css` with Tailwind imports, explicit source coverage, complete-color KnowMe tokens, run-phase roles, typography, radii, motion, and semantic compatibility aliases.
- [x] 2.2 Import the shared token source from `frontend/src/index.css` while preserving dark, light, high-contrast, system-light, project keyframe, and reduced-motion behavior.

## 3. Dangling References

- [x] 3.1 Clear the Tailwind config path in `frontend/components.json` for Tailwind 4.
- [x] 3.2 Repoint both Storybook visual-regression path filters from the deleted config to `frontend/src/shared/theme/tokens.css`.
- [x] 3.3 Verify no live frontend or workflow configuration references either deleted config file.

## 4. Validation

- [x] 4.1 Add targeted assertions for dependency pins, Vite/CSS integration, token roles, theme compatibility, animation coverage, and dangling-reference absence.
- [x] 4.2 Run frontend typecheck, lint, the targeted token/config assertions, strict OpenSpec validation, and artifact refinement. Lint ran; its pre-existing generated-output failure is recorded in `verification.md` as an external phase condition. The isolated adversarial review remains the external pre-archive gate.
