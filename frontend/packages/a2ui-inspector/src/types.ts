import type { SurfaceModel } from "@prometheus-ags/a2ui-core/v0_9";
import type { UarComponentImplementation } from "@prometheus-ags/a2ui-uar";

export type InspectorConnection = "idle" | "connecting" | "connected" | "disconnected" | "error";
export interface InspectedMessage { id: number; receivedAt: string; raw: unknown; valid: boolean; error?: string; kind: string; }
export type InspectorSurface = SurfaceModel<UarComponentImplementation>;
