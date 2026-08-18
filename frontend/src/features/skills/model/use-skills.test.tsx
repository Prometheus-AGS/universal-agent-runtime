import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, test } from "vitest";
import { useGraphStore } from "@/platform/entities";
import { useSkills } from "./use-skills";

const review = {
  skill_id: "review",
  title: "Review",
  description: "Review a change",
  version: "1.0.0",
  triggers: { keywords: [] },
  preferred_tools: [],
  enabled: true,
};

beforeEach(() => {
  useGraphStore.setState({ entities: {} } as never);
});

describe("useSkills", () => {
  test("renders graph entities without a separate list index", () => {
    const { result } = renderHook(() => useSkills());

    act(() => {
      useGraphStore.getState().upsertEntity("Skill", review.skill_id, review);
    });

    expect(result.current.items).toEqual([review]);
  });

  test("filters graph entities by enabled state and text", () => {
    useGraphStore.getState().upsertEntity("Skill", review.skill_id, review);
    useGraphStore.getState().upsertEntity("Skill", "draft", {
      ...review,
      skill_id: "draft",
      title: "Draft",
      enabled: false,
    });

    const { result } = renderHook(() => useSkills("review", true));

    expect(result.current.items).toEqual([review]);
  });
});
