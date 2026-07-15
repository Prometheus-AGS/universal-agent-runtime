import type { ReactElement } from "react";
import { MessageProcessor, type Catalog } from "@prometheus-ags/a2ui-core/v0_9";
import type { UarComponentImplementation } from "../react/types";
import { UarSurface } from "../react/UarSurface";

/**
 * Storybook-only counterpart of `test/helpers.ts`'s `buildSurface`: drives
 * a real `MessageProcessor` through the same wire messages a live UAR
 * agent session would send (`createSurface`/`updateComponents`/
 * `updateDataModel`), so stories render components through the actual
 * `GenericBinder` resolution path rather than hand-constructed props that
 * could drift from what the protocol produces at runtime.
 */
export function renderStorySurface(
  catalog: Catalog<UarComponentImplementation>,
  components: Record<string, unknown>[],
  data: Record<string, unknown> = {},
): ReactElement {
  const processor = new MessageProcessor([catalog]);
  processor.processMessages([
    { version: "v0.9", createSurface: { surfaceId: "story", catalogId: catalog.id } },
    { version: "v0.9", updateComponents: { surfaceId: "story", components } },
    { version: "v0.9", updateDataModel: { surfaceId: "story", path: "/", value: data } },
  ]);
  const surface = processor.model.getSurface("story");
  if (!surface) {
    throw new Error('Story surface "story" was not created.');
  }
  return <UarSurface surface={surface} />;
}
