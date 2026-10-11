import { useEntities } from "@prometheus-ags/prometheus-entity-management";

export function GraphReader() {
  const entities = useEntities("Provider");
  return <span>{String(entities)}</span>;
}
