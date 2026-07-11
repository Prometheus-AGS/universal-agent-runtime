import type { A2uiComponent } from "@/features/a2ui/a2ui-protocol";
import { A2UI_CATALOG_ID, validateA2uiMessage } from "@/features/a2ui/a2ui-protocol";

export interface A2uiSurfaceState {
  id: string;
  catalogId: typeof A2UI_CATALOG_ID;
  components: Record<string, A2uiComponent>;
  data: Record<string, unknown>;
  ready: boolean;
}

export interface A2uiProcessorState {
  surfaces: Record<string, A2uiSurfaceState>;
  error: string | null;
}

export const EMPTY_A2UI_PROCESSOR_STATE: A2uiProcessorState = { surfaces: {}, error: null };

function componentReferences(component: A2uiComponent): string[] {
  switch (component.component) {
    case "Button":
    case "Card":
      return [component.child];
    case "Row":
    case "Column":
      return component.children;
    default:
      return [];
  }
}

function surfaceIsReady(components: Record<string, A2uiComponent>): boolean {
  if (!components.root) return false;
  const visited = new Set<string>();
  const visiting = new Set<string>();
  const visit = (id: string): boolean => {
    if (visiting.has(id)) return false;
    if (visited.has(id)) return true;
    const component = components[id];
    if (!component) return false;
    visiting.add(id);
    const valid = componentReferences(component).every(visit);
    visiting.delete(id);
    visited.add(id);
    return valid;
  };
  return visit("root");
}

function updateAtPath(data: Record<string, unknown>, path: string, value: unknown): Record<string, unknown> {
  const next = structuredClone(data);
  const segments = path.slice(1).split("/").map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
  let parent = next;
  for (const segment of segments.slice(0, -1)) {
    const existing = parent[segment];
    parent[segment] = existing && typeof existing === "object" && !Array.isArray(existing) ? existing : {};
    parent = parent[segment] as Record<string, unknown>;
  }
  parent[segments.at(-1)!] = structuredClone(value);
  return next;
}

/** Deterministically validate and reduce one untrusted A2UI wire message. */
export function reduceA2uiMessage(state: A2uiProcessorState, input: unknown): A2uiProcessorState {
  const validation = validateA2uiMessage(input);
  if (!validation.success) return { ...state, error: validation.error };
  const message = validation.data;
  if ("createSurface" in message) {
    const { surfaceId, catalogId } = message.createSurface;
    return {
      surfaces: {
        ...state.surfaces,
        [surfaceId]: { id: surfaceId, catalogId, components: {}, data: {}, ready: false },
      },
      error: null,
    };
  }
  if ("deleteSurface" in message) {
    const surfaces = { ...state.surfaces };
    delete surfaces[message.deleteSurface.surfaceId];
    return { surfaces, error: null };
  }
  const update = "updateComponents" in message ? message.updateComponents : message.updateDataModel;
  const surface = state.surfaces[update.surfaceId];
  if (!surface) return { ...state, error: `A2UI surface ${update.surfaceId} was not created` };
  if ("components" in update) {
    const components = { ...surface.components };
    for (const component of update.components) components[component.id] = component;
    return {
      surfaces: {
        ...state.surfaces,
        [surface.id]: { ...surface, components, ready: surfaceIsReady(components) },
      },
      error: null,
    };
  }
  return {
    surfaces: {
      ...state.surfaces,
      [surface.id]: { ...surface, data: updateAtPath(surface.data, update.path, update.value) },
    },
    error: null,
  };
}

