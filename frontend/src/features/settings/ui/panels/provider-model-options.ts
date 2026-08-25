export function getProviderModelOptions(data: Record<string, unknown>) {
  if (!Array.isArray(data.models)) return [];
  const seenModelIds = new Set<string>();

  return data.models.flatMap((model) => {
    if (
      typeof model !== "object" ||
      model === null ||
      !("id" in model) ||
      typeof model.id !== "string" ||
      model.id.length === 0 ||
      ("enabled" in model && model.enabled === false) ||
      seenModelIds.has(model.id)
    ) {
      return [];
    }
    seenModelIds.add(model.id);

    const displayName =
      "display_name" in model &&
      typeof model.display_name === "string" &&
      model.display_name.length > 0
        ? model.display_name
        : model.id;

    return [{ value: model.id, label: displayName }];
  });
}
