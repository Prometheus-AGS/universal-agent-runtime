import { describe, expect, test } from "vitest";

import { A2UI_CATALOG_ID, A2UI_PROFILE, A2UI_VERSION } from "./a2ui-protocol";
import {
  MAX_A2UI_COMPONENTS,
  MAX_A2UI_MESSAGES,
  MAX_A2UI_SOURCE_BYTES,
  MAX_A2UI_SURFACES,
} from "./a2ui-rendering-limits";
import { A2uiStreamAccumulator } from "./a2ui-stream-accumulator";

describe("A2uiStreamAccumulator", () => {
  test("retains one ordered surface sequence across separate AG-UI frames", () => {
    const accumulator = new A2uiStreamAccumulator();
    const create = accumulator.advance({
      version: A2UI_VERSION,
      profile: A2UI_PROFILE,
      createSurface: { surfaceId: "policy", catalogId: A2UI_CATALOG_ID },
    });
    const components = accumulator.advance({
      version: A2UI_VERSION,
      updateComponents: {
        surfaceId: "policy",
        components: [{ id: "root", component: "Text", text: "Ready" }],
      },
    });
    const data = accumulator.advance({
      version: A2UI_VERSION,
      updateDataModel: { surfaceId: "policy", path: "/", value: { ready: true } },
    });

    expect(create).toMatchObject({ action: "none", displayed: false, generation: 1, surfaceId: "policy" });
    expect(components).toMatchObject({
      action: "create",
      displayed: true,
      profile: A2UI_PROFILE,
      version: A2UI_VERSION,
    });
    expect(components?.messages).toHaveLength(2);
    expect(data).toMatchObject({ action: "update", surfaceId: "policy" });
    expect(data?.messages).toHaveLength(3);
  });

  test("publishes a terminal delete update and resets the surface accumulator", () => {
    const accumulator = new A2uiStreamAccumulator();
    accumulator.advance({
      version: A2UI_VERSION,
      profile: A2UI_PROFILE,
      createSurface: { surfaceId: "policy", catalogId: A2UI_CATALOG_ID },
    });
    accumulator.advance({
      version: A2UI_VERSION,
      updateComponents: {
        surfaceId: "policy",
        components: [{ id: "root", component: "Text", text: "Ready" }],
      },
    });

    expect(accumulator.advance({
      version: A2UI_VERSION,
      deleteSurface: { surfaceId: "policy" },
    })).toMatchObject({ action: "update", type: "deleteSurface" });
    expect(accumulator.advance({
      version: A2UI_VERSION,
      profile: A2UI_PROFILE,
      createSurface: { surfaceId: "policy", catalogId: A2UI_CATALOG_ID },
    })).toMatchObject({ action: "none", generation: 2 });
  });

  test("rejects a surface once before stream accumulation can exceed its budgets", () => {
    const accumulator = new A2uiStreamAccumulator();
    accumulator.advance({
      version: A2UI_VERSION,
      profile: A2UI_PROFILE,
      createSurface: { surfaceId: "policy", catalogId: A2UI_CATALOG_ID },
    });
    accumulator.advance({
      version: A2UI_VERSION,
      updateComponents: {
        surfaceId: "policy",
        components: [{ id: "root", component: "Text", text: "Ready" }],
      },
    });
    for (let index = 0; index < MAX_A2UI_MESSAGES - 2; index += 1) {
      accumulator.advance({
        version: A2UI_VERSION,
        updateDataModel: { surfaceId: "policy", path: "/count", value: index },
      });
    }

    expect(accumulator.advance({
      version: A2UI_VERSION,
      updateDataModel: { surfaceId: "policy", path: "/count", value: "overflow" },
    })).toMatchObject({
      action: "reject",
      error: `A2UI stream exceeds the ${MAX_A2UI_MESSAGES}-message rendering limit.`,
    });
    expect(accumulator.advance({
      version: A2UI_VERSION,
      updateDataModel: { surfaceId: "policy", path: "/count", value: "ignored" },
    })).toBeNull();
  });

  test("enforces byte, component, and lifecycle limits before retaining frames", () => {
    const bytes = new A2uiStreamAccumulator();
    expect(bytes.advance({
      version: A2UI_VERSION,
      createSurface: {
        surfaceId: "oversized",
        catalogId: "x".repeat(MAX_A2UI_SOURCE_BYTES),
      },
    })).toMatchObject({ action: "reject", error: expect.stringContaining("256 KiB") });

    const components = new A2uiStreamAccumulator();
    components.advance({
      version: A2UI_VERSION,
      createSurface: { surfaceId: "components", catalogId: A2UI_CATALOG_ID },
    });
    const componentRejection = components.advance({
      version: A2UI_VERSION,
      updateComponents: {
        surfaceId: "components",
        components: Array.from({ length: MAX_A2UI_COMPONENTS + 1 }, (_, id) => ({
          id: String(id),
          component: "Text",
          text: "Ready",
        })),
      },
    });
    expect(componentRejection).toMatchObject({
      action: "reject",
      displayed: false,
      error: expect.stringContaining("500-component"),
    });
    expect(JSON.parse(componentRejection?.diagnosticSource ?? "{}")).toMatchObject({
      acceptedMessageCount: 1,
      rejectedFrameExcerpt: expect.stringContaining("updateComponents"),
    });
    expect(new TextEncoder().encode(componentRejection?.diagnosticSource).byteLength).toBeLessThan(9 * 1024);

    const surfaces = new A2uiStreamAccumulator();
    for (let index = 0; index < MAX_A2UI_SURFACES; index += 1) {
      expect(surfaces.advance({
        version: A2UI_VERSION,
        createSurface: { surfaceId: `surface-${index}`, catalogId: A2UI_CATALOG_ID },
      })?.action).toBe("none");
    }
    expect(surfaces.advance({
      version: A2UI_VERSION,
      createSurface: { surfaceId: "surface-overflow", catalogId: A2UI_CATALOG_ID },
    })).toMatchObject({
      action: "reject",
      displayed: false,
      error: expect.stringContaining("16-surface"),
    });
  });
});
