/** Pure display helpers for the Teams feature. No platform imports so they stay trivially testable. */

const MICROUNITS_PER_UNIT = 1_000_000;

export function formatDateTime(value: string | number | null | undefined): string {
  if (value === null || value === undefined || value === "") return "unknown time";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? String(value) : date.toLocaleString();
}

export function formatCount(value: number): string {
  return new Intl.NumberFormat().format(value);
}

/** Cost is carried in microunits of the budget currency; show it as currency. */
export function formatMicrocost(microunits: number, currency: string | undefined): string {
  const amount = microunits / MICROUNITS_PER_UNIT;
  if (!currency) return formatCount(amount);
  try {
    return new Intl.NumberFormat(undefined, {
      style: "currency",
      currency,
      minimumFractionDigits: 2,
      maximumFractionDigits: 4,
    }).format(amount);
  } catch {
    return `${formatCount(amount)} ${currency}`;
  }
}

export function formatSeconds(seconds: number): string {
  if (seconds < 120) return `${formatCount(seconds)} s`;
  const minutes = Math.round(seconds / 60);
  return minutes < 120 ? `${formatCount(minutes)} min` : `${formatCount(Math.round(minutes / 60))} h`;
}

/** The last non-empty segment of an id path or URN: `urn:uar:binding:binding-qwen-2` and `a/b/binding-qwen-2` both give `binding-qwen-2`. */
export function lastSegment(id: string): string {
  const parts = id.split(/[/:]/).filter((part) => part !== "");
  return parts[parts.length - 1] ?? id;
}

/** The last `length` characters of an id, enough to tell sibling records apart. */
export function shortId(id: string, length = 6): string {
  return id.length <= length ? id : id.slice(-length);
}

/** Shortens a long value in the middle so both its start and its end stay readable. The full value belongs in a `title`. */
export function elideMiddle(text: string, max = 36): string {
  if (text.length <= max) return text;
  const keep = max - 1;
  const head = Math.ceil(keep / 2);
  return `${text.slice(0, head)}…${text.slice(text.length - (keep - head))}`;
}

/** `sha256:abcdef…123456` */
export function shortDigest(digest: string): string {
  const [scheme, hex] = digest.includes(":") ? digest.split(":", 2) : ["", digest];
  if (hex.length <= 16) return digest;
  return `${scheme ? `${scheme}:` : ""}${hex.slice(0, 8)}…${hex.slice(-6)}`;
}

export function dollarsToMicrounits(dollars: number): number {
  return Math.round(dollars * MICROUNITS_PER_UNIT);
}

export function microunitsToDollars(microunits: number): number {
  return microunits / MICROUNITS_PER_UNIT;
}
