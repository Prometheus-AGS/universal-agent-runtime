import { useEffect, useState, type FC } from "react";
import type { ComponentContext } from "@prometheus-ags/a2ui-core/v0_9";
import type { BuildChild } from "../react/types";
import { Card as UiCard, CardContent } from "../components/ui/card";
import type { EntityStreamItem } from "./entity-stream-api";

interface RawEntityStreamProperties {
  entityType: string;
  source: { path: string };
  title?: string;
}

/**
 * `EntityStream` render implementation — see `entity-stream-api.ts` for the
 * schema and the binderless-vs-bound rationale. Registered via
 * `createBinderlessUarComponentImplementation`, so this receives only
 * `{ context, buildChild }`, not pre-resolved props: it reads its own
 * static declaration off `context.componentModel.properties` and manages
 * its own `dataContext` subscription.
 */
export const UarEntityStream: FC<{
  context: ComponentContext;
  buildChild: BuildChild;
}> = ({ context }) => {
  const properties = context.componentModel.properties as RawEntityStreamProperties;
  // Seeding via a lazy initializer (rather than setState inside the effect
  // below) avoids the extra synchronous re-render `react-hooks/set-state-in-effect`
  // flags. This subscribes once just to read the current value synchronously,
  // then immediately unsubscribes — the effect below re-subscribes for real,
  // ongoing updates.
  const [items, setItems] = useState<EntityStreamItem[]>(() => {
    const initial = context.dataContext.subscribeDynamicValue<EntityStreamItem[]>(
      { path: properties.source.path },
      () => undefined,
    );
    initial.unsubscribe();
    return initial.value ?? [];
  });

  useEffect(() => {
    const subscription = context.dataContext.subscribeDynamicValue<EntityStreamItem[]>(
      { path: properties.source.path },
      (next) => setItems(next ?? []),
    );
    return () => subscription.unsubscribe();
    // `context` and `properties.source.path` are stable for this component's
    // lifetime (a new `source` path means a new component instance, per the
    // A2UI protocol's structural-update model), so this effect only needs to
    // run once per mount.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <UiCard data-a2ui-component="EntityStream" data-entity-type={properties.entityType}>
      <CardContent className="flex flex-col gap-2">
        {properties.title ? (
          <h3 className="text-base font-semibold">{properties.title}</h3>
        ) : null}
        {items.length === 0 ? (
          <p className="text-sm text-muted-foreground">No items yet.</p>
        ) : (
          <ul className="flex flex-col gap-1 text-sm" data-a2ui-stream-count={items.length}>
            {items.map((item) => (
              <li
                key={item.id}
                data-a2ui-stream-item-id={item.id}
                className="flex items-baseline justify-between gap-2 border-b border-border/50 py-1 last:border-b-0"
              >
                <span>{item.label}</span>
                {item.value !== undefined ? (
                  <span className="text-muted-foreground">{item.value}</span>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </UiCard>
  );
};
