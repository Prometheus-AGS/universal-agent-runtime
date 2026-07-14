import { describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { uarEntityCatalog } from "../src/catalog/uar-entity-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

describe("EntityCard", () => {
  it("renders title, subtitle, fields, sync-origin badge, and dispatches an action on click", async () => {
    const { surface } = buildSurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityCard",
        entityType: "Order",
        entityId: "order-123",
        title: "Order #123",
        subtitle: "Placed 2026-07-10",
        syncOrigin: "optimistic",
        fields: [
          { label: "Status", value: "Pending" },
          { label: "Total", value: "$42.00" },
        ],
        actions: [{ label: "Cancel", action: { event: { name: "cancelOrder", context: {} } } }],
      },
    ]);

    const onAction = vi.fn();
    surface.onAction.subscribe(onAction);

    render(<UarSurface surface={surface} />);

    const card = document.querySelector('[data-a2ui-component="EntityCard"]');
    expect(card).toHaveAttribute("data-entity-type", "Order");
    expect(card).toHaveAttribute("data-entity-id", "order-123");
    expect(screen.getByText("Order #123")).toBeInTheDocument();
    expect(screen.getByText("Placed 2026-07-10")).toBeInTheDocument();
    expect(screen.getAllByText("Pending").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("$42.00")).toBeInTheDocument();
    expect(document.querySelector('[data-a2ui-entity-sync-origin="optimistic"]')).toHaveTextContent(
      "Pending",
    );

    await userEvent.click(screen.getByText("Cancel"));
    expect(onAction).toHaveBeenCalledTimes(1);
    expect(onAction.mock.calls[0][0]).toMatchObject({ name: "cancelOrder" });
  });

  it("renders with no fields/actions/subtitle without crashing", () => {
    const { surface } = buildSurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityCard",
        entityType: "Note",
        entityId: "note-1",
        title: "A bare card",
      },
    ]);

    render(<UarSurface surface={surface} />);
    expect(screen.getByText("A bare card")).toBeInTheDocument();
  });
});
