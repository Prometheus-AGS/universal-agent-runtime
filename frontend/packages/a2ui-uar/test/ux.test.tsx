import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { uarBasicCatalog } from "../src/catalog/uar-basic-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { UarTextField } from "../src/components/TextField";
import { UarCheckBox } from "../src/components/CheckBox";
import { UarChoicePicker } from "../src/components/ChoicePicker";
import { buildSurface } from "./helpers";

describe("surface UX contract", () => {
  it("exposes scoped high-contrast theme, locale, and explicit RTL direction", () => {
    const { surface } = buildSurface(uarBasicCatalog, [{ id: "root", component: "Text", text: "سلام" }]);
    const { container } = render(<UarSurface surface={surface} theme="high-contrast" locale="ja" direction="rtl" />);
    const root = container.querySelector(".uar-a2ui-surface");
    expect(root).toHaveAttribute("data-a2ui-theme", "high-contrast");
    expect(root).toHaveAttribute("lang", "ja");
    expect(root).toHaveAttribute("dir", "rtl");
  });

  it("localizes renderer-owned fallback copy without changing payload text", () => {
    const { surface } = buildSurface(uarBasicCatalog, [{ id: "root", component: "ChoicePicker", options: [{ label: "Rojo", value: "red" }] }]);
    render(<UarSurface surface={surface} locale="es" />);
    expect(screen.getByRole("listbox", { name: "Opciones" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Rojo" })).toBeInTheDocument();
  });

  it("renders a localized empty state instead of a blank region", () => {
    const { surface } = buildSurface(uarBasicCatalog, []);
    render(<UarSurface surface={surface} locale="zh" />);
    expect(screen.getByRole("status")).toHaveTextContent("此界面尚无内容");
  });

  it("resets and invokes retry after a contained render failure", async () => {
    const { surface } = buildSurface(uarBasicCatalog, [{ id: "root", component: "Unsupported" }]);
    const onRetry = vi.fn();
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    render(<UarSurface surface={surface} onRetry={onRetry} />);
    await userEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(onRetry).toHaveBeenCalledOnce();
    consoleError.mockRestore();
  });
});

describe("validation associations", () => {
  it("composes TextField help and error descriptions", () => {
    render(<UarTextField props={{ label: "Name", value: "", setValue: vi.fn(), isValid: false, validationErrors: ["Required"], accessibility: { description: "Public display name" } } as never} />);
    const input = screen.getByLabelText("Name");
    const ids = input.getAttribute("aria-describedby")?.split(" ") ?? [];
    expect(ids).toHaveLength(2);
    expect(ids.map((id) => document.getElementById(id)?.textContent)).toEqual(["Public display name", "Required"]);
  });

  it("associates CheckBox and ChoicePicker errors", () => {
    const { rerender } = render(<UarCheckBox props={{ label: "Approve", value: false, setValue: vi.fn(), isValid: false, validationErrors: ["Review required"] } as never} />);
    const checkbox = screen.getByRole("checkbox", { name: "Approve" });
    expect(document.getElementById(checkbox.getAttribute("aria-describedby") ?? "")).toHaveTextContent("Review required");
    rerender(<UarChoicePicker props={{ options: [{ label: "One", value: "one" }], value: [], setValue: vi.fn(), isValid: false, validationErrors: ["Choose one"] } as never} />);
    const listbox = screen.getByRole("listbox");
    expect(document.getElementById(listbox.getAttribute("aria-describedby") ?? "")).toHaveTextContent("Choose one");
  });
});
