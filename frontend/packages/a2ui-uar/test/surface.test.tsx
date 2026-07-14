import { describe, expect, it, vi } from "vitest";
import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { uarBasicCatalog } from "../src/catalog/uar-basic-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

describe("UarSurface", () => {
  it("renders a Text component bound to a data path", () => {
    const { surface } = buildSurface(
      uarBasicCatalog,
      [{ id: "root", component: "Text", text: { path: "/greeting" } }],
      { greeting: "Hello, UAR" },
    );

    render(<UarSurface surface={surface} />);

    expect(screen.getByText("Hello, UAR")).toBeInTheDocument();
  });

  it("re-renders reactively when the bound data model value changes", () => {
    const { surface } = buildSurface(
      uarBasicCatalog,
      [{ id: "root", component: "Text", text: { path: "/greeting" } }],
      { greeting: "Before" },
    );

    render(<UarSurface surface={surface} />);
    expect(screen.getByText("Before")).toBeInTheDocument();

    act(() => {
      surface.dataModel.set("/greeting", "After");
    });

    expect(screen.getByText("After")).toBeInTheDocument();
    expect(screen.queryByText("Before")).not.toBeInTheDocument();
  });

  it("renders a Row of Column/Card/Divider/Text via structural ChildList props", () => {
    const { surface } = buildSurface(uarBasicCatalog, [
      { id: "root", component: "Row", children: ["col"] },
      { id: "col", component: "Column", children: ["card", "divider", "label"] },
      { id: "card", component: "Card", child: "cardText" },
      { id: "cardText", component: "Text", text: "Inside the card" },
      { id: "divider", component: "Divider" },
      { id: "label", component: "Text", text: "Below the divider" },
    ]);

    render(<UarSurface surface={surface} />);

    expect(screen.getByText("Inside the card")).toBeInTheDocument();
    expect(screen.getByText("Below the divider")).toBeInTheDocument();
    expect(document.querySelector('[data-a2ui-component="Divider"]')).toBeInTheDocument();
    expect(document.querySelector('[data-a2ui-component="Row"]')).toBeInTheDocument();
    expect(document.querySelector('[data-a2ui-component="Column"]')).toBeInTheDocument();
  });

  it("dispatches an Action from a Button click through SurfaceModel.onAction", async () => {
    const { surface } = buildSurface(uarBasicCatalog, [
      {
        id: "root",
        component: "Button",
        child: "btnText",
        action: { event: { name: "submit", context: {} } },
      },
      { id: "btnText", component: "Text", text: "Submit" },
    ]);

    const onAction = vi.fn();
    surface.onAction.subscribe(onAction);

    render(<UarSurface surface={surface} />);
    await userEvent.click(screen.getByText("Submit"));

    expect(onAction).toHaveBeenCalledTimes(1);
    expect(onAction.mock.calls[0][0]).toMatchObject({ name: "submit" });
  });

  it("throws UnknownUarComponentError for a component type outside the catalog (fail-closed security boundary)", () => {
    const { surface } = buildSurface(uarBasicCatalog, [
      { id: "root", component: "NotARealComponent" },
    ]);

    // React logs the thrown error to console.error even when caught by our
    // assertion — silence that expected noise for this one test.
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(() => render(<UarSurface surface={surface} />)).toThrow(/Unknown A2UI component type/);
    consoleError.mockRestore();
  });
});
