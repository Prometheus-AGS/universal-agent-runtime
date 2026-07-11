import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { A2uiComponent } from "@/features/a2ui/a2ui-protocol";
import { A2uiSurfaceRenderer } from "@/features/a2ui/a2ui-surface-renderer";

const components: A2uiComponent[] = [
  { id: "heading", component: "Text", text: { path: "/title" }, variant: "h1" },
  { id: "name", component: "TextField", label: "Name", value: { path: "/name" } },
  { id: "submit-label", component: "Text", text: "Continue" },
  {
    id: "submit",
    component: "Button",
    child: "submit-label",
    action: { event: { name: "continue", context: { source: "test" } } },
  },
  { id: "root", component: "Column", children: ["heading", "name", "submit"] },
];

describe("A2uiSurfaceRenderer", () => {
  test("renders bindings and forwards typed data/action intent", () => {
    const onDataChange = vi.fn();
    const onAction = vi.fn();
    render(
      <A2uiSurfaceRenderer
        components={components}
        data={{ title: "Profile", name: "Ada" }}
        onDataChange={onDataChange}
        onAction={onAction}
      />,
    );

    expect(screen.getByRole("heading", { name: "Profile" })).toBeVisible();
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Grace" } });
    expect(onDataChange).toHaveBeenCalledWith("/name", "Grace");
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    expect(onAction).toHaveBeenCalledWith("continue", { source: "test" });
  });

  test("reports missing references and cycles without executing anything", () => {
    const invalid: A2uiComponent[] = [
      { id: "root", component: "Column", children: ["missing", "root"] },
    ];
    render(
      <A2uiSurfaceRenderer
        components={invalid}
        data={{}}
        onDataChange={() => undefined}
        onAction={() => undefined}
      />,
    );
    expect(screen.getByText(/missing is unavailable/i)).toBeVisible();
    expect(screen.getByText(/cycle detected/i)).toBeVisible();
  });
});
