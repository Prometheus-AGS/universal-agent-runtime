import { useMemo } from "react";
import { useEntityView, useGraphStore } from "@/platform/entities";
import type { FilterClause, ViewDescriptor } from "@/platform/entities";
import type { SkillEntity } from "@/entities/types";

const EMPTY_SKILLS: SkillEntity[] = [];

/**
 * Live, filterable view of all Skill entities in the graph.
 *
 * Sorted alphabetically by title.
 * Supports optional free-text search across title and description,
 * and optional filtering by enabled status.
 */
export function useSkills(searchTerm?: string, enabledFilter?: boolean) {
  const filter = useMemo<FilterClause[] | undefined>(() => {
    if (enabledFilter === undefined) return undefined;
    return [{ field: "enabled", op: "eq" as const, value: enabledFilter }];
  }, [enabledFilter]);

  const view = useMemo<ViewDescriptor>(() => {
    const baseFilter: FilterClause[] = filter ? [...filter] : [];

    return {
      filter: baseFilter.length > 0 ? baseFilter : undefined,
      sort: [{ field: "title", direction: "asc" as const }],
      search:
        searchTerm && searchTerm.length > 0
          ? { query: searchTerm, fields: ["title", "description"], minChars: 1 }
          : undefined,
    };
  }, [searchTerm, filter]);

  return useEntityView<SkillEntity>({
    type: "Skill",
    baseQueryKey: [
      "skills",
      searchTerm ?? "",
      enabledFilter === undefined ? "" : String(enabledFilter),
    ],
    view,
    mode: "local",
  });
}

/**
 * Returns a single Skill entity by id, read directly from the graph.
 *
 * Uses `useGraphStore` selector so the component re-renders only when
 * that skill's data changes. Returns null when the skill is not loaded.
 */
export function useSkill(id: string | undefined): SkillEntity | null {
  return useGraphStore((state) => {
    if (!id) return null;

    const skillMap = state.entities["Skill"];
    if (!skillMap) return null;

    const entity = skillMap[id];
    if (!entity) return null;

    return entity as unknown as SkillEntity;
  });
}

/**
 * Returns all enabled Skill entities, read directly from the graph.
 *
 * Returns a stable empty array when no skills match (avoids the Zustand
 * infinite-render bug).
 */
export function useEnabledSkills(): SkillEntity[] {
  return useGraphStore((state) => {
    const skillMap = state.entities["Skill"];
    if (!skillMap) return EMPTY_SKILLS;

    const results: SkillEntity[] = [];
    for (const id of Object.keys(skillMap)) {
      const entity = skillMap[id];
      if (entity && (entity as Record<string, unknown>)["enabled"] === true) {
        results.push(entity as unknown as SkillEntity);
      }
    }

    return results.length > 0 ? results : EMPTY_SKILLS;
  });
}
