import type { FC } from "react";
import { ListBox, ListBoxItem, type Selection } from "react-aria-components";
import type { ChoicePickerApi } from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import type { UarComponentProps } from "../react/types";
import { cn } from "../lib/cn";
import { resolvedText } from "../lib/resolved";

type ChoicePickerProps = UarComponentProps<typeof ChoicePickerApi>;

/**
 * `ChoicePicker` — single/multi-select from a fixed option list.
 *
 * Uses `react-aria-components`' `ListBox` (rather than shadcn's Radix/Base
 * UI `Select`, which is single-value only) because `variant:
 * multipleSelection` requires real multi-select listbox semantics
 * (`aria-multiselectable`, roving tabindex, type-ahead) — this is exactly
 * the accessibility-primitive gap `react-aria-components` is meant to fill
 * alongside the shadcn/ui visual baseline.
 */
export const UarChoicePicker: FC<{ props: ChoicePickerProps }> = ({ props }) => {
  const selectionMode = props.variant === "multipleSelection" ? "multiple" : "single";
  const selectedKeys = new Set(props.value ?? []);

  const handleSelectionChange = (keys: Selection) => {
    if (keys === "all") return;
    props.setValue(Array.from(keys, String));
  };

  return (
    <div data-a2ui-component="ChoicePicker" className="flex flex-col gap-1.5">
      {props.label ? <span className="text-sm font-medium">{props.label}</span> : null}
      <ListBox
        aria-label={props.label ?? resolvedText(props.accessibility?.label) ?? "Choices"}
        selectionMode={selectionMode}
        selectedKeys={selectedKeys}
        onSelectionChange={handleSelectionChange}
        className="flex flex-col gap-1 rounded-md border border-input p-1"
      >
        {props.options.map((option) => (
          <ListBoxItem
            key={option.value}
            id={option.value}
            textValue={resolvedText(option.label)}
            className={({ isSelected, isFocused }: { isSelected: boolean; isFocused: boolean }) =>
              cn(
                "cursor-pointer rounded-sm px-2 py-1 text-sm outline-none",
                isFocused && "bg-muted",
                isSelected && "bg-primary text-primary-foreground",
              )
            }
          >
            {resolvedText(option.label)}
          </ListBoxItem>
        ))}
      </ListBox>
      {props.validationErrors?.length ? (
        <span role="alert" className="text-xs text-destructive">
          {props.validationErrors.join(" ")}
        </span>
      ) : null}
    </div>
  );
};
