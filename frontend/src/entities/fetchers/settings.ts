import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { fetchSettingsNamespace } from "@/services/settings-api";

/**
 * Fetch all settings within a namespace and upsert each into the entity
 * graph as a `Setting` entity. Entity id format: `<namespace>:<key>`
 * (matches the natural unique identifier used by the settings API).
 *
 * Also upserts a singleton `SettingsNamespace:<namespace>` row containing
 * the list of ids in that namespace, so consumers can derive a namespace
 * view with a single graph subscription.
 */
export async function loadSettingsIntoGraph(namespace: string): Promise<void> {
  const records = await fetchSettingsNamespace(namespace);
  const { upsertEntity } = useGraphStore.getState();

  const ids: string[] = [];
  for (const setting of records) {
    const id = `${namespace}:${setting.key}`;
    ids.push(id);
    upsertEntity("Setting", id, {
      id,
      namespace,
      ...(setting as unknown as Record<string, unknown>),
    });
  }

  upsertEntity("SettingsNamespace", namespace, {
    id: namespace,
    namespace,
    ids,
  });
}
