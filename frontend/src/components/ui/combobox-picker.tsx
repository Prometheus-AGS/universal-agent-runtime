"use client"

import * as React from "react"
import { useVirtualizer } from "@tanstack/react-virtual"
import { X } from "lucide-react"

import { cn } from "@/lib/utils"
import {
  Combobox,
  ComboboxCollection,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxGroup,
  ComboboxInput,
  ComboboxItem,
  ComboboxLabel,
  ComboboxList,
  ComboboxStatus,
  ComboboxTrigger,
} from "@/components/ui/combobox"
import {
  GROUP_ROW_HEIGHT_PX,
  OPTION_ROW_HEIGHT_PX,
  buildVirtualRows,
  defaultCountLabel,
  filterPickerOptions,
  flattenGroups,
  groupPickerOptions,
  pickerRenderMode,
  type PickerGroup,
  type PickerOption,
  type PickerRenderMode,
  optionMarkers,
  type PickerStatus,
  type VirtualRow,
} from "@/components/ui/combobox-picker-model"

/**
 * Compound searchable picker over the Base UI Combobox wrapper.
 *
 * Parts: Root, Trigger, Content, Search, Status, Empty, List, Group, Item.
 * `SinglePicker` and `MultiPicker` compose them; there are no mode flags.
 *
 * Virtualizer attachment (Base UI 1.9.0): the root receives `virtualized`, so
 * items skip composite-list registration and take an explicit `index`. The
 * scroll element is the `ComboboxList` itself (its `overflow-y-auto` box); an
 * inner sized div holds absolutely positioned rows. Keyboard highlight is
 * mirrored to the virtualizer through `onItemHighlighted`.
 */

export const DEFAULT_RECOVERY_TEXT =
  "This saved value is no longer available. Choose another to replace it."

const TRIGGER_CLASS =
  "flex h-9 w-full items-center justify-between gap-1.5 rounded-md bg-input/30 py-2 pr-2 pl-2.5 font-mono text-xs whitespace-nowrap transition-[color,background-color,box-shadow] outline-none hover:bg-input/50 focus-visible:ring-3 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40"

const GROUP_LABEL_CLASS =
  "font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground"

export type PickerSelection =
  | {
      kind: "single"
      value: string
      onValueChange: (value: string) => void
    }
  | {
      kind: "multiple"
      values: readonly string[]
      onValuesChange: (values: string[]) => void
    }

interface PickerContextValue {
  mode: PickerRenderMode
  status: PickerStatus
  errorText: string
  options: readonly PickerOption[]
  filtered: readonly PickerOption[]
  groups: PickerGroup[] | null
  query: string
  deferredQuery: string
  countLabel: (count: number) => string
  placeholder: string
  emptyLabel: string
  noMatchLabel: string
  loadingLabel: string
  triggerText: string
  hasValue: boolean
  unavailable: boolean
  recoveryText: string | null
  recoveryId: string
  disabled: boolean
  onClear: (() => void) | undefined
  clearLabel: string
  selectedOptionIndex: number
  scrollToOptionRef: React.RefObject<((optionIndex: number) => void) | null>
  virtual: { rows: VirtualRow[]; optionRowIndex: number[] } | null
}

const PickerContext = React.createContext<PickerContextValue | null>(null)

function usePicker(): PickerContextValue {
  const context = React.use(PickerContext)
  if (!context) throw new Error("ComboboxPicker parts must render inside ComboboxPicker.Root")
  return context
}

export interface PickerRootProps {
  options: readonly PickerOption[]
  selection: PickerSelection
  status?: PickerStatus
  errorMessage?: string
  countLabel?: (count: number) => string
  placeholder?: string
  emptyLabel?: string
  noMatchLabel?: string
  loadingLabel?: string
  /** Recovery text for an unavailable stored value. Null when the caller already describes it. */
  unavailableText?: string | null
  disabled?: boolean
  /** When set, the trigger shows a clear button that calls it. */
  onClear?: () => void
  clearLabel?: string
  children: React.ReactNode
}

