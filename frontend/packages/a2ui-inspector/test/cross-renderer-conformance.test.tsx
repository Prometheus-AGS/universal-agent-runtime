import { cleanup as cleanupReact, render as renderReact } from "@testing-library/react";
import { cleanup as cleanupSvelte, render as renderSvelte } from "@testing-library/svelte";
import { afterEach, describe, expect, it } from "vitest";
import { Catalog, MessageProcessor, type ComponentApi } from "@prometheus-ags/a2ui-core/v0_9";
import { ButtonApi, CardApi, CheckBoxApi, ChoicePickerApi, ColumnApi, DividerApi, RowApi, TextApi, TextFieldApi } from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import { semanticSnapshot, UAR_CONFORMANCE_COMPONENTS, UAR_CONFORMANCE_DATA } from "@prometheus-ags/a2ui-core/conformance";
import { A2uiLitSurface } from "@prometheus-ags/a2ui-lit";
import A2uiSvelteSurface from "@prometheus-ags/a2ui-svelte";
import { UarSurface, uarBasicCatalog } from "@prometheus-ags/a2ui-uar";

const semanticCatalog = new Catalog<ComponentApi>("semantic", [TextApi, ButtonApi, TextFieldApi, CheckBoxApi, ChoicePickerApi, RowApi, ColumnApi, CardApi, DividerApi]);
function build<T extends ComponentApi>(catalog: Catalog<T>, id: string) { const processor = new MessageProcessor([catalog]); processor.processMessages([{ version: "v0.9", createSurface: { surfaceId: id, catalogId: catalog.id } }, { version: "v0.9", updateComponents: { surfaceId: id, components: [...UAR_CONFORMANCE_COMPONENTS] } }, { version: "v0.9", updateDataModel: { surfaceId: id, path: "/", value: { ...UAR_CONFORMANCE_DATA } } }]); return processor.model.getSurface(id)!; }
afterEach(() => { cleanupReact(); cleanupSvelte(); document.body.replaceChildren(); });
describe("React/Lit/Svelte semantic conformance", () => {
  it("renders equivalent accessibility-relevant output", async () => {
    const react = renderReact(<UarSurface surface={build(uarBasicCatalog, "react")} />);
    const lit = new A2uiLitSurface(); lit.surface = build(semanticCatalog, "lit"); document.body.append(lit); await lit.updateComplete;
    const svelte = renderSvelte(A2uiSvelteSurface, { surface: build(semanticCatalog, "svelte") }); await new Promise<void>((resolve) => queueMicrotask(resolve));
    const reactSnapshot = semanticSnapshot(react.container);
    expect(semanticSnapshot(lit)).toEqual(reactSnapshot);
    expect(semanticSnapshot(svelte.container)).toEqual(reactSnapshot);
  });
});
