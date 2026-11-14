import { DataModel } from "@prometheus-ags/a2ui-core/v0_9";
import { A2UI_VERSION, a2uiMessageComponents, validateA2uiMessage, type A2uiSurfaceComponent, type A2uiVersion } from "@/features/a2ui/a2ui-protocol";

export interface A2uiSurfaceState {
  id: string;
  catalogId: string;
  version: A2uiVersion;
  components: Record<string, A2uiSurfaceComponent>;
  data: unknown;
  ready: boolean;
}

export interface A2uiProcessorState {
  surfaces: Record<string, A2uiSurfaceState>;
  error: string | null;
}

export const EMPTY_A2UI_PROCESSOR_STATE: A2uiProcessorState = { surfaces: {}, error: null };

function componentReferences(component: A2uiSurfaceComponent): string[] {
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

export function a2uiSharedChildError(childIds: Iterable<string>): string | undefined {
  const referenced = new Set<string>();
  for (const id of childIds) {
    if (id === "root" || referenced.has(id)) return `A2UI component ${id} must have exactly one parent`;
    referenced.add(id);
  }
}

function surfaceIsReady(components: Record<string, A2uiSurfaceComponent>): boolean {
  if (!components.root) return false;
  const visited = new Set<string>();
  const visiting = new Set<string>();
  const visit = (id: string): boolean => {
    if (visiting.has(id)) return false;
    if (visited.has(id)) return true;
    const component = Object.hasOwn(components, id) ? components[id] : undefined;
    if (!component) return false;
    visiting.add(id);
    const valid = componentReferences(component).every(visit);
    visiting.delete(id);
    visited.add(id);
    return valid;
  };
  return visit("root");
}

function updateLegacyData(data: Record<string, unknown>, path: string, value: unknown): Record<string, unknown> {
  const copy = structuredClone(data);
  const parts = path.slice(1).split("/").map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
  let parent = copy;
  for (const part of parts.slice(0, -1)) {
    const child = Object.hasOwn(parent, part) ? parent[part] : undefined;
    parent[part] = child && typeof child === "object" && !Array.isArray(child) ? child : {};
    parent = parent[part] as Record<string, unknown>;
  }
  parent[parts.at(-1)!] = structuredClone(value);
  return copy;
}

/** Deterministically validate and reduce one untrusted A2UI wire message. */
export function reduceA2uiMessage(state: A2uiProcessorState, input: unknown): A2uiProcessorState {
  const validation = validateA2uiMessage(input);
  if (!validation.success) return { ...state, error: validation.error };
  const message = validation.data;
  if ("createSurface" in message) {
    const { surfaceId, catalogId } = message.createSurface;
    if (Object.hasOwn(state.surfaces, surfaceId) && (message.version === A2UI_VERSION || state.surfaces[surfaceId]?.version === A2UI_VERSION)) {
      return { ...state, error: `A2UI surface ${surfaceId} already exists` };
    }
    const components = Object.fromEntries(a2uiMessageComponents(message).map((component) => [component.id, component]));
    if (message.version === A2UI_VERSION) {
      const error = a2uiSharedChildError(Object.values(components).flatMap(componentReferences));
      if (error) return { ...state, error };
    }
    const data = message.version === A2UI_VERSION ? structuredClone(message.createSurface.dataModel ?? {}) : {};
    return {
      surfaces: {
        ...state.surfaces,
        [surfaceId]: { id: surfaceId, catalogId, version: message.version, components, data, ready: surfaceIsReady(components) },
      },
      error: null,
    };
  }
  const update = "updateComponents" in message ? message.updateComponents
    : "updateDataModel" in message ? message.updateDataModel : message.deleteSurface;
  const surface = Object.hasOwn(state.surfaces, update.surfaceId) ? state.surfaces[update.surfaceId] : undefined;
  if ((!surface && !("deleteSurface" in message && message.version !== A2UI_VERSION))
    || (surface && (surface.version === A2UI_VERSION) !== (message.version === A2UI_VERSION))) {
    return { ...state, error: `A2UI ${message.version} message does not match an active surface ${update.surfaceId}` };
  }
  if ("deleteSurface" in message) {
    const surfaces = { ...state.surfaces };
    delete surfaces[update.surfaceId];
    return { surfaces, error: null };
  }
  if (!surface) return state;
  if ("updateComponents" in message) {
    const components = {
      ...surface.components,
      ...Object.fromEntries(message.updateComponents.components.map((component) => [component.id, component])),
    };
    if (message.version === A2UI_VERSION) {
      const error = a2uiSharedChildError(Object.values(components).flatMap(componentReferences));
      if (error) return { ...state, error };
    }
    return {
      surfaces: {
        ...state.surfaces,
        [surface.id]: { ...surface, components, ready: surfaceIsReady(components) },
      },
      error: null,
    };
  }
  if ("updateDataModel" in message) {
    if (message.version !== A2UI_VERSION) {
      const { path, value } = message.updateDataModel;
      const data = updateLegacyData(surface.data as Record<string, unknown>, path, value);
      return { surfaces: { ...state.surfaces, [surface.id]: { ...surface, data } }, error: null };
    }
    try {
      const { path, value } = message.updateDataModel;
      const dataModel = new DataModel();
      dataModel.set("/", structuredClone(surface.data));
      dataModel.set(path ?? "/", structuredClone(value));
      return { surfaces: { ...state.surfaces, [surface.id]: { ...surface, data: dataModel.get("/") } }, error: null };
    } catch (error) {
      return { ...state, error: error instanceof Error ? error.message : "Invalid A2UI data model update" };
    }
  }
  return state;
}