function PickerRoot({
  options,
  selection,
  status = "ready",
  errorMessage,
  countLabel = defaultCountLabel,
  placeholder = "Select an option",
  emptyLabel = "No options available.",
  noMatchLabel = "No matching options.",
  loadingLabel = "Loading options…",
  unavailableText = DEFAULT_RECOVERY_TEXT,
  disabled = false,
  onClear,
  clearLabel = "Clear selection",
  children,
}: PickerRootProps) {
  const recoveryId = React.useId()
  const [open, setOpen] = React.useState(false)
  const [query, setQuery] = React.useState("")
  // Typing stays responsive: filtering and re-windowing follow the deferred query.
  const deferredQuery = React.useDeferredValue(query)
  const scrollToOptionRef = React.useRef<((optionIndex: number) => void) | null>(null)

  const mode = pickerRenderMode(options.length, status)
  const filtered = React.useMemo(
    () => filterPickerOptions(options, deferredQuery),
    [options, deferredQuery],
  )
  const groups = React.useMemo(() => groupPickerOptions(filtered), [filtered])
  const flat = React.useMemo(() => flattenGroups(groups, filtered), [groups, filtered])
  const structuredAll = React.useMemo(
    () => groupPickerOptions(options) ?? options,
    [options],
  )
  const virtual = React.useMemo(
    () => (mode === "virtual" ? buildVirtualRows(groups, filtered) : null),
    [mode, groups, filtered],
  )

  const byValue = React.useMemo(() => {
    const map = new Map<string, PickerOption>()
    for (const option of options) map.set(option.value, option)
    return map
  }, [options])

  const storedValues: readonly string[] =
    selection.kind === "single"
      ? selection.value
        ? [selection.value]
        : []
      : selection.values
  const selectedOptions: PickerOption[] = []
  const missing: string[] = []
  for (const value of storedValues) {
    const option = byValue.get(value)
    if (option && !option.disabled) selectedOptions.push(option)
    else missing.push(value)
  }
  const unavailable = status === "ready" && missing.length > 0
  const errorText = errorMessage ?? "Could not load options."

  const selectedLabels = selectedOptions.map((option) => option.label)
  let triggerText = placeholder
  if (unavailable) {
    const gone = `Unavailable: ${missing.join(", ")}`
    triggerText = selectedLabels.length > 0 ? `${selectedLabels.join(", ")}; ${gone}` : gone
  } else if (selectedOptions.length > 0) {
    triggerText =
      selection.kind === "single" || selectedOptions.length <= 2
        ? selectedLabels.join(", ")
        : `${selectedOptions.length} selected`
  } else if (status === "loading") {
    triggerText = loadingLabel
  } else if (status === "error") {
    triggerText = errorText
  }

  const selectedValue = storedValues[0]
  const selectedOptionIndex = React.useMemo(
    () =>
      selection.kind === "single" && selectedValue
        ? flat.findIndex((option) => option.value === selectedValue)
        : -1,
    [flat, selection.kind, selectedValue],
  )

  const handleHighlight = (
    item: PickerOption | undefined,
    details: { reason: string },
  ) => {
    if (!item || mode !== "virtual" || details.reason === "pointer") return
    const index = flat.findIndex((option) => option.value === item.value)
    if (index >= 0) scrollToOptionRef.current?.(index)
  }

  const handleOpenChange = (next: boolean) => {
    setOpen(next)
    if (!next) setQuery("")
  }

  const context: PickerContextValue = {
    mode,
    status,
    errorText,
    options,
    filtered,
    groups,
    query,
    deferredQuery,
    countLabel,
    placeholder,
    emptyLabel,
    noMatchLabel,
    loadingLabel,
    triggerText,
    hasValue: storedValues.length > 0,
    unavailable,
    recoveryText: unavailableText,
    recoveryId,
    disabled,
    onClear,
    clearLabel,
    selectedOptionIndex,
    scrollToOptionRef,
    virtual,
  }

  const common = {
    items: structuredAll,
    filteredItems: groups ?? filtered,
    filter: null,
    open,
    onOpenChange: handleOpenChange,
    inputValue: query,
    onInputValueChange: setQuery,
    itemToStringLabel: (option: PickerOption) => option.label,
    itemToStringValue: (option: PickerOption) => option.value,
    isItemEqualToValue: (item: PickerOption, value: PickerOption) =>
      item.value === value.value,
    onItemHighlighted: handleHighlight,
    autoHighlight: true,
    virtualized: mode === "virtual",
    disabled,
  }

  return (
    <PickerContext.Provider value={context}>
      {selection.kind === "single" ? (
        <Combobox<PickerOption, false>
          {...common}
          value={selectedOptions[0] ?? null}
          onValueChange={(next) => {
            if (next && !next.disabled) selection.onValueChange(next.value)
            handleOpenChange(false)
          }}
        >
          {children}
        </Combobox>
      ) : (
        <Combobox<PickerOption, true>
          {...common}
          multiple
          value={selectedOptions}
          onValueChange={(next) => selection.onValuesChange(next.map((option) => option.value))}
        >
          {children}
        </Combobox>
      )}
    </PickerContext.Provider>
  )
}

