import { useMemo } from "react";

import { SinglePicker } from "@/components/ui/single-picker";
import {
  useCatalogProviders,
  useModelCatalogLoadError,
  useModelCatalogLoaded,
} from "@/platform/entities";
import { providerCountLabel, providerOptions, sourceStatus } from "./picker-options";

export interface ProviderChoice {
  id: string;
  /** Display name. Falls back to the catalog name, then the id. */
  label?: string;
}

export interface ProviderPickerProps {
  /** The provider id, or an empty string when nothing is chosen. */
  value: string;
  onChange: (providerId: string) => void;
  /** Narrows the options, for example to the configured providers. Default: every catalog provider. */
  scope?: readonly ProviderChoice[];
  id?: string;
  testId?: string;
  ariaLabel?: string;
  ariaDescribedBy?: string;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
}

/** Chooses one provider; stores its id. Typing a name that is not an option selects nothing. */
export function ProviderPicker({
  value,
  onChange,
  scope,
  id,
  testId,
  ariaLabel = "Provider",
  ariaDescribedBy,
  placeholder = "Select a provider",
  className,
  disabled,
}: ProviderPickerProps) {
  const providers = useCatalogProviders();
  const loaded = useModelCatalogLoaded("providers");
  const loadError = useModelCatalogLoadError("providers");
  const options = useMemo(() => providerOptions(providers, scope), [providers, scope]);
  const status = scope
    ? "ready"
    : sourceStatus({ loaded, error: loadError, count: providers.length });
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
      disabled={disabled}
      className={className}
      placeholder={placeholder}
      searchPlaceholder="Search providers…"
      emptyLabel="No providers are available."
      noMatchLabel="No matching providers."
      loadingLabel="Loading providers…"
      countLabel={providerCountLabel}
    />
  );
}
