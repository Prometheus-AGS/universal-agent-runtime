import { useId, type ChangeEvent, type FC } from "react";
import type { TextFieldApi } from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import type { UarComponentProps } from "../react/types";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { resolvedText } from "../lib/resolved";

type TextFieldProps = UarComponentProps<typeof TextFieldApi>;

const HTML_TYPE: Record<string, string> = {
  longText: "text",
  number: "number",
  shortText: "text",
  obscured: "password",
};

/**
 * `TextField` — two-way bound text input. `GenericBinder` generates
 * `setValue` for the `value: DynamicString` field, so committing a value
 * back to the data model is just calling `props.setValue`.
 */
export const UarTextField: FC<{ props: TextFieldProps }> = ({ props }) => {
  const id = useId();
  const isLongText = props.variant === "longText";
  const sharedProps = {
    id,
    value: props.value ?? "",
    onChange: (e: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
      props.setValue(e.target.value),
    "aria-invalid": props.isValid === false || undefined,
    "aria-describedby": props.accessibility?.description ? `${id}-description` : undefined,
    pattern: props.validationRegexp,
  };

  return (
    <div data-a2ui-component="TextField" className="flex flex-col gap-1.5">
      <Label htmlFor={id}>{props.label}</Label>
      {isLongText ? (
        <textarea
          {...sharedProps}
          className="min-h-20 w-full rounded-md border border-input bg-transparent px-2.5 py-1.5 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
        />
      ) : (
        <Input type={HTML_TYPE[props.variant ?? "shortText"]} {...sharedProps} />
      )}
      {props.accessibility?.description ? (
        <span id={`${id}-description`} className="text-xs text-muted-foreground">
          {resolvedText(props.accessibility.description)}
        </span>
      ) : null}
      {props.validationErrors?.length ? (
        <span role="alert" className="text-xs text-destructive">
          {props.validationErrors.join(" ")}
        </span>
      ) : null}
    </div>
  );
};
