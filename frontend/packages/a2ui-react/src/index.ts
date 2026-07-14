/*
 * Copyright 2025 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * ============================================================================
 * REFERENCE IMPLEMENTATION ONLY — DO NOT IMPORT FROM UAR PRODUCT CODE.
 * ============================================================================
 *
 * `@prometheus-ags/a2ui-react` is a vendored, version-pinned re-export of
 * Google's `@a2ui/react` (the A2UI project's official React renderer,
 * https://a2ui.org/). It is kept in this repo strictly as a cross-testing
 * / behavioral-reference implementation for Change 17
 * (`a2ui-uar-renderer-on-webcore`), which cross-tests the UAR-owned
 * renderer against this package to confirm semantic parity.
 *
 * The UAR-owned renderer is `@prometheus-ags/a2ui-uar`, built on
 * `@prometheus-ags/a2ui-core` (the vendored `@a2ui/web_core`). That is the
 * only A2UI renderer product code should depend on.
 *
 * See `frontend/packages/a2ui-core/UPSTREAM.md` for the vendoring
 * rationale shared by both packages (this package is pinned the same way,
 * against `@a2ui/react`).
 */
export * from '@a2ui/react';
