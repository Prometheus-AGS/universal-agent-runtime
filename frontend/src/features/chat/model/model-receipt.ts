import type { ContentBlock } from "@/shared/content";
import { isModelOrigin, type ModelOrigin } from "@/platform/entities";

/**
 * The model receipt of one assistant turn: the model the provider reported, why the runtime chose
 * it, and, for the system default, since when. All fields come from the runtime. A refused,
 * failed or cancelled turn has no receipt, and a requested model is never shown as served.
 *
 * Arrives two ways, both read here from the message's content blocks:
 *  - live: an `agui.artifact` of type `model_receipt`; the chat stream keeps it as an artifact block
 *    with id `model-receipt-<run_id>` whose content is the receipt JSON;
 *  - on reload: `history_runs[run].model_receipt`, inside the history-provenance artifact block.
 *
 * This is the only module that knows either shape; change the field paths here.
 */

export const MODEL_RECEIPT_ARTIFACT_PREFIX = "model-receipt-";
export const HISTORY_PROVENANCE_KIND = "application/vnd.uar.history-provenance+json";

export interface ModelReceipt {
  /** Provider registry id, from the provider's own response. */
  provider: string;
  /** Model id the provider reported, for example `claude-5-haiku-20261101`. */
  model: string;
  /** Null when the runtime could not state an origin truthfully; nothing is shown then. */
  origin: ModelOrigin | null;
  /** RFC 3339 seconds; set only for `system_default`. */
  since: string | null;
}

const ORIGIN_LABELS: Record<ModelOrigin, string> = {
  system_default: "system default",
  conversation: "this conversation",
  agent: "agent",
  turn: "this turn",
  configured_fallback: "configured fallback",
};

/** The words the console uses for where a model came from. */
export function modelOriginLabel(origin: ModelOrigin): string {
  return ORIGIN_LABELS[origin];
}

function record(value: unknown): Record<string, unknown> | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

function nonEmpty(value: unknown): string | null {
  return typeof value === "string" && value.trim() !== "" ? value : null;
}

/** Decodes one receipt object; null when it does not name a serving provider and model. */
export function parseModelReceipt(value: unknown): ModelReceipt | null {
  const source = record(value);
  if (!source) return null;
  const provider = nonEmpty(source.served_provider);
  const model = nonEmpty(source.served_model);
  if (!provider || !model) return null;
  const origin = isModelOrigin(source.model_origin) ? source.model_origin : null;
  const since = origin === "system_default" ? nonEmpty(source.model_origin_since) : null;
  return { provider, model, origin, since };
}

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

// Every store write reruns each receipt selector; provenance blocks are immutable text, so the
// parsed receipt is remembered by block content instead of parsed again per streamed token.
const PROVENANCE_RECEIPTS = new Map<string, string | null>();
const PROVENANCE_CACHE_LIMIT = 256;

function receiptOfProvenance(content: string): string | null {
  const known = PROVENANCE_RECEIPTS.get(content);
  if (known !== undefined) return known;
  const receipt = record(parseJson(content))?.model_receipt;
  const source = receipt ? JSON.stringify(receipt) : null;
  if (PROVENANCE_RECEIPTS.size >= PROVENANCE_CACHE_LIMIT) PROVENANCE_RECEIPTS.clear();
  PROVENANCE_RECEIPTS.set(content, source);
  return source;
}

/**
 * The receipt JSON of a message, or null. Returned as the raw string so a subscriber compares by
 * value and does not rerender on every store write.
 */
export function receiptSourceOf(content: readonly ContentBlock[]): string | null {
  for (const block of content) {
    if (block.type !== "artifact") continue;
    if (block.id.startsWith(MODEL_RECEIPT_ARTIFACT_PREFIX)) return block.content;
  }
  for (const block of content) {
    if (block.type !== "artifact" || block.kind !== HISTORY_PROVENANCE_KIND) continue;
    const source = receiptOfProvenance(block.content);
    if (source) return source;
  }
  return null;
}

/** Decodes the string returned by `receiptSourceOf`. */
export function parseReceiptSource(source: string | null): ModelReceipt | null {
  return source ? parseModelReceipt(parseJson(source)) : null;
}

const pad = (value: number): string => String(value).padStart(2, "0");

/** Local `HH:MM:SS` of an RFC 3339 time, or null when it does not parse. */
export function formatReceiptTime(since: string): string | null {
  const date = new Date(since);
  if (Number.isNaN(date.getTime())) return null;
  return `${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}
