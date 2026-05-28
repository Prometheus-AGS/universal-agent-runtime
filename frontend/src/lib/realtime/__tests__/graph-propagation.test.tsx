/**
 * Contract test — graph propagation.
 *
 * Two components reading the same `Provider:p1` entity from the graph
 * must both re-render when the graph mutates. This is the foundation of
 * the "no stale data anywhere" guarantee for every direct-`useEntity*`
 * consumer in the SPA.
 */
import { act, render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";

function Reader({ id, tag }: { id: string; tag: string }) {
  const entity = useGraphStore(
    (s) =>
      s.entities["Provider"]?.[id] as { display_name?: string } | undefined,
  );
  return <span data-testid={tag}>{entity?.display_name ?? "—"}</span>;
}

describe("graph propagation", () => {
  test("upserting an entity re-renders every subscribed consumer", () => {
    render(
      <>
        <Reader id="p1" tag="a" />
        <Reader id="p1" tag="b" />
      </>,
    );
    // Pre-mutation: both readers show the empty-state value.
    expect(screen.getByTestId("a").textContent).toBe("—");
    expect(screen.getByTestId("b").textContent).toBe("—");

    act(() => {
      useGraphStore.getState().upsertEntity("Provider", "p1", {
        id: "p1",
        display_name: "Alpha",
      });
    });

    expect(screen.getByTestId("a").textContent).toBe("Alpha");
    expect(screen.getByTestId("b").textContent).toBe("Alpha");
  });

  test("updating an entity flows to every subscribed consumer", () => {
    render(
      <>
        <Reader id="p2" tag="a" />
        <Reader id="p2" tag="b" />
      </>,
    );

    act(() => {
      useGraphStore
        .getState()
        .upsertEntity("Provider", "p2", { id: "p2", display_name: "Bravo" });
    });
    expect(screen.getByTestId("a").textContent).toBe("Bravo");

    act(() => {
      useGraphStore.getState().upsertEntity("Provider", "p2", {
        id: "p2",
        display_name: "Bravo Renamed",
      });
    });

    expect(screen.getByTestId("a").textContent).toBe("Bravo Renamed");
    expect(screen.getByTestId("b").textContent).toBe("Bravo Renamed");
  });

  test("removing an entity restores the empty-state for every consumer", () => {
    render(
      <>
        <Reader id="p3" tag="a" />
        <Reader id="p3" tag="b" />
      </>,
    );

    act(() => {
      useGraphStore
        .getState()
        .upsertEntity("Provider", "p3", { id: "p3", display_name: "Charlie" });
    });
    expect(screen.getByTestId("a").textContent).toBe("Charlie");

    act(() => {
      useGraphStore.getState().removeEntity("Provider", "p3");
    });

    expect(screen.getByTestId("a").textContent).toBe("—");
    expect(screen.getByTestId("b").textContent).toBe("—");
  });
});
