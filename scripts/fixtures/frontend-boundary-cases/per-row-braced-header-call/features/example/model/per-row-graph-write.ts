declare const graph: {
  upsertEntity(type: string, id: string, data: Record<string, unknown>): void;
};

export function publishRows(rows: Array<{ id: string }>, startAt: () => number) {
  for (let index = startAt(); index < rows.length; index += 1) {
    const row = rows[index];
    graph.upsertEntity("ConfiguredModel", row.id, row);
  }
}
