import { afterEach, describe, expect, it } from "vitest";
import { ButtonApi, CardApi, CheckBoxApi, ChoicePickerApi, ColumnApi, DividerApi, RowApi, TextApi, TextFieldApi } from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import { Catalog, ComponentModel, MessageProcessor, type ComponentApi } from "@prometheus-ags/a2ui-core/v0_9";
import { renderSemanticSurface } from "@prometheus-ags/a2ui-core/semantic-dom";
import { A2uiLitSurface } from "../src";

const catalog = new Catalog<ComponentApi>("test", [TextApi, ButtonApi, TextFieldApi, CheckBoxApi, ChoicePickerApi, RowApi, ColumnApi, CardApi, DividerApi]);
function surface(components: Record<string, unknown>[], data: Record<string, unknown> = {}) { const processor = new MessageProcessor([catalog]); processor.processMessages([{ version: "v0.9", createSurface: { surfaceId: "s", catalogId: "test" } }, { version: "v0.9", updateComponents: { surfaceId: "s", components } }, { version: "v0.9", updateDataModel: { surfaceId: "s", path: "/", value: data } }]); const model = processor.model.getSurface("s"); if (!model) throw new Error("surface missing"); return model; }
afterEach(() => document.body.replaceChildren());
describe("A2uiLitSurface", () => {
  it("renders structural children and bound state", async () => { const element = new A2uiLitSurface(); element.surface = surface([{ id: "root", component: "Row", children: ["text", "check"] }, { id: "text", component: "Text", text: { path: "/label" } }, { id: "check", component: "CheckBox", label: "Ready", value: { path: "/ready" } }], { label: "Hello", ready: true }); document.body.append(element); await element.updateComplete; expect(element.textContent).toContain("Hello"); expect(element.querySelector<HTMLInputElement>('input[type="checkbox"]')?.checked).toBe(true); });
  it("reacts to data updates", async () => { const model = surface([{ id: "root", component: "Text", text: { path: "/label" } }], { label: "Before" }); const element = new A2uiLitSurface(); element.surface = model; document.body.append(element); await element.updateComplete; model.dataModel.set("/label", "After"); await new Promise<void>((resolve) => queueMicrotask(resolve)); expect(element.textContent).toContain("After"); });
  it("fails closed on unknown components", () => { const model = surface([{ id: "root", component: "Text", text: "safe" }]); model.componentsModel.removeComponent("root"); model.componentsModel.addComponent(new ComponentModel("root", "Unknown", {})); expect(() => renderSemanticSurface(model, document.createElement("div"))).toThrow(/Unknown.*root/); });
});
