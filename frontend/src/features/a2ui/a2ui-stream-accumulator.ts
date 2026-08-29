import {
  MAX_A2UI_COMPONENTS,
  MAX_A2UI_MESSAGES,
  MAX_A2UI_SOURCE_BYTES,
  MAX_A2UI_SURFACES,
} from "./a2ui-rendering-limits";

type CurrentA2uiEnvelopeType =
  | "createSurface"
  | "updateComponents"
  | "updateDataModel"
  | "deleteSurface";

const CURRENT_A2UI_ENVELOPE_KEYS: CurrentA2uiEnvelopeType[] = [
  "createSurface",
  "updateComponents",
  "updateDataModel",
  "deleteSurface",
];

const MAX_REJECTED_FRAME_EXCERPT_BYTES = 1024;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function envelopeComponentCount(envelope: Record<string, unknown>): number {
  const update = envelope.updateComponents;
  return isRecord(update) && Array.isArray(update.components) ? update.components.length : 0;
}

interface SurfaceAccumulatorState {
  bytes: number;
  components: number;
  displayed: boolean;
  generation: number;
  messages: Record<string, unknown>[];
  rejected: boolean;
}

export interface AccumulatedA2uiSurface {
  action: "none" | "create" | "update" | "reject";
  diagnosticSource?: string;
  displayed: boolean;
  error?: string;
  generation: number;
  type: CurrentA2uiEnvelopeType;
  surfaceId: string;
  messages: Record<string, unknown>[];
  profile: string;
  version: string;
}

function rejectedFrameDiagnostic(
  envelope: Record<string, unknown>,
  acceptedMessageCount: number,
  error: string,
): string {
  const sourceBytes = new TextEncoder().encode(JSON.stringify(envelope));
  const excerptBytes = sourceBytes.slice(0, MAX_REJECTED_FRAME_EXCERPT_BYTES);
  return JSON.stringify({
    error,
    acceptedMessageCount,
    rejectedFrameExcerpt: new TextDecoder().decode(excerptBytes),
    rejectedFrameTruncated: sourceBytes.byteLength > excerptBytes.byteLength,
  });
}

/** Keeps current A2UI messages ordered and bounded by surface across AG-UI frames. */
export class A2uiStreamAccumulator {
  private readonly activeSurfaces = new Map<string, SurfaceAccumulatorState>();
  private readonly generations = new Map<string, number>();
  private lifecycleCount = 0;
  private rejectedSurfaceLimit = false;

  clear() {
    this.activeSurfaces.clear();
    this.generations.clear();
    this.lifecycleCount = 0;
    this.rejectedSurfaceLimit = false;
  }

  advance(envelope: Record<string, unknown>): AccumulatedA2uiSurface | null {
    const type = CURRENT_A2UI_ENVELOPE_KEYS.find((key) => key in envelope);
    if (!type) return null;
    const payload = envelope[type];
    if (!isRecord(payload) || typeof payload.surfaceId !== "string") return null;

    const surfaceId = payload.surfaceId;
    let state = this.activeSurfaces.get(surfaceId);
    if (!state) {
      if (this.lifecycleCount >= MAX_A2UI_SURFACES) {
        if (this.rejectedSurfaceLimit) return null;
        this.rejectedSurfaceLimit = true;
        return {
          action: "reject",
          diagnosticSource: rejectedFrameDiagnostic(
            envelope,
            0,
            `A2UI stream exceeds the ${MAX_A2UI_SURFACES}-surface rendering limit.`,
          ),
          displayed: false,
          error: `A2UI stream exceeds the ${MAX_A2UI_SURFACES}-surface rendering limit.`,
          generation: (this.generations.get(surfaceId) ?? 0) + 1,
          type,
          surfaceId,
          messages: [],
          profile: "",
          version: "",
        };
      }
      const generation = (this.generations.get(surfaceId) ?? 0) + 1;
      this.generations.set(surfaceId, generation);
      this.lifecycleCount += 1;
      state = {
        bytes: 0,
        components: 0,
        displayed: false,
        generation,
        messages: [],
        rejected: false,
      };
      this.activeSurfaces.set(surfaceId, state);
    }
    if (state.rejected) return null;

    const envelopeBytes = new TextEncoder().encode(JSON.stringify(envelope)).byteLength;
    const nextBytes = state.bytes + envelopeBytes;
    const nextMessages = state.messages.length + 1;
    const nextComponents = state.components + envelopeComponentCount(envelope);
    const limitError = nextBytes > MAX_A2UI_SOURCE_BYTES
      ? `A2UI stream exceeds the ${MAX_A2UI_SOURCE_BYTES / 1024} KiB rendering limit.`
      : nextMessages > MAX_A2UI_MESSAGES
        ? `A2UI stream exceeds the ${MAX_A2UI_MESSAGES}-message rendering limit.`
        : nextComponents > MAX_A2UI_COMPONENTS
          ? `A2UI stream exceeds the ${MAX_A2UI_COMPONENTS}-component rendering limit.`
          : null;
    if (limitError) {
      state.rejected = true;
      return this.result(
        state,
        type,
        surfaceId,
        "reject",
        limitError,
        rejectedFrameDiagnostic(envelope, state.messages.length, limitError),
      );
    }

    state.bytes = nextBytes;
    state.components = nextComponents;
    state.messages = [...state.messages, envelope];
    const renderReady = state.messages.some((message) => "updateComponents" in message);
    const action = state.displayed ? "update" : renderReady ? "create" : "none";
    if (action === "create") state.displayed = true;
    const result = this.result(state, type, surfaceId, action);
    if (type === "deleteSurface") this.activeSurfaces.delete(surfaceId);
    return result;
  }

  private result(
    state: SurfaceAccumulatorState,
    type: CurrentA2uiEnvelopeType,
    surfaceId: string,
    action: AccumulatedA2uiSurface["action"],
    error?: string,
    diagnosticSource?: string,
  ): AccumulatedA2uiSurface {
    const profileMessage = state.messages.find((message) => typeof message.profile === "string");
    const versionMessage = state.messages.find((message) => typeof message.version === "string");
    return {
      action,
      diagnosticSource,
      displayed: state.displayed,
      error,
      generation: state.generation,
      type,
      surfaceId,
      messages: state.messages,
      profile: typeof profileMessage?.profile === "string" ? profileMessage.profile : "",
      version: typeof versionMessage?.version === "string" ? versionMessage.version : "",
    };
  }
}
