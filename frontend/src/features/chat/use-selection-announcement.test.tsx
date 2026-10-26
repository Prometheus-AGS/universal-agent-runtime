import { renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useSelectionAnnouncement } from "./use-selection-announcement";

describe("routing announcement ownership", () => {
  it("announces once across remounts and keeps restored history silent", () => {
    const first = renderHook(() => useSelectionAnnouncement("fresh-child-once", true, "Registered agent"));
    expect(first.result.current).toBe("Selected agent: Registered agent");
    first.unmount();
    const replay = renderHook(() => useSelectionAnnouncement("fresh-child-once", true, "Registered agent"));
    expect(replay.result.current).toBe("");
    replay.unmount();
    const history = renderHook(() => useSelectionAnnouncement("restored-child", false, "Stored agent"));
    expect(history.result.current).toBe("");
  });
});
