import { z } from "zod";
import { AgentToRendererMessageSchema } from "@prometheus-ags/a2ui-core/v1_0";

// Shared declarative contract for runtime surfaces and reusable Presentations.

export const A2UI_PROFILE = "uar.a2ui/1" as const;
export const A2UI_VERSION = "1.0" as const;
export const LEGACY_A2UI_VERSION = "v0.9.1" as const;
export const A2UI_CATALOG_ID = "urn:uar:a2ui:catalog:1" as const;
export const A2UI_BASIC_CATALOG_ID = "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json" as const;

const identifier = z.string().min(1).max(128).regex(/^[A-Za-z0-9][A-Za-z0-9._:-]*$/);
const dataPath = z.object({ path: z.string().startsWith("/").max(512) }).strict();
const dynamicString = z.union([z.string().max(16_384), dataPath]);
const dynamicBoolean = z.union([z.boolean(), dataPath]);
const dynamicStringList = z.union([z.array(z.string().max(1024)).max(100), dataPath]);
const childList = z.array(identifier).max(200);
type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };
const jsonValueSchema: z.ZodType<JsonValue> = z.lazy(() =>
  z.union([
    z.string(),
    z.number(),
    z.boolean(),
    z.null(),
    z.array(jsonValueSchema),
    z.record(z.string(), jsonValueSchema),
  ]),
);

const textComponent = z.object({
  id: identifier,
  component: z.literal("Text"),
  text: dynamicString,
  variant: z.enum(["h1", "h2", "h3", "body", "caption"]).optional(),
}).strict();

const action = z.object({
  event: z.object({
    name: identifier,
    context: z.record(z.string(), jsonValueSchema).optional(),
  }).strict(),
}).strict();

const buttonComponent = z.object({
  id: identifier,
  component: z.literal("Button"),
  child: identifier,
  variant: z.enum(["primary", "secondary", "borderless"]).optional(),
  action,
}).strict();

const textFieldComponent = z.object({
  id: identifier,
  component: z.literal("TextField"),
  label: z.string().min(1).max(256),
  value: dynamicString,
  variant: z.enum(["shortText", "longText", "email", "number"]).optional(),
  placeholder: z.string().max(512).optional(),
}).strict();

const checkBoxComponent = z.object({
  id: identifier,
  component: z.literal("CheckBox"),
  label: z.string().min(1).max(256),
  value: dynamicBoolean,
}).strict();

const choicePickerComponent = z.object({
  id: identifier,
  component: z.literal("ChoicePicker"),
  label: z.string().min(1).max(256),
  value: dynamicStringList,
  variant: z.enum(["mutuallyExclusive", "multipleSelection"]),
  options: z.array(z.object({
    value: z.string().min(1).max(256),
    label: z.string().min(1).max(256),
  }).strict()).min(1).max(100),
}).strict();

const rowComponent = z.object({
  id: identifier,
  component: z.literal("Row"),
  children: childList,
  justify: z.enum(["start", "center", "end", "spaceBetween"]).optional(),
  align: z.enum(["start", "center", "end", "stretch"]).optional(),
}).strict();

const columnComponent = z.object({
  id: identifier,
  component: z.literal("Column"),
  children: childList,
  align: z.enum(["start", "center", "end", "stretch"]).optional(),
}).strict();

const cardComponent = z.object({
  id: identifier,
  component: z.literal("Card"),
  child: identifier,
}).strict();

const dividerComponent = z.object({
  id: identifier,
  component: z.literal("Divider"),
  axis: z.enum(["horizontal", "vertical"]).optional(),
}).strict();

export const a2uiComponentSchema = z.discriminatedUnion("component", [
  textComponent,
  buttonComponent,
  textFieldComponent,
  checkBoxComponent,
  choicePickerComponent,
  rowComponent,
  columnComponent,
  cardComponent,
  dividerComponent,
]);

export type A2uiComponent = z.infer<typeof a2uiComponentSchema>;

const legacyVersion = z.enum(["v0.9", LEGACY_A2UI_VERSION]);
const legacyEnvelope = { version: legacyVersion, profile: z.literal(A2UI_PROFILE).optional() };
const legacyCatalog = z.enum([A2UI_CATALOG_ID, "https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json"]);
const legacyMessageSchema = z.union([
  z.object({
    ...legacyEnvelope,
    createSurface: z.object({
      surfaceId: identifier,
      catalogId: legacyCatalog,
    }).strict(),
  }).strict(),
  z.object({
    ...legacyEnvelope,
    updateComponents: z.object({
      surfaceId: identifier,
      components: z.array(a2uiComponentSchema).min(1).max(500),
    }).strict(),
  }).strict(),
  z.object({
    ...legacyEnvelope,
    updateDataModel: z.object({
      surfaceId: identifier,
      path: z.string().startsWith("/").max(512),
      value: jsonValueSchema,
    }).strict(),
  }).strict(),
  z.object({
    ...legacyEnvelope,
    deleteSurface: z.object({ surfaceId: identifier }).strict(),
  }).strict(),
]);

