import { describe, expect, it } from "vitest";
import { act } from "@testing-library/react";
import { createRoot } from "react-dom/client";
import { uarBasicCatalog } from "../../src/catalog/uar-basic-catalog";
import { UarSurface } from "../../src/react/UarSurface";
import { measure, measureMany, percentile } from "../../src/perf/measure";
import { buildSurface } from "../helpers";

/**
 * Change 17's stated performance budget:
 *   - initial render < 16ms (one 60fps frame)
 *   - streaming chunk (an `updateComponents`/`updateDataModel` message
 *     applied to an already-mounted surface) < 8ms
 *
 * This is the measurement harness, run as `pnpm --filter
 * @prometheus-ags/a2ui-uar run perf` (see vitest.perf.config.ts). It is
 * CI executes these literal budgets as a regression gate.
 */

const CI_INITIAL_RENDER_BUDGET_MS = 16;
const CI_STREAMING_UPDATE_BUDGET_MS = 8;

function moderateSurfaceMessages() {
  return [
    { id: "root", component: "Column", children: ["row1", "row2", "row3"] },
    { id: "row1", component: "Row", children: ["card1"] },
    { id: "card1", component: "Card", child: "cardText" },
    { id: "cardText", component: "Text", text: "Card content" },
    { id: "row2", component: "Row", children: ["field", "checkbox"] },
    { id: "field", component: "TextField", label: "Name", value: { path: "/name" } },
    { id: "checkbox", component: "CheckBox", label: "Agree", value: { path: "/agree" } },
    { id: "row3", component: "Row", children: ["btnText"] },
    { id: "btnText", component: "Text", text: "Footer" },
  ];
}

describe("performance budget (measurement harness)", () => {
  it("initial render completes within one frame", () => {
    const { surface } = buildSurface(uarBasicCatalog, moderateSurfaceMessages(), {
      name: "",
      agree: false,
    });

    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    // Exclude one-time happy-dom/React JIT initialization from the renderer
    // budget; production processes initialize the engine before surfaces arrive.
    act(() => {
      root.render(<UarSurface surface={surface} />);
    });
    act(() => root.unmount());
    const measuredRoot = createRoot(container);
    const { durationMs } = measure(() => {
      act(() => {
        measuredRoot.render(<UarSurface surface={surface} />);
      });
    });

    act(() => {
      measuredRoot.unmount();
    });
    container.remove();

    expect(durationMs).toBeLessThan(CI_INITIAL_RENDER_BUDGET_MS);
  });

  it("a streaming data-model update re-renders within half a frame", () => {
    const { surface } = buildSurface(
      uarBasicCatalog,
      [{ id: "root", component: "Text", text: { path: "/greeting" } }],
      { greeting: "Initial" },
    );

    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);
    act(() => {
      root.render(<UarSurface surface={surface} />);
    });

    const durations = measureMany(() => {
      act(() => {
        surface.dataModel.set("/greeting", `Update ${Math.random()}`);
      });
    }, 20).map((r) => r.durationMs);

    act(() => {
      root.unmount();
    });
    container.remove();

    const p95 = percentile(durations, 95);
    expect(p95).toBeLessThan(CI_STREAMING_UPDATE_BUDGET_MS);
  });
});
