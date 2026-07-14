import { useId, type FC } from "react";
import type { CheckBoxApi } from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import type { UarComponentProps } from "../react/types";
import { Checkbox } from "./ui/checkbox";
import { Label } from "./ui/label";

type CheckBoxProps = UarComponentProps<typeof CheckBoxApi>;

/** `CheckBox` — two-way bound boolean input via the generated `setValue` setter. */
export const UarCheckBox: FC<{ props: CheckBoxProps }> = ({ props }) => {
  const id = useId();
  return (
    <div data-a2ui-component="CheckBox" className="flex items-center gap-2">
      <Checkbox
        id={id}
        checked={!!props.value}
        onCheckedChange={(checked: boolean) => props.setValue(checked === true)}
        aria-invalid={props.isValid === false || undefined}
      />
      <Label htmlFor={id}>{props.label}</Label>
      {props.validationErrors?.length ? (
        <span role="alert" className="text-xs text-destructive">
          {props.validationErrors.join(" ")}
        </span>
      ) : null}
    </div>
  );
};
