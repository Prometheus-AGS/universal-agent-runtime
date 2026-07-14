/*
 * Copyright 2026 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Re-export of `@a2ui/react`'s v0_9 surface (`A2uiSurface`, `basicCatalog`,
 * `createComponentImplementation`, the 18 basic-catalog component
 * implementations, ...). See ./index.ts for the reference-implementation
 * disclaimer.
 *
 * Added alongside `@prometheus-ags/a2ui-core`'s `./v0_9` subpath (see
 * `frontend/packages/a2ui-core/src/v0_9.ts`) specifically so Change 17
 * (`a2ui-uar-renderer-on-webcore`) can cross-test the UAR renderer — which
 * targets `web_core`'s v0.9 surface — against this reference
 * implementation on the same protocol version. `@a2ui/react`'s own default
 * export (`.`) is v0_8, which uses a different component set/API
 * (`MultipleChoice` instead of `ChoicePicker`, a provider/hook pattern
 * instead of `MessageProcessor` + `SurfaceModel`) and isn't comparable to
 * the v0.9-based UAR renderer.
 */
export * from '@a2ui/react/v0_9';
