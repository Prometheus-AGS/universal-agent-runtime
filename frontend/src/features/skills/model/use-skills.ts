import { useMemo } from "react";
import { useGraphStore } from "@/platform/entities";
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
  const skillMap = useGraphStore((state) => state.entities["Skill"]);

  const items = useMemo(() => {
    const needle = searchTerm?.trim().toLowerCase();
    const all = Object.values(skillMap ?? {}) as unknown as SkillEntity[];

    return all
      .filter((skill) => {
        if (enabledFilter !== undefined && skill.enabled !== enabledFilter) return false;
        if (!needle) return true;
        return skill.title.toLowerCase().includes(needle)
          || skill.description.toLowerCase().includes(needle);
      })
      .sort((a, b) => a.title.localeCompare(b.title));
  }, [skillMap, searchTerm, enabledFilter]);

  return { items: items.length > 0 ? items : EMPTY_SKILLS };
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
