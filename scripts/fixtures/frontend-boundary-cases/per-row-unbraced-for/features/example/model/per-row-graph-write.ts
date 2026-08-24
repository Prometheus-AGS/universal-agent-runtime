declare const graph: {
  upsertEntity(type: string, id: string, data: Record<string, unknown>): void;
};

export function publishRows(rows: Array<{ id: string }>) {
  for (const row of rows) graph.upsertEntity("ConfiguredModel", row.id, row);
}
