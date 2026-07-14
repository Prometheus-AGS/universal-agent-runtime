import type { FC } from "react";
import type { UarComponentProps } from "../react/types";
import { Card as UiCard, CardContent } from "../components/ui/card";
import { cn } from "../lib/cn";
import { resolvedText } from "../lib/resolved";
import type { EntityDiffApi } from "./entity-diff-api";

type EntityDiffProps = UarComponentProps<typeof EntityDiffApi>;

/** `EntityDiff` render implementation — see `entity-diff-api.ts` for the schema. */
export const UarEntityDiff: FC<{ props: EntityDiffProps }> = ({ props }) => {
  return (
    <UiCard
      data-a2ui-component="EntityDiff"
      data-entity-type={props.entityType}
      data-entity-id={props.entityId}
    >
      <CardContent className="flex flex-col gap-3">
        {props.title ? (
          <h3 className="text-base font-semibold">{resolvedText(props.title)}</h3>
        ) : null}

        {props.fields.length ? (
          <div className="grid grid-cols-[auto_1fr_1fr] gap-x-4 gap-y-1 text-sm">
            <span className="text-xs font-medium uppercase text-muted-foreground" />
            <span className="text-xs font-medium uppercase text-muted-foreground">
              Before
            </span>
            <span className="text-xs font-medium uppercase text-muted-foreground">
              After
            </span>
            {props.fields.map((field, index) => {
              const before = resolvedText(field.before);
              const after = resolvedText(field.after);
              const changed = before !== after;
              return (
                // A2UI field entries carry no id of their own (display-only
                // data, not addressable child components), so position is
                // the only stable key available here — same convention as
                // EntityCard.
                <div key={`${resolvedText(field.label)}-${index}`} className="contents">
                  <dt className="text-muted-foreground">{resolvedText(field.label)}</dt>
                  <dd
                    data-a2ui-diff-side="before"
                    className={cn(changed && "text-red-600 line-through dark:text-red-400")}
                  >
                    {before}
                  </dd>
                  <dd
                    data-a2ui-diff-side="after"
                    data-a2ui-diff-changed={changed}
                    className={cn(
                      changed && "font-medium text-emerald-600 dark:text-emerald-400",
                    )}
                  >
                    {after}
                  </dd>
                </div>
              );
            })}
          </div>
        ) : null}
      </CardContent>
    </UiCard>
  );
};
