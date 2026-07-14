import { Catalog } from "@prometheus-ags/a2ui-core/v0_9";
import { UarEntityCard } from "../entities/EntityCard";
import { EntityCardApi } from "../entities/entity-card-api";
import { createUarComponentImplementation } from "../react/create-component";
import type { UarComponentImplementation } from "../react/types";
import { uarBasicCatalogComponents } from "./uar-basic-catalog";

/**
 * Catalog id for the UAR entity-aware extension surface: the 9
 * `uar.a2ui/1` protocol components plus the (so far) 1 of 7 planned
 * `Entity*` components — `EntityCard`. `EntityDiff`, `EntityStream`,
 * `EntityApproval`, `EntityToolProvider`, `EntityChat`, and
 * `EntityCopilot` are deferred to a follow-up pass (see this package's
 * README "Deferred" section) and Change 18
 * (`a2ui-entity-component-migration`).
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
];

export const uarEntityCatalog = new Catalog<UarComponentImplementation>(
  UAR_A2UI_ENTITY_CATALOG_ID,
  uarEntityCatalogComponents,
);