export interface PickerTriggerProps {
  id?: string
  "aria-label": string
  "aria-describedby"?: string
  /** Forces the invalid state in addition to an unavailable stored value. */
  invalid?: boolean
  className?: string
  "data-testid"?: string
}

function PickerTrigger({
  id,
  "aria-label": ariaLabel,
  "aria-describedby": describedBy,
  invalid,
  className,
  "data-testid": testId,
}: PickerTriggerProps) {
  const picker = usePicker()
  const showRecovery = picker.unavailable && picker.recoveryText !== null
  const clearable = picker.onClear !== undefined && picker.hasValue && !picker.disabled
  const description =
    [describedBy, showRecovery ? picker.recoveryId : undefined].filter(Boolean).join(" ") ||
    undefined
  const isInvalid = invalid === true || picker.unavailable || picker.status === "error"
  return (
    <div className="relative w-full">
      <ComboboxTrigger
        id={id}
        data-testid={testId}
        aria-label={ariaLabel}
        aria-invalid={isInvalid || undefined}
        aria-busy={picker.status === "loading" || undefined}
        aria-describedby={description}
        disabled={picker.disabled}
        className={cn(TRIGGER_CLASS, clearable && "pr-14", className)}
      >
        <span
          className={cn(
            "min-w-0 flex-1 truncate text-left",
            !picker.hasValue && "text-muted-foreground",
            picker.unavailable && "text-destructive",
          )}
        >
          {picker.triggerText}
        </span>
      </ComboboxTrigger>
      {clearable && (
        <PickerClearButton label={picker.clearLabel} onClear={picker.onClear!} />
      )}
      {showRecovery && (
        <span id={picker.recoveryId} className="sr-only">
          {picker.recoveryText}
        </span>
      )}
    </div>
  )
}

export function PickerClearButton({
  label,
  onClear,
}: {
  label: string
  onClear: () => void
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClear}
      className="absolute top-1/2 right-8 -translate-y-1/2 rounded-sm p-1 text-muted-foreground outline-none hover:bg-accent hover:text-accent-foreground focus-visible:ring-3 focus-visible:ring-ring"
    >
      <X className="size-3" aria-hidden="true" />
    </button>
  )
}

function PickerContent({ className, children }: { className?: string; children: React.ReactNode }) {
  return <ComboboxContent className={cn("min-w-(--anchor-width)", className)}>{children}</ComboboxContent>
}

function PickerSearch({
  "aria-label": ariaLabel,
  placeholder = "Search…",
}: {
  "aria-label": string
  placeholder?: string
}) {
  return (
    <ComboboxInput
      autoFocus
      showTrigger={false}
      aria-label={ariaLabel}
      placeholder={placeholder}
      className="w-full font-mono text-xs"
    />
  )
}

/** Polite status line: match count when ready, progress text while loading. */
function PickerStatusLine() {
  const picker = usePicker()
  let text = ""
  if (picker.status === "loading") text = picker.loadingLabel
  else if (picker.status === "ready" && picker.options.length > 0) {
    text = picker.countLabel(picker.filtered.length)
  }
  return (
    <ComboboxStatus className="font-mono" data-testid="picker-status">
      {text}
    </ComboboxStatus>
  )
}

function PickerEmpty() {
  const picker = usePicker()
  return (
    <ComboboxEmpty
      className={cn("font-mono text-xs", picker.status === "loading" && "py-0")}
    >
      {picker.status === "error" ? (
        <span role="alert" className="text-destructive">
          {picker.errorText}
        </span>
      ) : picker.status === "loading" ? null : picker.options.length === 0 ? (
        picker.emptyLabel
      ) : (
        picker.noMatchLabel
      )}
    </ComboboxEmpty>
  )
}

interface PickerItemProps {
  option: PickerOption
  index?: number
  style?: React.CSSProperties
  setSize?: number
}

const PickerItem = React.memo(function PickerItem({
  option,
  index,
  style,
  setSize,
}: PickerItemProps) {
  const virtualAria =
    setSize !== undefined && index !== undefined
      ? { "aria-setsize": setSize, "aria-posinset": index + 1 }
      : undefined
  return (
    <ComboboxItem
      value={option}
      disabled={option.disabled}
      index={index}
      style={style}
      className={cn("items-center gap-2 font-mono text-xs", style && "h-9")}
      {...optionMarkers(option)}
      {...virtualAria}
    >
      <span className="min-w-0 flex-1 truncate">{option.label}</span>
      {option.detail && option.detail !== option.label && (
        <span className="max-w-[45%] shrink-0 truncate text-[10px] text-muted-foreground">
          {option.detail}
        </span>
      )}
      {option.badge && (
        <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
          {option.badge}
        </span>
      )}
      {option.disabled && option.disabledReason && (
        <span className="shrink-0 text-[10px] text-muted-foreground">
          {option.disabledReason}
        </span>
      )}
    </ComboboxItem>
  )
})

