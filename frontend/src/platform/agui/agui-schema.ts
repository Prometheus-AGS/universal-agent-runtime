export interface UarAguiEvent extends Record<string, unknown> {
  type: string;
  profile: "uar.agui/1";
  eventId: string;
  sequence: number;
}

const EVENT_TYPES = new Set([
  "RUN_STARTED", "RUN_FINISHED", "RUN_ERROR", "STEP_STARTED", "STEP_FINISHED",
  "TEXT_MESSAGE_START", "TEXT_MESSAGE_CONTENT", "TEXT_MESSAGE_END",
  "REASONING_START", "REASONING_MESSAGE_START", "REASONING_MESSAGE_CONTENT",
  "REASONING_MESSAGE_END", "REASONING_END", "TOOL_CALL_START", "TOOL_CALL_ARGS",
  "TOOL_CALL_END", "TOOL_CALL_RESULT", "STATE_SNAPSHOT", "STATE_DELTA",
  "MESSAGES_SNAPSHOT", "RAW", "CUSTOM",
]);

export function isUarAguiEvent(value: unknown): value is UarAguiEvent {
  if (!value || typeof value !== "object") return false;
  const event = value as Record<string, unknown>;
  if (
    typeof event.type !== "string" || !EVENT_TYPES.has(event.type) ||
    event.profile !== "uar.agui/1" || typeof event.eventId !== "string" ||
    event.eventId.length === 0 || !Number.isInteger(event.sequence) ||
    (event.sequence as number) < 0
  ) return false;
  if (event.type === "CUSTOM") {
    return typeof event.name === "string" && event.name.startsWith("uar.") && "value" in event;
  }
  if (event.type === "STATE_DELTA") return Array.isArray(event.delta);
  return true;
}
