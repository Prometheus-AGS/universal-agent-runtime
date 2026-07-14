import { describe, expect, it } from "vitest";
import { uarEntityCatalogComponents } from "../src/catalog/uar-entity-catalog";

describe("UAR entity catalog", () => {
  it("contains the baseline and all seven entity components", () => {
    expect(uarEntityCatalogComponents.map(({ name }) => name)).toEqual(expect.arrayContaining([
      "EntityCard", "EntityDiff", "EntityStream", "EntityApproval",
      "EntityToolProvider", "EntityChat", "EntityCopilot",
    ]));
    expect(uarEntityCatalogComponents).toHaveLength(16);
  });
});
