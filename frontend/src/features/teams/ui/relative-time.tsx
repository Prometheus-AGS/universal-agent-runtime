import { useEffect, useState } from "react";
import { formatDateTime } from "../model/format";
import { formatAge } from "./run/run-model";

const TICK_MS = 30_000;
const DAY_MS = 86_400_000;

function relativeText(elapsedMs: number): string {
  const days = Math.floor(elapsedMs / DAY_MS);
  return days >= 2 ? `${days} d ago` : formatAge(elapsedMs);
}

/**
 * "12 min ago", with the full local timestamp on hover. The age is re-read every 30 seconds so a list left
 * open does not drift; it is not a live region.
 */
export function RelativeTime({ value, className }: { value: string | number | null | undefined; className?: string }) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), TICK_MS);
    return () => clearInterval(timer);
  }, []);
  const at = value === null || value === undefined || value === "" ? Number.NaN : new Date(value).getTime();
  const full = formatDateTime(value);
  if (Number.isNaN(at)) return <span className={className}>{full}</span>;
  return (
    <time dateTime={new Date(at).toISOString()} title={full} className={className}>
      {relativeText(now - at)}
    </time>
  );
}
