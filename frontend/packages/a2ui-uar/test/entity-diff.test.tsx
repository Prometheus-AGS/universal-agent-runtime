import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { uarEntityCatalog } from "../src/catalog/uar-entity-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

describe("EntityDiff", () => {
  it("renders title and highlights changed fields differently from unchanged ones", () => {
    const { surface } = buildSurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityDiff",
        entityType: "Order",
        entityId: "order-123",
        title: "Order #123 updated",
        fields: [
          { label: "Status", before: "Pending", after: "Shipped" },
          { label: "Total", before: "$42.00", after: "$42.00" },
        ],
      },
    ]);

    render(<UarSurface surface={surface} />);

    const panel = document.querySelector('[data-a2ui-component="EntityDiff"]');
    expect(panel).toHaveAttribute("data-entity-type", "Order");
    expect(panel).toHaveAttribute("data-entity-id", "order-123");
    expect(screen.getByText("Order #123 updated")).toBeInTheDocument();

    const changedAfter = document.querySelector('[data-a2ui-diff-changed="true"]');
    expect(changedAfter).toHaveTextContent("Shipped");

    const unchangedRows = document.querySelectorAll('[data-a2ui-diff-changed="false"]');
    expect(unchangedRows.length).toBeGreaterThanOrEqual(1);
  });

  it("renders with no fields without crashing", () => {
    const { surface } = buildSurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityDiff",
        entityType: "Note",
        entityId: "note-1",
        title: "No changes yet",
        fields: [],
      },
    ]);

    render(<UarSurface surface={surface} />);
    expect(screen.getByText("No changes yet")).toBeInTheDocument();
  });
});
