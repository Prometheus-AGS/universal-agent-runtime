import { Catalog } from "@prometheus-ags/a2ui-core/v0_9";
import { UarEntityCard } from "../entities/EntityCard";
import { EntityCardApi } from "../entities/entity-card-api";
import { UarEntityDiff } from "../entities/EntityDiff";
import { EntityDiffApi } from "../entities/entity-diff-api";
import { UarEntityStream } from "../entities/EntityStream";
import { EntityStreamApi } from "../entities/entity-stream-api";
import {
  createBinderlessUarComponentImplementation,
  createUarComponentImplementation,
} from "../react/create-component";
import type { UarComponentImplementation } from "../react/types";
import { uarBasicCatalogComponents } from "./uar-basic-catalog";

/**
 * Catalog id for the UAR entity-aware extension surface: the 9
 * `uar.a2ui/1` protocol components plus 3 of 7 planned `Entity*`
 * components — `EntityCard` (Change 17), `EntityDiff` and `EntityStream`
 * (Change 18, `a2ui-migrate-entity-components-from-prometheus-entity-management`
 * — see that change's proposal for why "migrate" became "build fresh": no
 * source component actually existed under the plan's assumed path).
 * `EntityApproval`, `EntityToolProvider`, `EntityChat`, and `EntityCopilot`
 * remain deferred (closer to mini-applications than single components —
 * see this package's README "Deferred" section).
 *
 * This is a distinct, separately-versioned catalog id from
 * `urn:uar:a2ui:catalog:1` (the certified `uar.a2ui/1` baseline in
 * `docs/protocols/a2ui-profile.md`) because `Entity*` components are a UAR
 * extension, not part of the audited/certified protocol baseline —
 * servers must opt in to `urn:uar:a2ui:catalog:1+entities` explicitly via
 * `createSurface.catalogId` rather than getting extension components for
 * free on the baseline catalog.
 */
export const UAR_A2UI_ENTITY_CATALOG_ID = "urn:uar:a2ui:catalog:1+entities";

export const uarEntityCatalogComponents: UarComponentImplementation[] = [
  ...uarBasicCatalogComponents,
  createUarComponentImplementation(EntityCardApi, UarEntityCard),
  createUarComponentImplementation(EntityDiffApi, UarEntityDiff),
  createBinderlessUarComponentImplementation(EntityStreamApi, UarEntityStream),
];

export const uarEntityCatalog = new Catalog<UarComponentImplementation>(
  UAR_A2UI_ENTITY_CATALOG_ID,
  uarEntityCatalogComponents,
);
