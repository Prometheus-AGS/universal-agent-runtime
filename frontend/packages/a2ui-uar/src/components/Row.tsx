import type { FC } from "react";
import type { RowApi } from "@prometheus-ags/a2ui-core/v0_9/basic_catalog";
import type { BuildChild, UarComponentProps } from "../react/types";
import { cn } from "../lib/cn";
import { resolveChildRefs } from "../lib/child-refs";

const JUSTIFY: Record<string, string> = {
  start: "justify-start",
  center: "justify-center",
  end: "justify-end",
  spaceAround: "justify-around",
  spaceBetween: "justify-between",
  spaceEvenly: "justify-evenly",
  stretch: "justify-stretch",
};

const ALIGN: Record<string, string> = {
  start: "items-start",
  center: "items-center",
  end: "items-end",
  stretch: "items-stretch",
};

type RowProps = UarComponentProps<typeof RowApi>;

/** `Row` — horizontal layout primitive. Renders `children` (a `web_core`-resolved `{ id, basePath }[]`) via `buildChild`. */
export const UarRow: FC<{ props: RowProps; buildChild: BuildChild }> = ({ props, buildChild }) => {
  const children = resolveChildRefs(props.children);
  return (
    <div
      data-a2ui-component="Row"
      className={cn(
        "flex flex-row gap-2",
        JUSTIFY[props.justify ?? "start"],
        ALIGN[props.align ?? "stretch"],
      )}
    >
      {children.map((child) => (
        <div key={child.id} style={props.weight ? { flexGrow: props.weight } : undefined}>
          {buildChild(child.id, child.basePath)}
        </div>
      ))}
    </div>
  );
};
