import { A2uiMessageSchema, MessageProcessor, type SurfaceModel } from "@prometheus-ags/a2ui-core/v0_9";
import { a2uiBasicCatalogV1, uarBasicCatalog, uarBasicCatalogV1, type UarComponentImplementation } from "@prometheus-ags/a2ui-uar";
import {
  A2UI_CATALOG_ID,
  A2UI_VERSION,
  a2uiMessageComponents,
  validateA2uiMessage,
  type A2uiSurfaceComponent,
  type A2uiVersion,
} from "./a2ui-protocol";
import { MAX_A2UI_COMPONENTS, MAX_A2UI_MESSAGES, MAX_A2UI_SOURCE_BYTES, MAX_A2UI_SURFACES } from "./a2ui-rendering-limits";
import { a2uiSharedChildError } from "./a2ui-surface-processor";

export interface StandardSurface {
  surface: SurfaceModel<UarComponentImplementation>;
  version: A2uiVersion;
}

export function parseA2uiMessageSource(content: string): unknown[] {
  if (new TextEncoder().encode(content).byteLength > MAX_A2UI_SOURCE_BYTES) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_SOURCE_BYTES / 1024} KiB rendering limit.`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(content);
  } catch {
    const lines = content.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
    parsed = lines.map((line) => JSON.parse(line) as unknown);
  }

  const messages = Array.isArray(parsed) ? parsed
    : parsed && typeof parsed === "object" && "messages" in parsed && Array.isArray(parsed.messages) ? parsed.messages
    : [parsed];
  if (messages.length === 0) throw new Error("The artifact contains no A2UI messages.");
  if (messages.length > MAX_A2UI_MESSAGES) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_MESSAGES}-message rendering limit.`);
  }
  return messages;
}

function rendererComponent(component: A2uiSurfaceComponent) {
  if ("@catalogId" in component) {
    const { "@catalogId": catalogId, ...properties } = component;
    return { ...properties, catalogId };
  }
  return component;
}

/** Process verified wire shapes through the UAR renderer's existing state and binder. */
export function processA2uiDisplayContent(content: string) {
  const messages = parseA2uiMessageSource(content).map((raw) => {
    const validated = validateA2uiMessage(raw);
    if (!validated.success) throw new Error(validated.error);
    return validated.data;
  });
  const componentCount = messages.reduce((count, message) => count + a2uiMessageComponents(message).length, 0);
  if (componentCount > MAX_A2UI_COMPONENTS) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_COMPONENTS}-component rendering limit.`);
  }

  // The same catalog identifier can describe different protocol versions.
  // Give each generation the matching schema and the same UAR React widgets.
  const validationConfig = { allowMissingRoot: true, allowDanglingReferences: true, allowOrphanComponents: true };
  const legacy = new MessageProcessor<UarComponentImplementation>([uarBasicCatalog], undefined, { validationConfig });
  const modern = new MessageProcessor<UarComponentImplementation>([uarBasicCatalogV1, a2uiBasicCatalogV1]);
  const active = new Map<string, A2uiVersion>();
  const seenVersions = new Set<A2uiVersion>();
  let created = 0;
  let deleted = false;
  for (const message of messages) {
    const { version } = message;
    const isModern = version === A2UI_VERSION;
    const processor = isModern ? modern : legacy;
    const payload = "createSurface" in message ? message.createSurface
      : "updateComponents" in message ? message.updateComponents
      : "updateDataModel" in message ? message.updateDataModel
      : message.deleteSurface;
    const { surfaceId } = payload;
    const activeVersion = active.get(surfaceId);
    if ("createSurface" in message) {
      if (activeVersion && (isModern || activeVersion === A2UI_VERSION)) throw new Error(`A2UI surface ${surfaceId} already exists`);
      if (++created > MAX_A2UI_SURFACES) {
        throw new Error(`A2UI source exceeds the ${MAX_A2UI_SURFACES}-surface rendering limit.`);
      }
      active.set(surfaceId, version);
    } else if ((!activeVersion && !("deleteSurface" in message)) || (activeVersion && (activeVersion === A2UI_VERSION) !== isModern)) {
      throw new Error(`A2UI ${version} message does not match an active surface ${surfaceId}`);
    }
    seenVersions.add(version);

    if (message.version === A2UI_VERSION) {
      if ("createSurface" in message) {
        const { components, ...create } = message.createSurface;
        // The installed renderer's internal operations support inline state.
        // The trust boundary above admits only the published 1.0 wire shape.
        processor.processMessages({ type: "createSurface", version, ...create, components: components?.map(rendererComponent) });
      } else if ("updateComponents" in message) {
        processor.processMessages({ type: "updateComponents", surfaceId, components: message.updateComponents.components.map(rendererComponent) });
      } else if ("updateDataModel" in message) {
        processor.processMessages({ type: "updateDataModel", ...message.updateDataModel });
      } else {
        processor.processMessages({ type: "deleteSurface", surfaceId });
      }
    } else {
      const wire = { ...message };
      delete wire.profile;
      if ("createSurface" in wire) {
        if (activeVersion) processor.processMessages({ type: "deleteSurface", surfaceId });
        processor.processMessages(A2uiMessageSchema.parse({ ...wire, createSurface: { ...wire.createSurface, catalogId: A2UI_CATALOG_ID } }));
      } else {
        processor.processMessages(A2uiMessageSchema.parse(wire));
      }
    }

    if ("deleteSurface" in message) {
      active.delete(surfaceId);
      deleted = true;
    }
  }

  const surfaces: StandardSurface[] = [];
  for (const [id, version] of active) {
    const surface = (version === A2UI_VERSION ? modern : legacy).model.getSurface(id);
    if (surface) {
      if (version === A2UI_VERSION) {
        const children = [...surface.componentsModel.entries].flatMap(([id]) => surface.componentsModel.getChildIds(id));
        const sharedChildError = a2uiSharedChildError(children);
        if (sharedChildError) throw new Error(sharedChildError);
        // The local processor's envelope preflight treats every `path` key
        // as a legacy binding. We validated the versioned wire shape already.
        surface.componentsModel.validateTopology({ allowDanglingReferences: true, allowOrphanComponents: true });
      }
      surfaces.push({ surface, version });
    }
  }
  return { surfaces, deleted: deleted && surfaces.length === 0, versions: [...seenVersions] };
}
