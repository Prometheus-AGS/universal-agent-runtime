import { create } from "zustand";

const useCount = create<{ count: number }>(() => ({ count: 0 }));

export function Stateful() {
  return <span>{useCount((state) => state.count)}</span>;
}
