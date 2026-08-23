## 1. Add the final publication validators

- [x] 1.1 Add a data-driven deployed-route validator covering every required product route, root, documentation/history, and generated API references with bounded retries and intended-portal markers.
- [x] 1.2 Add a local Playwright/axe browser validator for four viewport/theme combinations, representative navigation, search, Mermaid, keyboard focus, console/network health, responsive overflow, screenshots, and WCAG A/AA violations.
- [x] 1.3 Wire deployed-route validation into the sole Pages workflow as deployment validation only; add local package commands and negative controls without changing dependencies or locks.

## 2. Build the complete local artifact once

- [x] 2.1 Run frozen npm installs for the website and TypeScript SDK.
- [x] 2.2 Run the Docusaurus production build, UAR library Rustdoc under server-full, TypeDoc, and real reference staging in order.
- [x] 2.3 Run composed publication with the built output, truth/link/route/privacy/history/README/brand/architecture/product/security/developer/staging/Actions controls, and strict validation for all eleven phase changes.

## 3. Validate the rendered artifact locally

- [x] 3.1 Serve the built artifact and run the deployed-route validator against localhost; observe a missing-route negative control fail.
- [x] 3.2 Run browser validation and retain desktop/mobile light/dark screenshots with zero console/network failures, successful navigation/search/Mermaid/focus assertions, no horizontal overflow, and no WCAG A/AA violations.
- [x] 3.3 If a defect occurs, fix only its documentation source, rebuild the complete artifact, and rerun the affected and final checks.

## 4. Record and publish

- [x] 4.1 Run strict OpenSpec validation and artifact-refiner final review; audit the diff for prohibited runtime/product/dependency/lock/vendor/submodule changes.
- [x] 4.2 Write row-form verification with exact commands/output, source SHA, documentation limits, screenshot inventory, and deferred runtime claims.
- [x] 4.3 Transition the KBD change complete, run KBD reflection, and commit all final source/evidence/KBD artifacts independently.
- [x] 4.4 Push the branch, dispatch the deployment workflow, observe its successful deployed-route validation, and independently validate the live canonical URL.
- [x] 4.5 Set the GitHub repository homepage to the observed URL, verify README/homepage equality, create and record the PR, then preserve the `main`-only Pages policy by merging through the protected branch before the live gate.
