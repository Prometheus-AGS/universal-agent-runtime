import { useMemo } from "react";

import { SinglePicker } from "@/components/ui/single-picker";
import {
  useCatalogModels,
  useConfiguredModels,
  useModelCatalogLoadError,
  useModelCatalogLoaded,
} from "@/platform/entities";
import { catalogModelOptions, modelCountLabel, sourceStatus } from "./picker-options";

export interface CatalogModelPickerProps {
  /** The `provider/model` route, or an empty string when nothing is chosen. */
  value: string;
  onChange: (route: string) => void;
  id?: string;
  testId?: string;
  ariaLabel?: string;
  ariaDescribedBy?: string;
  ariaInvalid?: boolean;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
}

/**
 * Every catalog model across providers, grouped by provider, virtualized past
 * about 1,000 options. A model is selectable only when it is enabled on a
 * configured, enabled provider; the rest are listed but disabled with a reason.
 * The search count reads `<n> models`.
 */
export function CatalogModelPicker({
  value,
  onChange,
  id,
  testId,
  ariaLabel = "Model",
  ariaDescribedBy,
  ariaInvalid,
  placeholder = "Select a model",
  className,
  disabled,
}: CatalogModelPickerProps) {
  const catalog = useCatalogModels();
  const configured = useConfiguredModels();
  const loaded = useModelCatalogLoaded("models");
  const loadError = useModelCatalogLoadError("models");

  const selectable = useMemo(
    () => new Set(configured.items.map((model) => model.id)),
    [configured.items],
  );
  const options = useMemo(() => catalogModelOptions(catalog, selectable), [catalog, selectable]);
  const status = sourceStatus({
    loaded,
    error: loadError,
    count: catalog.length,
    pending: configured.isLoading && configured.items.length === 0,
  });
  return (
    <SinglePicker
      value={value}
      onValueChange={onChange}
      options={options}
      status={status}
      errorMessage={loadError ?? undefined}
      id={id}
      testId={testId}
      ariaLabel={ariaLabel}
      searchAriaLabel={`Search ${ariaLabel.toLowerCase()}`}
      ariaDescribedBy={ariaDescribedBy}
      ariaInvalid={ariaInvalid}
      disabled={disabled}
      className={className}
      placeholder={placeholder}
      searchPlaceholder="Search models…"
      emptyLabel="The model catalog is empty."
      noMatchLabel="No matching models."
      loadingLabel="Loading models…"
      countLabel={modelCountLabel}
    />
  );
}
