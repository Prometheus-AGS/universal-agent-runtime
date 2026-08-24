declare const graph: {
  upsertEntity(type: string, id: string, data: Record<string, unknown>): void;
};

export function publishRows(rows: Array<{ id: string }>) {
  rows.forEach((row) => graph.upsertEntity("ConfiguredModel", row.id, row));
}
