import { z } from "zod";
import {
  AccessibilityAttributesSchema,
  DynamicStringSchema,
} from "@prometheus-ags/a2ui-core/v0_9";

/**
 * `EntityDiff` — a UAR-specific A2UI component (Change 18,
 * `a2ui-migrate-entity-components-from-prometheus-entity-management`) that
 * renders a before/after comparison of an entity's fields, e.g. after a tool
 * call or a realtime update. See this package's README for why this is
 * described as "migrated" even though no source component actually existed
 * to migrate from.
 *
 * Unlike `EntityCard`, every field row carries two `DynamicString` values
 * (`before`/`after`) rather than one, so the same field can be re-bound
 * independently on either side of the comparison (e.g. `before` pinned to a
 * snapshot path, `after` bound live to the current entity state).
 */
export const EntityDiffApi = {
  name: "EntityDiff",
  schema: z
    .object({
      accessibility: AccessibilityAttributesSchema.optional(),
      weight: z.number().optional(),
      /** The entity's logical kind, e.g. `"Order"` — mirrors `EntityType` in prometheus-entity-management. Static: the shape of a diff is determined by its type, not runtime data. */
      entityType: z
        .string()
        .describe(
          "The entity's logical kind (e.g. 'Order'), mirroring EntityType in prometheus-entity-management.",
        ),
      /** The entity's primary key — mirrors `EntityId`. */
      entityId: DynamicStringSchema.describe(
        "The entity's primary key, mirroring EntityId in prometheus-entity-management.",
      ),
      title: DynamicStringSchema.describe(
        "The diff panel's heading — typically describes what changed and why (e.g. a tool-call name).",
      ).optional(),
      fields: z
        .array(
          z
            .object({
              label: DynamicStringSchema,
              before: DynamicStringSchema,
              after: DynamicStringSchema,
            })
            .strict(),
        )
        .describe(
          "Label/before/after triples. A row is highlighted as changed when the resolved before and after values differ.",
        ),
    })
    .strict()
    .describe(
      "A before/after field comparison for a single entity record. UAR-specific — not part of the uar.a2ui/1 9-component protocol baseline.",
    ),
};
