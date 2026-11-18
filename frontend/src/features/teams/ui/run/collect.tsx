import type { ReactNode } from "react";

/**
 * Gathers one narrow hook result per id without a hook in a loop and without copying graph data into
 * component state. It recurses over components: level N calls `useValue` for `ids[N]` (a hook call at a
 * fixed position of its own component), folds the value into `acc`, and renders level N + 1. The leaf
 * hands the folded result to `children`.
 *
 * The platform exposes per-record hooks only; cross-record views (the order of lanes, which roles have
 * available members, which messages involve one member) need a fold like this until the domain offers
 * derived hooks. `ids` is expected to be small (a team's members, a member's waits).
 *
 * `useValue` and `reduce` must be module-level functions so their identity never changes.
 */
export interface CollectProps<Context, Value, Acc> {
  ids: readonly string[];
  context: Context;
  /** A hook: it runs once per id, in a component of its own. Called with "" past the last id. */
  useValue: (id: string, context: Context) => Value;
  reduce: (acc: Acc, id: string, value: Value) => Acc;
  initial: Acc;
  children: (result: Acc) => ReactNode;
}

interface StepProps<Context, Value, Acc> extends CollectProps<Context, Value, Acc> {
  index: number;
  acc: Acc;
}

function CollectStep<Context, Value, Acc>(props: StepProps<Context, Value, Acc>) {
  const { ids, context, useValue, reduce, index, acc, children } = props;
  const id = ids[index] ?? "";
  const value = useValue(id, context);
  if (index >= ids.length) return <>{children(acc)}</>;
  return <CollectStep {...props} index={index + 1} acc={reduce(acc, id, value)} />;
}

export function Collect<Context, Value, Acc>(props: CollectProps<Context, Value, Acc>) {
  return <CollectStep {...props} index={0} acc={props.initial} />;
}
