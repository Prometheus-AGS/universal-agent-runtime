import { describe, expect, it } from "vitest";
import { render } from "@testing-library/react";
import axe from "axe-core";
import { uarBasicCatalog } from "../src/catalog/uar-basic-catalog";
import { UarSurface, type UarTheme } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

const themes: UarTheme[] = ["light", "dark", "high-contrast"];

describe("axe-core certified surfaces", () => {
  for (const theme of themes) {
    it(`${theme} controls have no serious or critical violations`, async () => {
      const { surface } = buildSurface(uarBasicCatalog, [
        { id: "root", component: "Column", children: ["title", "name", "accept", "choice", "action"] },
        { id: "title", component: "Text", variant: "h2", text: "Review request" },
        { id: "name", component: "TextField", label: "Display name", accessibility: { description: "Shown to collaborators" } },
        { id: "accept", component: "CheckBox", label: "Approve request" },
        { id: "choice", component: "ChoicePicker", label: "Priority", options: [{ label: "Normal", value: "normal" }, { label: "Urgent", value: "urgent" }] },
        { id: "action", component: "Button", child: "actionText", action: { event: { name: "save", context: {} } } },
        { id: "actionText", component: "Text", text: "Save" },
      ]);
      const { container } = render(<UarSurface surface={surface} theme={theme} locale="en" />);
      const result = await axe.run(container, { resultTypes: ["violations"] });
      expect(result.violations.filter((violation) => ["serious", "critical"].includes(violation.impact ?? ""))).toEqual([]);
    });
  }
});
