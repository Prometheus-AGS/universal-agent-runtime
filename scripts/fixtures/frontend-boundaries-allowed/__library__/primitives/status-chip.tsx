import type { ReactNode } from "react";
import { toneLabel } from "./tone";

export function StatusChip({ tone, children }: { tone: "ok" | "warn"; children: ReactNode }) {
  return <span aria-label={toneLabel(tone)}>{children}</span>;
}
