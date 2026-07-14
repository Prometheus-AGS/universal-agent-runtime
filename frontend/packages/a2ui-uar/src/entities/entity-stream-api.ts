import { z } from "zod";
import { AccessibilityAttributesSchema } from "@prometheus-ags/a2ui-core/v0_9";

/**
 * `EntityStream` — a UAR-specific A2UI component (Change 18,
 * `a2ui-migrate-entity-components-from-prometheus-entity-management`) that
 * subscribes to a live-updating data-model path and renders each item as it
 * arrives, e.g. a stream of tool-call events or realtime entity updates.
 *
 * Unlike `EntityCard`/`EntityDiff`, this component is registered via
 * `createBinderlessUarComponentImplementation` (see `create-component.tsx`):
 * `source` is a *static* `{ path }` pointer read directly from
 * `context.componentModel.properties`, not a `DynamicString` resolved by
 * `GenericBinder`. The component subscribes to that path itself via
 * `context.dataContext.subscribeDynamicValue`, because `GenericBinder`'s
 * declarative model re-renders the whole component on any bound-value
 * change — a stream wants to imperatively append/prepend items without
 * losing scroll position or discarding items no longer in the data model
 * (which a re-run of the generic binder's single-value resolution would
 * do). See `EntityStream.tsx` for the subscription itself.
 */
export const EntityStreamApi = {
  name: "EntityStream",
  schema: z
    .object({
      accessibility: AccessibilityAttributesSchema.optional(),
      weight: z.number().optional(),
      /** The entity's logical kind, e.g. `"ToolCall"` — mirrors `EntityType` in prometheus-entity-management. */
      entityType: z
        .string()
        .describe(
          "The logical kind of the streamed entities (e.g. 'ToolCall'), mirroring EntityType in prometheus-entity-management.",
        ),
      /**
       * A static data-model path (not a `DynamicString`) that resolves to an
       * array. `EntityStream` subscribes to this path directly rather than
       * going through `GenericBinder`'s single-value resolution.
       */
      source: z
        .object({ path: z.string() })
        .strict()
        .describe(
          "A JSON-pointer path into the data model that resolves to an array of stream items. Subscribed to directly, not resolved via GenericBinder.",
        ),
      title: z
        .string()
        .describe("Static heading shown above the stream.")
        .optional(),
    })
    .strict()
    .describe(
      "Subscribes to a live-updating array at a data-model path and renders each item as a simple label/value row. UAR-specific — not part of the uar.a2ui/1 9-component protocol baseline.",
    ),
};

/** The shape of a single item this pass of `EntityStream` expects at `source.path`. */
export interface EntityStreamItem {
  id: string;
  label: string;
  value?: string;
}
