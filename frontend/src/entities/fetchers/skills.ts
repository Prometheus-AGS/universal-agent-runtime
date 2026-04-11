import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchSkillsList } from "@/services/skills-api";

/**
 * Fetch all skills and upsert each into the entity graph as a `Skill` entity.
 *
 * The API may return `skill_id` rather than `id`, so we normalise
 * to ensure every entity has a stable `id` field.
 */
export async function loadSkillsIntoGraph(): Promise<void> {
  const skills = await fetchSkillsList();
  const { upsertEntity } = useGraphStore.getState();

  for (const s of skills) {
    const id = s.skill_id ?? s.title;
    upsertEntity("Skill", id, { ...s, id } as unknown as Record<string, unknown>);
  }
}
