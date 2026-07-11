import { describe, expect, test } from "vitest";

import { A2UI_CATALOG_ID, a2uiMessageSchema, validateA2uiMessage } from "@/features/a2ui/a2ui-protocol";

describe("UAR A2UI production profile", () => {
  test("accepts v0.9.1 messages using the approved catalog", () => {
    expect(a2uiMessageSchema.safeParse({
      version: "v0.9.1",
      profile: "uar.a2ui/1",
      createSurface: { surfaceId: "surface-1", catalogId: A2UI_CATALOG_ID },
    }).success).toBe(true);
    expect(a2uiMessageSchema.safeParse({
      version: "v0.9.1",
      profile: "uar.a2ui/1",
      updateComponents: {
        surfaceId: "surface-1",
        components: [
          { id: "label", component: "Text", text: "Continue?" },
          {
            id: "submit",
            component: "Button",
            child: "label",
            action: { event: { name: "continue" } },
          },
        ],
      },
    }).success).toBe(true);
  });

  test("rejects unknown catalogs, components, props, and candidate versions", () => {
    const messages = [
      {
        version: "v0.9.1",
        profile: "uar.a2ui/1",
        createSurface: { surfaceId: "surface-1", catalogId: "https://agent.invalid/catalog.json" },
      },
      {
        version: "v0.9.1",
        profile: "uar.a2ui/1",
        updateComponents: {
          surfaceId: "surface-1",
          components: [{ id: "root", component: "Script", source: "alert(1)" }],
        },
      },
      {
        version: "v0.9.1",
        profile: "uar.a2ui/1",
        updateComponents: {
          surfaceId: "surface-1",
          components: [{ id: "root", component: "Text", text: "safe", dangerouslySetInnerHTML: "<b>x</b>" }],
        },
      },
      {
        version: "v1.0",
        profile: "uar.a2ui/1",
        deleteSurface: { surfaceId: "surface-1" },
      },
    ];
    expect(messages.every((message) => !a2uiMessageSchema.safeParse(message).success)).toBe(true);
  });

  test("rejects executable content and duplicate component IDs", () => {
    const update = (components: unknown[]) => ({
      version: "v0.9.1",
      profile: "uar.a2ui/1",
      updateComponents: { surfaceId: "surface-1", components },
    });
    expect(validateA2uiMessage(update([
      { id: "root", component: "Text", text: "<img src=x onerror=alert(1)>" },
    ]))).toMatchObject({ success: false, error: expect.stringContaining("Executable") });
    expect(validateA2uiMessage(update([
      { id: "root", component: "Text", text: "one" },
      { id: "root", component: "Text", text: "two" },
    ]))).toMatchObject({ success: false, error: expect.stringContaining("unique") });
    expect(validateA2uiMessage(update([
      { id: "label", component: "Text", text: "Safe" },
      {
        id: "button",
        component: "Button",
        child: "label",
        action: { event: { name: "submit", context: { value: "javascript:alert(1)" } } },
      },
    ]))).toMatchObject({ success: false, error: expect.stringContaining("Executable") });
  });
});
