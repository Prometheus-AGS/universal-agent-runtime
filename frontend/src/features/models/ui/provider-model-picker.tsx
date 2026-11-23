import { useMemo } from "react";

import { SinglePicker, type SinglePickerProps } from "@/components/ui/single-picker";
import {
  useCatalogModelsForProvider,
  useModelCatalogLoadError,
  useModelCatalogLoaded,
} from "@/platform/entities";
import {
  modelCountLabel,
  providerModelOptions,
  sourceStatus,
  suppliedModelOptions,
} from "./picker-options";

export interface SuppliedModel {
  id: string;
  label?: string;
}

export interface ProviderModelPickerProps {
  /** The provider whose models are offered. Empty means no provider is chosen yet. */
  providerId: string;
  /** The raw model id, or an empty string when nothing is chosen. */
  value: string;
  onChange: (modelId: string) => void;
  /**
   * Offer exactly these models instead of the provider's catalog models, for a
   * list that comes from a settings draft. Fixed for the life of the instance.
   */
  models?: readonly SuppliedModel[];
  /** Raw model ids left out of the catalog options, for example models already enabled. */
  excludeModelIds?: readonly string[];
  id?: string;
  testId?: string;
  ariaLabel?: string;
  searchAriaLabel?: string;
  ariaDescribedBy?: string;
  ariaInvalid?: boolean;
  placeholder?: string;
  emptyLabel?: string;
  /** Recovery text for an unavailable stored value; null when the caller describes it. */
  unavailableText?: string | null;
  className?: string;
  disabled?: boolean;
}

type SharedProps = Omit<ProviderModelPickerProps, "models" | "excludeModelIds" | "providerId">;

function pickerProps(
  props: SharedProps,
  status: SinglePickerProps["status"],
  errorMessage?: string,
): Omit<SinglePickerProps, "options"> {
  const ariaLabel = props.ariaLabel ?? "Model";
  return {
    value: props.value,
    onValueChange: props.onChange,
    status,
    errorMessage,
    id: props.id,
    testId: props.testId,
    ariaLabel,
    searchAriaLabel: props.searchAriaLabel ?? `Search ${ariaLabel.toLowerCase()}`,
    ariaDescribedBy: props.ariaDescribedBy,
    ariaInvalid: props.ariaInvalid,
    disabled: props.disabled,
    className: props.className,
    placeholder: props.placeholder ?? "Select a model",
    emptyLabel: props.emptyLabel ?? "No models are available for this provider.",
    noMatchLabel: "No matching models.",
    loadingLabel: "Loading models…",
    searchPlaceholder: "Search models…",
    countLabel: modelCountLabel,
    unavailableText: props.unavailableText,
  };
}

/**
 * Chooses one model of a fixed provider; stores the raw model id. Options are the
 * provider's catalog models unless `models` supplies them. Search matches display
 * name and raw id.
 */
export function ProviderModelPicker(props: ProviderModelPickerProps) {
  const { providerId, models, excludeModelIds, ...shared } = props;
  if (models) {
    return <SuppliedProviderModelPicker providerId={providerId} models={models} {...shared} />;
  }
  return (
    <CatalogProviderModelPicker
      providerId={providerId}
      excludeModelIds={excludeModelIds}
      {...shared}
    />
  );
}

function SuppliedProviderModelPicker({
  providerId,
  models,
  ...shared
}: SharedProps & { providerId: string; models: readonly SuppliedModel[] }) {
  const options = useMemo(() => suppliedModelOptions(providerId, models), [providerId, models]);
  return <SinglePicker {...pickerProps(shared, "ready")} options={options} />;
}

function CatalogProviderModelPicker({
  providerId,
  excludeModelIds,
  ...shared
}: SharedProps & { providerId: string; excludeModelIds?: readonly string[] }) {
  const catalogModels = useCatalogModelsForProvider(providerId);
  const loaded = useModelCatalogLoaded("models");
  const loadError = useModelCatalogLoadError("models");
  const options = useMemo(
    () => providerModelOptions(catalogModels, excludeModelIds ? new Set(excludeModelIds) : undefined),
    [catalogModels, excludeModelIds],
  );
  const status = providerId
    ? sourceStatus({ loaded, error: loadError, count: catalogModels.length })
    : "ready";
  return (
    <SinglePicker
      {...pickerProps(
        {
          ...shared,
          placeholder: providerId ? shared.placeholder : "Select a provider first",
          emptyLabel: providerId ? shared.emptyLabel : "Select a provider first",
          disabled: shared.disabled || !providerId,
        },
        status,
        loadError ?? undefined,
      )}
      options={options}
    />
  );
}
