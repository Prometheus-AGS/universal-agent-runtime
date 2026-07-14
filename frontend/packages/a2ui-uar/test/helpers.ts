import { MessageProcessor, type Catalog } from "@prometheus-ags/a2ui-core/v0_9";
import type { UarComponentImplementation } from "../src/react/types";

/**
 * Builds a `MessageProcessor` over the given catalog, creates a single
 * surface, and applies `updateComponents`/`updateDataModel` messages —
 * the minimum wire traffic needed to get a renderable `SurfaceModel` out
 * of `web_core`, matching what a real UAR agent session would send per
 * `docs/protocols/a2ui-profile.md`.
 */
export function buildSurface(
  catalog: Catalog<UarComponentImplementation>,
  components: Record<string, unknown>[],
  data: Record<string, unknown> = {},
  surfaceId = "test-surface",
) {
  const processor = new MessageProcessor([catalog]);
  processor.processMessages([
    {
      version: "v0.9",
      createSurface: { surfaceId, catalogId: catalog.id },
    },
    {
      version: "v0.9",
      updateComponents: { surfaceId, components },
    },
    {
      version: "v0.9",
      updateDataModel: { surfaceId, path: "/", value: data },
    },
  ]);

  const surface = processor.model.getSurface(surfaceId);
  if (!surface) {
    throw new Error(`Surface "${surfaceId}" was not created.`);
  }
  return { processor, surface };
}
