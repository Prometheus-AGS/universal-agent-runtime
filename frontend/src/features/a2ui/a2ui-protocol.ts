import { z } from "zod";

export const A2UI_PROFILE = "uar.a2ui/1" as const;
export const A2UI_VERSION = "v0.9.1" as const;
export const A2UI_CATALOG_ID = "urn:uar:a2ui:catalog:1" as const;

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

export const a2uiMessageSchema = z.union([
  z.object({
    version: z.literal(A2UI_VERSION),
    profile: z.literal(A2UI_PROFILE),
    createSurface: z.object({
      surfaceId: identifier,
      catalogId: z.literal(A2UI_CATALOG_ID),
    }).strict(),
  }).strict(),
  z.object({
    version: z.literal(A2UI_VERSION),
    profile: z.literal(A2UI_PROFILE),
    updateComponents: z.object({
      surfaceId: identifier,
      components: z.array(a2uiComponentSchema).min(1).max(500),
    }).strict(),
  }).strict(),
  z.object({
    version: z.literal(A2UI_VERSION),
    profile: z.literal(A2UI_PROFILE),
    updateDataModel: z.object({
      surfaceId: identifier,
      path: z.string().startsWith("/").max(512),
      value: jsonValueSchema,
    }).strict(),
  }).strict(),
  z.object({
    version: z.literal(A2UI_VERSION),
    profile: z.literal(A2UI_PROFILE),
    deleteSurface: z.object({ surfaceId: identifier }).strict(),
  }).strict(),
]);

export type A2uiMessage = z.infer<typeof a2uiMessageSchema>;

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

function containsExecutableContent(value: unknown): boolean {
  if (typeof value === "string") return EXECUTABLE_CONTENT.test(value);
  if (Array.isArray(value)) return value.some(containsExecutableContent);
  if (value && typeof value === "object") {
    return Object.values(value as Record<string, unknown>).some(containsExecutableContent);
  }
  return false;
}

export type A2uiValidationResult =
  | { success: true; data: A2uiMessage }
  | { success: false; error: string };

/** Validate structural and trust-boundary constraints before reducing a message. */
export function validateA2uiMessage(value: unknown): A2uiValidationResult {
  const parsed = a2uiMessageSchema.safeParse(value);
  if (!parsed.success) return { success: false, error: parsed.error.issues[0]?.message ?? "Invalid A2UI message" };
  if (containsExecutableContent(parsed.data)) {
    return { success: false, error: "Executable HTML or JavaScript is not allowed in A2UI data" };
  }
  if ("updateComponents" in parsed.data) {
    const ids = parsed.data.updateComponents.components.map((component) => component.id);
    if (new Set(ids).size !== ids.length) {
      return { success: false, error: "A2UI component IDs must be unique within an update" };
    }
  }
  return { success: true, data: parsed.data };
}
