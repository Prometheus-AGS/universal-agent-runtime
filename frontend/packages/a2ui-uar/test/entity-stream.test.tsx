import { act } from "react";
import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import { uarEntityCatalog } from "../src/catalog/uar-entity-catalog";
import { UarSurface } from "../src/react/UarSurface";
import { buildSurface } from "./helpers";

describe("EntityStream", () => {
  it("renders items already present at its source path on mount", () => {
    const { surface } = buildSurface(
      uarEntityCatalog,
      [
        {
          id: "root",
          component: "EntityStream",
          entityType: "ToolCall",
          source: { path: "/toolCalls" },
          title: "Tool calls",
        },
      ],
      {
        toolCalls: [
          { id: "call-1", label: "search_web", value: "3 results" },
          { id: "call-2", label: "read_file" },
        ],
      },
    );

    render(<UarSurface surface={surface} />);

    const stream = document.querySelector('[data-a2ui-component="EntityStream"]');
    expect(stream).toHaveAttribute("data-entity-type", "ToolCall");
    expect(screen.getByText("Tool calls")).toBeInTheDocument();
    expect(screen.getByText("search_web")).toBeInTheDocument();
    expect(screen.getByText("3 results")).toBeInTheDocument();
    expect(screen.getByText("read_file")).toBeInTheDocument();
    expect(document.querySelector('[data-a2ui-stream-count="2"]')).not.toBeNull();
  });

  it("reactively appends a new item when the data model updates after mount", () => {
    const { processor, surface } = buildSurface(
      uarEntityCatalog,
      [
        {
          id: "root",
          component: "EntityStream",
          entityType: "ToolCall",
          source: { path: "/toolCalls" },
        },
      ],
      { toolCalls: [{ id: "call-1", label: "search_web" }] },
    );

    render(<UarSurface surface={surface} />);
    expect(screen.getByText("search_web")).toBeInTheDocument();

    act(() => {
      processor.processMessages([
        {
          version: "v0.9",
          updateDataModel: {
            surfaceId: "test-surface",
            path: "/toolCalls",
            value: [
              { id: "call-1", label: "search_web" },
              { id: "call-2", label: "read_file" },
            ],
          },
        },
      ]);
    });

    expect(screen.getByText("read_file")).toBeInTheDocument();
  });

  it("renders an empty state when the source path has no items yet", () => {
    const { surface } = buildSurface(uarEntityCatalog, [
      {
        id: "root",
        component: "EntityStream",
        entityType: "ToolCall",
        source: { path: "/toolCalls" },
      },
    ]);

    render(<UarSurface surface={surface} />);
    expect(screen.getByText("No items yet.")).toBeInTheDocument();
  });
});
