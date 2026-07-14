import { describe, expect, it } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { MessageProcessor } from "@prometheus-ags/a2ui-core/v0_9";
import { A2uiSurface as ReferenceA2uiSurface, basicCatalog as referenceBasicCatalog } from "@prometheus-ags/a2ui-react/v0_9";
import { uarBasicCatalog } from "../src/catalog/uar-basic-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

/**
 * Cross-tests the UAR renderer against `@a2ui/react` (vendored as
 * `@prometheus-ags/a2ui-react`, reference-implementation only — see that
 * package's README) on the *same wire messages*, asserting semantic
 * equivalence (accessible roles/names/text content) rather than DOM/CSS
 * equality, since the two renderers deliberately use different visual
 * systems (shadcn/ui + react-aria-components here vs. `@a2ui/react`'s own
 * basic_catalog styles there). This satisfies Change 17's cross-testing
 * requirement for a representative subset of the 9 `uar.a2ui/1` protocol
 * components; a full parity matrix across all 9 is deferred (see README).
 */

function buildReferenceSurface(components: Record<string, unknown>[], data: Record<string, unknown> = {}) {
  const processor = new MessageProcessor([referenceBasicCatalog]);
  const surfaceId = "reference-surface";
  processor.processMessages([
    { version: "v0.9", createSurface: { surfaceId, catalogId: referenceBasicCatalog.id } },
    { version: "v0.9", updateComponents: { surfaceId, components } },
    { version: "v0.9", updateDataModel: { surfaceId, path: "/", value: data } },
  ]);
  const surface = processor.model.getSurface(surfaceId);
  if (!surface) throw new Error("Reference surface was not created.");
  return surface;
}

describe("cross-testing: UAR renderer vs. @a2ui/react reference", () => {
  it("Text: both renderers show the same bound text content", () => {
    const message = [{ id: "root", component: "Text", text: { path: "/greeting" } }];
    const data = { greeting: "Hello from A2UI" };

    const { surface: uarSurface } = buildSurface(uarBasicCatalog, message, data);
    const { unmount: unmountUar } = render(<UarSurface surface={uarSurface} />);
    expect(screen.getByText("Hello from A2UI")).toBeInTheDocument();
    unmountUar();

    const refSurface = buildReferenceSurface(message, data);
    render(<ReferenceA2uiSurface surface={refSurface} />);
    expect(screen.getByText("Hello from A2UI")).toBeInTheDocument();
  });

  it("Button: both renderers expose a clickable button with the same accessible name", () => {
    const message = [
      {
        id: "root",
        component: "Button",
        child: "label",
        action: { event: { name: "go", context: {} } },
      },
      { id: "label", component: "Text", text: "Continue" },
    ];

    const { surface: uarSurface } = buildSurface(uarBasicCatalog, message);
    const { unmount: unmountUar, container: uarContainer } = render(<UarSurface surface={uarSurface} />);
    expect(within(uarContainer).getByRole("button", { name: "Continue" })).toBeInTheDocument();
    unmountUar();

    const refSurface = buildReferenceSurface(message);
    const { container: refContainer } = render(<ReferenceA2uiSurface surface={refSurface} />);
    expect(within(refContainer).getByRole("button", { name: "Continue" })).toBeInTheDocument();
  });

  it("CheckBox: both renderers expose a checkbox with the same label and checked state", () => {
    const message = [
      { id: "root", component: "CheckBox", label: "Subscribe", value: { path: "/subscribed" } },
    ];
    const data = { subscribed: true };

    const { surface: uarSurface } = buildSurface(uarBasicCatalog, message, data);
    const { unmount: unmountUar, container: uarContainer } = render(<UarSurface surface={uarSurface} />);
    expect(within(uarContainer).getByRole("checkbox", { name: "Subscribe" })).toBeChecked();
    unmountUar();

    const refSurface = buildReferenceSurface(message, data);
    const { container: refContainer } = render(<ReferenceA2uiSurface surface={refSurface} />);
    expect(within(refContainer).getByRole("checkbox", { name: "Subscribe" })).toBeChecked();
  });

  it("Row/Column/Divider: both renderers walk the same structural ChildList tree to the same leaf text", () => {
    const message = [
      { id: "root", component: "Column", children: ["row"] },
      { id: "row", component: "Row", children: ["a", "divider", "b"] },
      { id: "a", component: "Text", text: "Left" },
      { id: "divider", component: "Divider" },
      { id: "b", component: "Text", text: "Right" },
    ];

    const { surface: uarSurface } = buildSurface(uarBasicCatalog, message);
    const { unmount: unmountUar } = render(<UarSurface surface={uarSurface} />);
    expect(screen.getByText("Left")).toBeInTheDocument();
    expect(screen.getByText("Right")).toBeInTheDocument();
    unmountUar();

    const refSurface = buildReferenceSurface(message);
    render(<ReferenceA2uiSurface surface={refSurface} />);
    expect(screen.getByText("Left")).toBeInTheDocument();
    expect(screen.getByText("Right")).toBeInTheDocument();
  });
});
