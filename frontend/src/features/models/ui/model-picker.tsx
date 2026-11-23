import { useMemo } from "react";

import { SinglePicker } from "@/components/ui/single-picker";
import { useConfiguredModels } from "@/platform/entities";
import { configuredModelOptions, modelCountLabel, sourceStatus } from "./picker-options";

export interface ModelPickerProps {
  /** The `provider/model` route, or an empty string when nothing is chosen. */
  value: string;
  onChange: (route: string) => void;
  id?: string;
  testId?: string;
  ariaLabel?: string;
  ariaDescribedBy?: string;
  /** Shown when the value is empty, for example "Use agent / global default". */
  defaultLabel?: string;
  placeholder?: string;
  className?: string;
  disabled?: boolean;
}

/**
 * Chooses one enabled model of a configured provider; stores its `provider/model`
 * route. Fewer than eight models render a simple select. Subscribes only to the
 * configured-model list.
 */
export function ModelPicker({
  value,
  onChange,
  id,
  testId,
  ariaLabel = "Model",
  ariaDescribedBy,
  defaultLabel,
  placeholder = "Select a model",
  className,
  disabled,
}: ModelPickerProps) {
  const { items, isLoading, error } = useConfiguredModels();
  const options = useMemo(() => configuredModelOptions(items), [items]);
  const errorMessage = error?.message ?? null;
  const status = sourceStatus({ loaded: !isLoading, error: errorMessage, count: items.length });
  return (
    <SinglePicker
      value={value}
      onValueChange={onChange}
      options={options}
      status={status}
      errorMessage={errorMessage ?? undefined}
      id={id}
      testId={testId}
      ariaLabel={ariaLabel}
      searchAriaLabel={`Search ${ariaLabel.toLowerCase()}`}
      ariaDescribedBy={ariaDescribedBy}
      disabled={disabled}
      className={className}
      placeholder={defaultLabel ?? placeholder}
      searchPlaceholder="Search models…"
      emptyLabel="No models are configured."
      noMatchLabel="No matching models."
      loadingLabel="Loading models…"
      countLabel={modelCountLabel}
      onClear={() => onChange("")}
      clearLabel="Clear model"
    />
  );
}
