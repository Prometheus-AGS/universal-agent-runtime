import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { uarBasicCatalog } from "../src/catalog/uar-basic-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

describe("TextField", () => {
  it("two-way binds value: typing writes back to the data model via the generated setValue", async () => {
    const { surface } = buildSurface(
      uarBasicCatalog,
      [{ id: "root", component: "TextField", label: "Name", value: { path: "/name" } }],
      { name: "" },
    );

    render(<UarSurface surface={surface} />);
    const input = screen.getByLabelText("Name");
    await userEvent.type(input, "Ada");

    expect(surface.dataModel.get("/name")).toBe("Ada");
  });

  it("renders a textarea for the longText variant", () => {
    const { surface } = buildSurface(uarBasicCatalog, [
      { id: "root", component: "TextField", label: "Notes", variant: "longText" },
    ]);
    render(<UarSurface surface={surface} />);
    expect(screen.getByLabelText("Notes").tagName).toBe("TEXTAREA");
  });
});

describe("CheckBox", () => {
  it("two-way binds value: clicking toggles the data model via the generated setValue", async () => {
    const { surface } = buildSurface(
      uarBasicCatalog,
      [{ id: "root", component: "CheckBox", label: "Accept terms", value: { path: "/accepted" } }],
      { accepted: false },
    );

    render(<UarSurface surface={surface} />);
    await userEvent.click(screen.getByRole("checkbox", { name: "Accept terms" }));

    expect(surface.dataModel.get("/accepted")).toBe(true);
  });
});

describe("ChoicePicker", () => {
  it("single-select (mutuallyExclusive) writes the selected value back via setValue", async () => {
    const { surface } = buildSurface(
      uarBasicCatalog,
      [
        {
          id: "root",
          component: "ChoicePicker",
          label: "Color",
          variant: "mutuallyExclusive",
          options: [
            { label: "Red", value: "red" },
            { label: "Blue", value: "blue" },
          ],
          value: { path: "/color" },
        },
      ],
      { color: [] },
    );

    render(<UarSurface surface={surface} />);
    await userEvent.click(screen.getByRole("option", { name: "Blue" }));

    expect(surface.dataModel.get("/color")).toEqual(["blue"]);
    expect(screen.getByRole("option", { name: "Blue" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("option", { name: "Red" })).toHaveAttribute("aria-selected", "false");
  });

  it("multi-select renders all options with listbox semantics", () => {
    const { surface } = buildSurface(uarBasicCatalog, [
      {
        id: "root",
        component: "ChoicePicker",
        variant: "multipleSelection",
        options: [
          { label: "Red", value: "red" },
          { label: "Blue", value: "blue" },
          { label: "Green", value: "green" },
        ],
        value: ["red", "green"],
      },
    ]);

    render(<UarSurface surface={surface} />);
    const listbox = screen.getByRole("listbox");
    expect(listbox).toHaveAttribute("aria-multiselectable", "true");
    expect(screen.getAllByRole("option")).toHaveLength(3);
  });
});