const finalCatalog = z.enum([A2UI_CATALOG_ID, A2UI_BASIC_CATALOG_ID]);
const finalId = z.string().max(128).regex(/^[\p{XID_Start}_]\p{XID_Continue}*$/u);
const finalSurfaceId = z.string().max(128).refine((value) => value.trim().length > 0);
// The local DataModel collapses empty path segments. Leave room for its
// root spellings and reject ambiguous paths before reading or writing data.
const finalPath = z.string().max(512).regex(/^(?:\/?|(?:\/(?:[^~/]|~[01])+)*)$/u);
const finalBinding = z.object({ "@path": finalPath }).strict();
const finalString = z.union([z.string().max(16_384), finalBinding]);
const finalBoolean = z.union([z.boolean(), finalBinding]);
const finalStringList = z.union([z.array(z.string().max(1024)).max(100), finalBinding]);
const finalComponentBase = { id: finalId, "@catalogId": finalCatalog.optional() };
const finalChildren = z.array(finalId).max(200);
const finalJustify = z.enum(["start", "center", "end", "spaceBetween", "spaceAround", "spaceEvenly", "stretch"]).optional();
export const finalA2uiComponentSchema = z.discriminatedUnion("component", [
  textComponent.extend({
    ...finalComponentBase,
    text: finalString,
    variant: z.enum(["body", "caption"]).optional(),
  }),
  buttonComponent.extend({
    ...finalComponentBase,
    child: finalId,
    variant: z.enum(["default", "primary", "borderless"]).optional(),
    action: z.object({ event: action.shape.event.extend({
      name: z.string().max(128).refine((value) => value.trim().length > 0),
      userMessage: finalString.optional(),
    }) }).strict(),
  }),
  textFieldComponent.extend({
    ...finalComponentBase,
    label: finalString,
    value: finalString,
    placeholder: finalString.optional(),
    variant: z.enum(["longText", "number", "shortText", "obscured"]).optional(),
  }),
  checkBoxComponent.extend({ ...finalComponentBase, label: finalString, value: finalBoolean }),
  choicePickerComponent.extend({
    ...finalComponentBase,
    label: finalString.optional(),
    value: finalStringList,
    variant: choicePickerComponent.shape.variant.optional(),
    options: z.array(choicePickerComponent.shape.options.element.extend({ label: finalString, value: z.string().max(256) })).max(100),
  }),
  rowComponent.extend({ ...finalComponentBase, children: finalChildren, justify: finalJustify }),
  columnComponent.extend({ ...finalComponentBase, children: finalChildren, justify: finalJustify }),
  cardComponent.extend({ ...finalComponentBase, child: finalId }),
  dividerComponent.extend(finalComponentBase),
]);

const finalEnvelope = { version: z.literal(A2UI_VERSION) };
const finalComponents = z.array(finalA2uiComponentSchema).min(1).max(500);
const finalMessageSchema = z.union([
  z.object({
    ...finalEnvelope,
    createSurface: z.object({
      surfaceId: finalSurfaceId,
      catalogId: finalCatalog,
      sendDataModel: z.literal(false).optional(),
      components: finalComponents.optional(),
      dataModel: z.record(z.string(), jsonValueSchema).optional(),
    }).strict(),
  }).strict(),
  z.object({
    ...finalEnvelope,
    updateComponents: z.object({ surfaceId: finalSurfaceId, components: finalComponents }).strict(),
  }).strict(),
  z.object({
    ...finalEnvelope,
    updateDataModel: z.object({
      surfaceId: finalSurfaceId,
      path: finalPath.optional(),
      value: jsonValueSchema,
    }).strict(),
  }).strict(),
  z.object({ ...finalEnvelope, deleteSurface: z.object({ surfaceId: finalSurfaceId }).strict() }).strict(),
]);

export const a2uiMessageSchema = z.union([legacyMessageSchema, finalMessageSchema]);
export type A2uiMessage = z.infer<typeof a2uiMessageSchema>;
export type A2uiSurfaceComponent = A2uiComponent | z.infer<typeof finalA2uiComponentSchema>;
export type A2uiVersion = A2uiMessage["version"];

