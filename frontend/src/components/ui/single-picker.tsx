"use client"

import * as React from "react"

import { cn } from "@/lib/utils"
import {
  DEFAULT_RECOVERY_TEXT,
  PickerClearButton,
} from "@/components/ui/combobox-picker"
import { ComboboxPicker } from "@/components/ui/combobox-picker-namespace"
import {
  defaultCountLabel,
  isUnavailableValue,
  optionMarkers,
  pickerRenderMode,
  type PickerOption,
  type PickerStatus,
} from "@/components/ui/combobox-picker-model"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

export interface SinglePickerProps {
  /** The stored value; empty string means nothing is chosen. */
  value: string
  onValueChange: (value: string) => void
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
  /** Recovery text for an unavailable stored value; null when the caller describes it. */
  unavailableText?: string | null
  /** When set, a clear button calls it. */
  onClear?: () => void
  clearLabel?: string
}

/**
 * Picks one option. Fewer than eight options (and a ready list) render a
 * simple select; everything else renders the searchable combobox, virtualized
 * above about 1,000 options.
 */
export function SinglePicker(props: SinglePickerProps) {
  const status = props.status ?? "ready"
  if (pickerRenderMode(props.options.length, status) === "select") {
    return <SimpleSelectPicker {...props} />
  }
  return <SearchableSinglePicker {...props} status={status} />
}

function SearchableSinglePicker({
  value,
  onValueChange,
  options,
  status,
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
  onClear,
  clearLabel,
}: SinglePickerProps) {
  return (
    <ComboboxPicker.Root
      options={options}
      selection={{ kind: "single", value, onValueChange }}
      status={status}
      errorMessage={errorMessage}
      countLabel={countLabel}
      placeholder={placeholder}
      emptyLabel={emptyLabel}
      noMatchLabel={noMatchLabel}
      loadingLabel={loadingLabel}
      unavailableText={unavailableText}
      disabled={disabled}
      onClear={onClear}
      clearLabel={clearLabel}
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

function SimpleSelectPicker({
  value,
  onValueChange,
  options,
  id,
  testId,
  ariaLabel,
  ariaDescribedBy,
  ariaInvalid,
  disabled,
  className,
  placeholder = "Select an option",
  emptyLabel,
  unavailableText = DEFAULT_RECOVERY_TEXT,
  onClear,
  clearLabel = "Clear selection",
}: SinglePickerProps) {
  const recoveryId = React.useId()
  const unavailable = isUnavailableValue(value, options, "ready")
  const items = React.useMemo(
    () => options.map((option) => ({ value: option.value, label: option.label })),
    [options],
  )
  const showRecovery = unavailable && unavailableText !== null
  const description =
    [ariaDescribedBy, showRecovery ? recoveryId : undefined].filter(Boolean).join(" ") ||
    undefined
  const clearable = onClear !== undefined && value !== "" && !disabled
  return (
    <div className="relative w-full">
      <Select
        items={items}
        value={unavailable ? "" : value}
        onValueChange={(next) => {
          if (next != null) onValueChange(next)
        }}
        disabled={disabled}
      >
        <SelectTrigger
          id={id}
          data-testid={testId}
          aria-label={ariaLabel}
          aria-invalid={ariaInvalid || unavailable || undefined}
          aria-describedby={description}
          className={cn("w-full font-mono text-xs", clearable && "pr-14", className)}
        >
          {unavailable ? (
            <span className="min-w-0 flex-1 truncate text-left text-destructive">
              {`Unavailable: ${value}`}
            </span>
          ) : (
            <SelectValue
              placeholder={options.length === 0 && emptyLabel ? emptyLabel : placeholder}
            />
          )}
        </SelectTrigger>
        <SelectContent>
          {options.map((option) => (
            <SelectItem
              key={option.value}
              value={option.value}
              disabled={option.disabled}
              className="font-mono text-xs"
              {...optionMarkers(option)}
            >
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      {clearable && <PickerClearButton label={clearLabel} onClear={onClear} />}
      {showRecovery && (
        <span id={recoveryId} className="sr-only">
          {unavailableText}
        </span>
      )}
    </div>
  )
}
