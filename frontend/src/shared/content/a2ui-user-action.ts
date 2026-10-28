/**
 * A click on an A2UI control starts a run whose input is a JSON envelope:
 * {"type":"a2ui.user_action","sourceRunId":"...","interaction":{"name":...}}.
 * That envelope is a protocol message, not something the user wrote, so the
 * console shows a short label instead. The stored text stays raw: backfill
 * compares history by the exact server text.
 */

const ACTION_PREFIX = /^\s*\{\s*"type"\s*:\s*"a2ui\.user_action"/u;
const NAME_MAX_CODE_POINTS = 40;
const FALLBACK_LABEL = "Action sent";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * Reduces an agent-authored event name to words. The result only contains
 * letters, digits, marks and single spaces, so the Markdown renderer has no
 * link, image, HTML, math or emphasis syntax to act on.
 */
function readableName(name: string): string {
  const words = name
    .replace(/([\p{Ll}\p{N}])(\p{Lu})/gu, "$1 $2")
    .replace(/[^\p{L}\p{M}\p{N}]+/gu, " ")
    .trim();
  const codePoints = Array.from(words);
  return codePoints.length > NAME_MAX_CODE_POINTS
    ? `${codePoints.slice(0, NAME_MAX_CODE_POINTS).join("").trimEnd()}…`
    : words;
}

function actionName(text: string): string {
  try {
    const parsed: unknown = JSON.parse(text);
    if (!isRecord(parsed) || !isRecord(parsed.interaction)) return "";
    const { name } = parsed.interaction;
    return typeof name === "string" ? readableName(name) : "";
  } catch {
    return "";
  }
}

/**
 * Returns the display label for an A2UI user-action envelope, or null when the
 * text is anything else. Once the envelope prefix matches this never returns
 * null, so a malformed or oversized envelope cannot fall through as raw JSON.
 */
export function a2uiUserActionLabel(text: string): string | null {
  if (!ACTION_PREFIX.test(text)) return null;
  const name = actionName(text);
  return name ? `Action: ${name}` : FALLBACK_LABEL;
}