export function a2uiMessageComponents(message: A2uiMessage): A2uiSurfaceComponent[] {
  if ("updateComponents" in message) return message.updateComponents.components;
  if (message.version === A2UI_VERSION && "createSurface" in message) return message.createSurface.components ?? [];
  return [];
}

export interface A2uiArtifactSchema {
  schema_id: string;
  title: string;
  description: string;
  artifact_type: "form" | "confirm" | "select" | "text_input" | "display" | "chart" | "media";
  json_schema: unknown;
  render_hint?: string;
  builtin: boolean;
}

export interface A2uiTestTriggerPayload {
  artifact_type: string;
  title: string;
  content: string;
  metadata?: Record<string, unknown>;
}

const EXECUTABLE_CONTENT = /<\/?[a-z][^>]*>|javascript\s*:|data\s*:\s*text\/html|\bon[a-z]+\s*=/i;
const FORBIDDEN_DATA_KEYS = new Set(["__proto__", "constructor", "prototype"]);

function unsafePath(path: string): boolean {
  return path.split("/").some((part) => FORBIDDEN_DATA_KEYS.has(part.replaceAll("~1", "/").replaceAll("~0", "~")));
}

function validateJsonTree(value: unknown, depth = 0): string | undefined {
  if (depth > 64) return "A2UI data exceeds the nesting limit";
  if (typeof value === "string" && EXECUTABLE_CONTENT.test(value)) {
    return "Executable HTML or JavaScript is not allowed in A2UI data";
  }
  if (!value || typeof value !== "object") return;
  for (const [key, child] of Object.entries(value)) {
    if (FORBIDDEN_DATA_KEYS.has(key)) return "Prototype properties are not allowed in A2UI data";
    const error = validateJsonTree(child, depth + 1);
    if (error) return error;
  }
}

function validateComponentDirectives(value: unknown, modern: boolean): string | undefined {
  if (!value || typeof value !== "object") return;
  const bindingKey = modern ? "@path" : "path";
  const fields = Object.entries(value);
  for (const [key, child] of fields) {
    if (key === "@call" || (!modern && (key === "functionCall" || key === "call" && typeof child === "string"))) {
      return "A2UI function calls are unavailable";
    }
    if (key === bindingKey) {
      if (modern && (fields.length !== 1 || typeof child !== "string" || !finalPath.safeParse(child).success)) {
        return "Invalid A2UI data binding";
      }
      if (typeof child === "string" && unsafePath(child)) return "Invalid A2UI data binding";
    }
    if (modern && key.startsWith("@") && !key.startsWith("@@") && key !== "@path" && key !== "@catalogId") {
      return "Unsupported A2UI component directive";
    }
    const error = validateComponentDirectives(child, modern);
    if (error) return error;
  }
}

export type A2uiValidationResult =
  | { success: true; data: A2uiMessage }
  | { success: false; error: string };

/** Validate structural and trust-boundary constraints before reducing a message. */
export function validateA2uiMessage(value: unknown): A2uiValidationResult {
  const source = value as { createSurface?: { components?: unknown }; updateComponents?: { components?: unknown } } | null;
  const incomingComponents = source?.createSurface?.components ?? source?.updateComponents?.components;
  if (Array.isArray(incomingComponents) && incomingComponents.length > 500) {
    return { success: false, error: "A2UI component updates are limited to 500 components" };
  }
  const jsonError = validateJsonTree(value);
  if (jsonError) return { success: false, error: jsonError };
  const parsed = a2uiMessageSchema.safeParse(value);
  if (!parsed.success) return { success: false, error: parsed.error.issues[0]?.message ?? "Invalid A2UI message" };
  if (parsed.data.version === A2UI_VERSION) {
    if (!AgentToRendererMessageSchema.safeParse(parsed.data).success) {
      return { success: false, error: "Invalid A2UI 1.0 message" };
    }
  }
  if ("updateDataModel" in parsed.data && unsafePath(parsed.data.updateDataModel.path ?? "/")) {
    return { success: false, error: "Prototype properties are not allowed in A2UI data paths" };
  }
  const components = a2uiMessageComponents(parsed.data);
  const ids = components.map((component) => component.id);
  if (new Set(ids).size !== ids.length) return { success: false, error: "A2UI component IDs must be unique within an update" };
  for (const component of components) {
    const error = validateComponentDirectives(component, parsed.data.version === A2UI_VERSION);
    if (error) return { success: false, error };
  }
  return { success: true, data: parsed.data };
}
