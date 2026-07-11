import { describe, expect, test } from "vitest";

import { A2UI_CATALOG_ID } from "@/features/a2ui/a2ui-protocol";
import { EMPTY_A2UI_PROCESSOR_STATE, reduceA2uiMessage } from "@/features/a2ui/a2ui-surface-processor";

const envelope = <T extends Record<string, unknown>>(message: T) => ({
  version: "v0.9.1",
  profile: "uar.a2ui/1",
  ...message,
});

describe("A2UI surface processor", () => {
  test("creates, progressively resolves, updates, and deletes a surface", () => {
    let state = reduceA2uiMessage(EMPTY_A2UI_PROCESSOR_STATE, envelope({
      createSurface: { surfaceId: "surface-1", catalogId: A2UI_CATALOG_ID },
    }));
    expect(state.surfaces["surface-1"]?.ready).toBe(false);

    state = reduceA2uiMessage(state, envelope({
      updateComponents: {
        surfaceId: "surface-1",
        components: [{ id: "root", component: "Column", children: ["message"] }],
      },
    }));
    expect(state.surfaces["surface-1"]?.ready).toBe(false);

    state = reduceA2uiMessage(state, envelope({
      updateComponents: {
        surfaceId: "surface-1",
        components: [{ id: "message", component: "Text", text: { path: "/message" } }],
      },
    }));
    expect(state.surfaces["surface-1"]?.ready).toBe(true);

    state = reduceA2uiMessage(state, envelope({
      updateDataModel: { surfaceId: "surface-1", path: "/message", value: "Ready" },
    }));
    expect(state.surfaces["surface-1"]?.data).toEqual({ message: "Ready" });

    state = reduceA2uiMessage(state, envelope({ deleteSurface: { surfaceId: "surface-1" } }));
    expect(state.surfaces["surface-1"]).toBeUndefined();
  });

  test("retains the last safe state when validation fails", () => {
    const created = reduceA2uiMessage(EMPTY_A2UI_PROCESSOR_STATE, envelope({
      createSurface: { surfaceId: "surface-1", catalogId: A2UI_CATALOG_ID },
    }));
    const invalid = reduceA2uiMessage(created, envelope({
      updateComponents: {
        surfaceId: "surface-1",
        components: [{ id: "root", component: "Text", text: "<script>bad()</script>" }],
      },
    }));
    expect(invalid.surfaces).toEqual(created.surfaces);
    expect(invalid.error).toMatch(/executable/i);
  });
});