function PickerGroupItems({ group }: { group: PickerGroup }) {
  return (
    <ComboboxGroup items={group.items}>
      <ComboboxLabel className={GROUP_LABEL_CLASS}>{group.label}</ComboboxLabel>
      <ComboboxCollection>
        {(option: PickerOption) => <PickerItem key={option.value} option={option} />}
      </ComboboxCollection>
    </ComboboxGroup>
  )
}

function VirtualPickerList({
  virtual,
}: {
  virtual: { rows: VirtualRow[]; optionRowIndex: number[] }
}) {
  const picker = usePicker()
  const { rows, optionRowIndex } = virtual
  const [scrollElement, setScrollElement] = React.useState<HTMLElement | null>(null)
  const attach = React.useCallback((element: HTMLDivElement | null) => {
    // The sized div sits directly inside the ComboboxList, which owns the scrollbar.
    setScrollElement(element?.parentElement ?? null)
  }, [])

  // TanStack Virtual returns imperative functions the React Compiler cannot memoize.
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollElement,
    estimateSize: (index) =>
      rows[index]?.kind === "group" ? GROUP_ROW_HEIGHT_PX : OPTION_ROW_HEIGHT_PX,
    getItemKey: (index) => rows[index]?.key ?? index,
    overscan: 8,
    initialRect: { width: 400, height: 288 },
  })

  const { scrollToOptionRef, selectedOptionIndex, deferredQuery } = picker
  React.useEffect(() => {
    scrollToOptionRef.current = (optionIndex: number) => {
      const row = optionRowIndex[optionIndex]
      if (row !== undefined) virtualizer.scrollToIndex(row, { align: "auto" })
    }
    return () => {
      scrollToOptionRef.current = null
    }
  }, [scrollToOptionRef, optionRowIndex, virtualizer])

  // New result set: show the stored value when browsing, the top when searching.
  React.useEffect(() => {
    const selectedRow =
      deferredQuery.trim() === "" && selectedOptionIndex >= 0
        ? optionRowIndex[selectedOptionIndex]
        : undefined
    if (selectedRow !== undefined) virtualizer.scrollToIndex(selectedRow, { align: "center" })
    else virtualizer.scrollToOffset(0)
  }, [rows, deferredQuery, selectedOptionIndex, optionRowIndex, virtualizer])

  const measured = virtualizer.getVirtualItems()
  const items =
    measured.length > 0
      ? measured
      : rows.slice(0, 12).map((row, index) => ({
          index,
          key: row.key,
          start: index * OPTION_ROW_HEIGHT_PX,
        }))

  return (
    <div
      ref={attach}
      className="relative w-full"
      style={{ height: `${virtualizer.getTotalSize()}px` }}
      data-virtualized="true"
      data-mounted-count={items.length}
    >
      {items.map((item) => {
        const row = rows[item.index]
        if (!row) return null
        const style: React.CSSProperties = {
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          height: row.kind === "group" ? GROUP_ROW_HEIGHT_PX : OPTION_ROW_HEIGHT_PX,
          transform: `translateY(${item.start}px)`,
        }
        if (row.kind === "group") {
          return (
            <div
              key={item.key}
              role="presentation"
              style={style}
              className={cn("flex items-end px-2 pb-1", GROUP_LABEL_CLASS)}
            >
              {row.label}
            </div>
          )
        }
        return (
          <PickerItem
            key={item.key}
            option={row.option}
            index={row.optionIndex}
            setSize={optionRowIndex.length}
            style={style}
          />
        )
      })}
    </div>
  )
}

function PickerList() {
  const picker = usePicker()
  if (picker.virtual) {
    return (
      <ComboboxList>
        <VirtualPickerList virtual={picker.virtual} />
      </ComboboxList>
    )
  }
  return (
    <ComboboxList>
      {picker.groups
        ? (group: PickerGroup) => <PickerGroupItems key={group.id} group={group} />
        : (option: PickerOption) => <PickerItem key={option.value} option={option} />}
    </ComboboxList>
  )
}

// The compound namespace lives in combobox-picker-namespace.ts so this module
// exports only components (React Fast Refresh).
export {
  PickerContent,
  PickerEmpty,
  PickerGroupItems,
  PickerItem,
  PickerList,
  PickerRoot,
  PickerSearch,
  PickerStatusLine,
  PickerTrigger,
}
