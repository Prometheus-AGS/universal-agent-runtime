# Review plan

1. Confirm the complete local build sequence and generated-reference staging.
2. Re-run the composed publication validator against `website/build` and retain
   all focused negative-control results.
3. Serve the production artifact, validate every required route, and observe a
   missing-route control fail.
4. Exercise representative pages, local search, Mermaid, keyboard focus,
   desktop/mobile themes, accessibility, console, and network behavior.
5. Inspect the four screenshots and audit the diff for prohibited surfaces.
6. Strict-validate all eleven OpenSpec changes and publish a constraint-by-
   constraint review summary with explicit limits.
