/**
 * Pure picker logic: option shape, search, grouping and virtual row layout.
 *
 * No React in this file. The compound `ComboboxPicker` and its variants read
 * these helpers, so counts and ordering are decided in one place.
 */

/** Fewer than eight options render a simple select (existing spec rule). */
export const SIMPLE_SELECT_MAX_OPTIONS = 7;
/** More than this many options render only the visible rows. */
export const VIRTUALIZE_ABOVE_OPTIONS = 1000;
export const OPTION_ROW_HEIGHT_PX = 36;
export const GROUP_ROW_HEIGHT_PX = 28;

export type PickerStatus = "ready" | "loading" | "error";

export interface PickerOption {
  /** The stored value, for example a provider id, model id or `provider/model` route. */
  value: string;
  /** Display name shown as the primary text. */
  label: string;
  /** Raw identifier shown beside the label when it differs. Searched. */
  detail?: string;
  /** Short trailing chip such as a context window. Not searched. */
  badge?: string;
  /** Stable group id. Options with no group render ungrouped. */
  groupId?: string;
  /** Group heading. Not searched, so a provider name never matches every model. */
  groupLabel?: string;
  disabled?: boolean;
  /** Visible reason an option cannot be chosen. */
  disabledReason?: string;
  /** Extra searchable text that is not displayed. */
  searchText?: string;
  /** Rendered as `data-testid` on the option element. */
  testId?: string;
  /** Rendered as `data-model-id` on the option element. */
  modelId?: string;
}

/** Marker attributes shared by both rendering modes. Non-selectable options are flagged. */
export function optionMarkers(option: PickerOption): Record<string, string | undefined> {
  return {
    "data-testid": option.testId,
    "data-model-id": option.modelId,
    "aria-disabled": option.disabled ? "true" : undefined,
    "data-disabled": option.disabled ? "" : undefined,
  };
}

/** A group in the shape Base UI expects for grouped items. */
export type PickerGroup = {
  id: string;
  label: string;
  items: PickerOption[];
};

export type PickerRenderMode = "select" | "list" | "virtual";

export type VirtualRow =
  | { kind: "group"; key: string; label: string; count: number }
  | { kind: "option"; key: string; option: PickerOption; optionIndex: number };

export function pickerRenderMode(
  optionCount: number,
  status: PickerStatus,
): PickerRenderMode {
  if (status === "ready" && optionCount <= SIMPLE_SELECT_MAX_OPTIONS) {
    return "select";
  }
  return optionCount > VIRTUALIZE_ABOVE_OPTIONS ? "virtual" : "list";
}

export function normalizeSearch(value: string): string {
  return value.trim().toLowerCase();
}

const haystackCache = new WeakMap<PickerOption, string>();

function haystack(option: PickerOption): string {
  let cached = haystackCache.get(option);
  if (cached === undefined) {
    cached = [option.label, option.value, option.detail ?? "", option.searchText ?? ""]
      .join("\u0000")
      .toLowerCase();
    haystackCache.set(option, cached);
  }
  return cached;
}

/** Case-insensitive substring match over display name and raw identifier. */
export function filterPickerOptions(
  options: readonly PickerOption[],
  query: string,
): readonly PickerOption[] {
  const needle = normalizeSearch(query);
  if (!needle) return options;
  return options.filter((option) => haystack(option).includes(needle));
}

/** Groups by first appearance. Returns null when no option names a group. */
export function groupPickerOptions(
  options: readonly PickerOption[],
): PickerGroup[] | null {
  if (!options.some((option) => option.groupId !== undefined)) return null;
  const groups = new Map<string, PickerGroup>();
  for (const option of options) {
    const id = option.groupId ?? "";
    let group = groups.get(id);
    if (!group) {
      group = { id, label: option.groupLabel ?? (id || "Other"), items: [] };
      groups.set(id, group);
    }
    group.items.push(option);
  }
  return [...groups.values()];
}

/**
 * Flattens options into virtual rows. Group headings are presentational rows;
 * `optionIndex` is the position among options only, which is the index Base UI
 * highlights.
 */
export function buildVirtualRows(
  groups: readonly PickerGroup[] | null,
  options: readonly PickerOption[],
): { rows: VirtualRow[]; optionRowIndex: number[] } {
  const rows: VirtualRow[] = [];
  const optionRowIndex: number[] = [];
  const pushOption = (option: PickerOption) => {
    optionRowIndex.push(rows.length);
    rows.push({
      kind: "option",
      key: `o:${option.value}`,
      option,
      optionIndex: optionRowIndex.length - 1,
    });
  };
  if (groups) {
    for (const group of groups) {
      rows.push({
        kind: "group",
        key: `g:${group.id}`,
        label: group.label,
        count: group.items.length,
      });
      group.items.forEach(pushOption);
    }
  } else {
    options.forEach(pushOption);
  }
  return { rows, optionRowIndex };
}

/** Flat options in display order (grouped order when grouped). */
export function flattenGroups(
  groups: readonly PickerGroup[] | null,
  options: readonly PickerOption[],
): readonly PickerOption[] {
  return groups ? groups.flatMap((group) => group.items) : options;
}

/**
 * A stored value is unavailable when the list is ready and does not contain it,
 * or contains it only as a disabled entry. It is never cleared automatically.
 */
export function isUnavailableValue(
  value: string,
  options: readonly PickerOption[],
  status: PickerStatus,
): boolean {
  if (!value || status !== "ready") return false;
  const found = options.find((option) => option.value === value);
  return found === undefined || found.disabled === true;
}

export function defaultCountLabel(count: number): string {
  return `${count} ${count === 1 ? "option" : "options"}`;
}
