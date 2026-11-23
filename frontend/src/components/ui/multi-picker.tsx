"use client"

import { ComboboxPicker } from "@/components/ui/combobox-picker-namespace"
import {
  defaultCountLabel,
  type PickerOption,
  type PickerStatus,
} from "@/components/ui/combobox-picker-model"

export interface MultiPickerProps {
  values: readonly string[]
  onValuesChange: (values: string[]) => void
  options: readonly PickerOption[]
  status?: PickerStatus
  errorMessage?: string
  id?: string
  testId?: string
  ariaLabel: string
  searchAriaLabel: string
  ariaDescribedBy?: string
  ariaInvalid?: boolean
  disabled?: boolean
  className?: string
  placeholder?: string
  emptyLabel?: string
  noMatchLabel?: string
  loadingLabel?: string
  searchPlaceholder?: string
  countLabel?: (count: number) => string
  unavailableText?: string | null
}

/** Picks any number of options from one searchable, keyboard-operable list. */
export function MultiPicker({
  values,
  onValuesChange,
  options,
  status = "ready",
  errorMessage,
  id,
  testId,
  ariaLabel,
  searchAriaLabel,
  ariaDescribedBy,
  ariaInvalid,
  disabled,
  className,
  placeholder,
  emptyLabel,
  noMatchLabel,
  loadingLabel,
  searchPlaceholder,
  countLabel = defaultCountLabel,
  unavailableText,
}: MultiPickerProps) {
  return (
    <ComboboxPicker.Root
      options={options}
      selection={{ kind: "multiple", values, onValuesChange }}
      status={status}
      errorMessage={errorMessage}
      countLabel={countLabel}
      placeholder={placeholder}
      emptyLabel={emptyLabel}
      noMatchLabel={noMatchLabel}
      loadingLabel={loadingLabel}
      unavailableText={unavailableText}
      disabled={disabled}
    >
      <ComboboxPicker.Trigger
        id={id}
        data-testid={testId}
        aria-label={ariaLabel}
        aria-describedby={ariaDescribedBy}
        invalid={ariaInvalid}
        className={className}
      />
      <ComboboxPicker.Content>
        <ComboboxPicker.Search aria-label={searchAriaLabel} placeholder={searchPlaceholder} />
        <ComboboxPicker.Status />
        <ComboboxPicker.Empty />
        <ComboboxPicker.List />
      </ComboboxPicker.Content>
    </ComboboxPicker.Root>
  )
}
