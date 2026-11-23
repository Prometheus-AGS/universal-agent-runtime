import type { PickerOption, PickerStatus } from "@/components/ui/combobox-picker-model";
import { catalogModelId } from "@/platform/entities";
import type { CatalogModel, ConfiguredModel, Provider } from "@/platform/entities";

const MODEL_OPTION_TEST_ID = "model-option";

/** Short context-window badge, for example `128k`. */
export function formatContext(tokens: number | null | undefined): string | undefined {
  if (typeof tokens !== "number" || !Number.isFinite(tokens) || tokens <= 0) return undefined;
  if (tokens >= 1_000_000) return `${Math.round(tokens / 1_000_000)}M`;
  if (tokens >= 1_000) return `${Math.round(tokens / 1_000)}k`;
  return String(tokens);
}

/** Raw-identifier order (code units), the order the frozen picker oracle lists. */
function compareRaw(left: string, right: string): number {
  if (left < right) return -1;
  return left > right ? 1 : 0;
}

export function modelCountLabel(count: number): string {
  return `${count} ${count === 1 ? "model" : "models"}`;
}

export function providerCountLabel(count: number): string {
  return `${count} ${count === 1 ? "provider" : "providers"}`;
}

/** Enabled models of enabled providers, grouped by provider. The value is the `provider/model` route. */
export function configuredModelOptions(models: readonly ConfiguredModel[]): PickerOption[] {
  return [...models]
    .sort(
      (a, b) =>
        a.provider_name.localeCompare(b.provider_name) ||
        a.display_name.localeCompare(b.display_name),
    )
    .map((model) => ({
      value: model.id,
      label: model.display_name,
      detail: model.model_id,
      badge: formatContext(model.context_window),
      groupId: model.provider_id,
      groupLabel: model.provider_name,
      testId: MODEL_OPTION_TEST_ID,
      modelId: model.id,
    }));
}

/**
 * Catalog providers, optionally narrowed to an explicit scope. A scoped id the
 * catalog does not know still appears, named by the scope entry or its id.
 */
export function providerOptions(
  providers: readonly Provider[],
  scope?: readonly { id: string; label?: string }[],
): PickerOption[] {
  if (!scope) {
    return providers.map((provider) => ({
      value: provider.id,
      label: provider.display_name || provider.id,
      detail: provider.id,
    }));
  }
  const byId = new Map(providers.map((provider) => [provider.id, provider]));
  return scope.map((entry) => {
    const known = byId.get(entry.id);
    return {
      value: entry.id,
      label: entry.label ?? known?.display_name ?? entry.id,
      detail: entry.id,
    };
  });
}

/** One provider's catalog models in raw-id order. `excluded` raw ids are left out. */
export function providerModelOptions(
  models: readonly CatalogModel[],
  excluded?: ReadonlySet<string>,
): PickerOption[] {
  return models
    .filter((model) => !excluded?.has(model.model_id))
    .sort((a, b) => compareRaw(a.model_id, b.model_id))
    .map((model) => ({
      value: model.model_id,
      label: model.name || model.model_id,
      detail: model.model_id,
      badge: formatContext(model.context),
      testId: MODEL_OPTION_TEST_ID,
      modelId: model.id,
    }));
}

/** Plain `{ id, label }` models, for callers whose list comes from a settings draft. */
export function suppliedModelOptions(
  providerId: string,
  models: readonly { id: string; label?: string }[],
): PickerOption[] {
  return models.map((model) => ({
    value: model.id,
    label: model.label || model.id,
    detail: model.id,
    testId: MODEL_OPTION_TEST_ID,
    modelId: catalogModelId(providerId, model.id),
  }));
}

/**
 * Every catalog model grouped by provider. A model is selectable only when it
 * is enabled on a configured, enabled provider (`selectable` holds those routes).
 */
export function catalogModelOptions(
  models: readonly CatalogModel[],
  selectable: ReadonlySet<string>,
): PickerOption[] {
  return [...models]
    .sort(
      (a, b) =>
        compareRaw(a.provider_name, b.provider_name) || compareRaw(a.model_id, b.model_id),
    )
    .map((model) => {
      const enabled = selectable.has(model.id);
      return {
        value: model.id,
        label: model.name || model.model_id,
        detail: model.model_id,
        badge: formatContext(model.context),
        groupId: model.provider_id,
        groupLabel: model.provider_name,
        testId: MODEL_OPTION_TEST_ID,
        modelId: model.id,
        disabled: !enabled,
        disabledReason: enabled
          ? undefined
          : model.provider_configured
            ? "Not enabled"
            : "Provider not configured",
      };
    });
}

/**
 * Load status from graph facts. A failed refresh that left rows in the graph
 * stays ready; a first load with no rows and no error is still loading.
 */
export function sourceStatus(input: {
  loaded: boolean;
  error: string | null;
  count: number;
  pending?: boolean;
}): PickerStatus {
  if (input.count > 0 && !input.pending) return "ready";
  if (input.error) return "error";
  if (input.loaded && !input.pending) return "ready";
  return "loading";
}
