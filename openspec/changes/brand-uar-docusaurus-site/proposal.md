## Why

The public portal still presents the stock Docusaurus tutorial identity even
though UAR ships a complete ember/cyan brand system, reusable logo assets, and a
Flat 2.0 interaction language. Readers cannot currently recognize the portal as
the same product or discover the planned documentation efficiently.

## What Changes

- Replace all stock Docusaurus identity, tutorial copy, sample illustrations,
  and green theme tokens with the shipped UAR mark, wordmark, ember/cyan palette,
  surface ladder, typography, and product voice.
- Replace the tutorial homepage with a product-specific orientation that explains
  the runtime boundary, supported surfaces, profile limits, and clear next steps.
- Reconcile navbar, footer, sidebars, Mermaid, code blocks, focus states, and
  responsive behavior with the React application's Flat 2.0 design contract.
- Add deterministic local documentation search without a hosted search service.
- Preserve light, dark, system-preference, reduced-motion, keyboard, zoom, and
  high-contrast readability requirements for final local certification.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `dev-portal-2026`: Require the portal to use the shipped UAR identity and
  interaction tokens, expose product-specific orientation and local search, and
  remain usable across responsive, keyboard, light, dark, and reduced-motion
  conditions.

## Impact

- **Site UI:** Docusaurus configuration, homepage/components, theme CSS, sidebar,
  and public brand assets under `website/`.
- **Documentation dependencies:** Exact documentation-only font and local-search
  packages are added to `website/package.json` and its npm lockfile.
- **Runtime UX:** The public site mirrors the existing React application's brand;
  it does not alter the application itself or runtime state.
- **Provider compatibility:** No model, provider, credential, or inference changes.
- **Realtime state:** No SSE, AG-UI, A2UI, or entity-graph behavior changes.
- **KBD:** The registered branding change advances only after OpenSpec and bounded
  source-level UI review pass; full build and browser certification remain at the
  final phase gate after all content exists.
