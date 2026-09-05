import { z } from "zod";
import {
  A2UI_CATALOG_ID,
  A2UI_PROFILE,
  A2UI_VERSION,
  a2uiComponentSchema,
  validateA2uiMessage,
} from "@/platform/a2ui/protocol";

export const PRESENTATION_ENTITY = "Presentation";
export const PRESENTATION_DRAFT_ENTITY = "PresentationDraft";
export const PRESENTATION_CATALOG_ENTITY = "PresentationCatalog";

export const presentationTemplateSchema = z.object({
  version: z.literal(A2UI_VERSION),
  catalog_id: z.literal(A2UI_CATALOG_ID),
  components: z.array(a2uiComponentSchema).min(1).max(500),
  default_data: z.record(z.unknown()).default({}),
}).strict();

const presentationContentSchema = z.object({
  title: z.string().refine((value) => value.trim().length > 0, "Enter a title."),
  description: z.string(),
  enabled: z.boolean(),
  template: presentationTemplateSchema,
}).strict();

export const presentationSchema = z.object({
  id: z.string().min(1),
  owner_id: z.string().min(1),
  revision: z.number().int().positive().max(Number.MAX_SAFE_INTEGER),
  content: presentationContentSchema,
  created_at: z.string(),
  updated_at: z.string(),
}).strict();

export const presentationCatalogSchema = z.object({
  owner_id: z.string().min(1),
  presentations: z.array(presentationSchema),
}).strict();

export type PresentationTemplate = z.infer<typeof presentationTemplateSchema>;
export type PresentationContent = z.infer<typeof presentationContentSchema>;
export type Presentation = z.infer<typeof presentationSchema> & Record<string, unknown>;
export type PresentationCatalogResponse = z.infer<typeof presentationCatalogSchema>;
export type PresentationDraftField = "title" | "description" | "enabled" | "source";

export interface PresentationDraft extends Record<string, unknown> {
  id: string;
  owner_id: string;
  presentation_id: string | null;
  expected_revision: number | null;
  title: string;
  description: string;
  enabled: boolean;
  source: string;
  dirty: boolean;
  status: "idle" | "saving" | "saved" | "error";
  error: string | null;
  uncertain: boolean;
  conflict: boolean;
}

export interface PresentationCatalog extends Record<string, unknown> {
  id: string;
  owner_id: string | null;
  generation: number;
  status: "loading" | "ready" | "error";
  error: string | null;
  editor_open: boolean;
}

export const STARTER_PRESENTATION_SOURCE = JSON.stringify({
  version: A2UI_VERSION,
  catalog_id: A2UI_CATALOG_ID,
  components: [{ id: "root", component: "Text", text: { path: "/message" } }],
  default_data: { message: "Ready" },
}, null, 2);

const RESERVED_KEYS = new Set(["__proto__", "prototype", "constructor"]);

function validateKeys(value: unknown): void {
  if (!value || typeof value !== "object") return;
  for (const [key, child] of Object.entries(value)) {
    if (RESERVED_KEYS.has(key)) throw new Error("Prototype property names are not allowed.");
    if (`/${key.replaceAll("~", "~0").replaceAll("/", "~1")}`.length > 512) {
      throw new Error("Data keys must fit the 512-character path limit.");
    }
    validateKeys(child);
  }
}

/** Reject unsafe or non-renderable author input before mounting a preview. */
export function parsePresentationSource(source: string):
  | { template: PresentationTemplate; error: null }
  | { template: null; error: string } {
  try {
    const raw: unknown = JSON.parse(source);
    validateKeys(raw);
    const result = presentationTemplateSchema.safeParse(raw);
    if (!result.success) {
      const issue = result.error.issues[0];
      return { template: null, error: `${issue?.path.join(".") || "Template"}: ${issue?.message ?? "Invalid template"}. See supported components below.` };
    }
    const template = result.data;
    const validation = validateA2uiMessage({
      version: A2UI_VERSION,
      profile: A2UI_PROFILE,
      updateComponents: { surfaceId: "presentation-preview", components: template.components },
    });
    if (!validation.success) throw new Error(validation.error);
    for (const [key, value] of Object.entries(template.default_data)) {
      const data = validateA2uiMessage({
        version: A2UI_VERSION,
        profile: A2UI_PROFILE,
        updateDataModel: {
          surfaceId: "presentation-preview",
          path: `/${key.replaceAll("~", "~0").replaceAll("/", "~1")}`,
          value,
        },
      });
      if (!data.success) throw new Error(data.error);
    }
    const graph = new Map(template.components.map((component) => [component.id, component]));
    const referenced = new Set<string>();
    for (const component of template.components) {
      for (const field of ["text", "value"] as const) {
        const binding = field in component ? (component as Record<string, unknown>)[field] : null;
        if (binding && typeof binding === "object" && "path" in binding) {
          const parts = String(binding.path).slice(1).split("/").map((part) => part.replaceAll("~1", "/").replaceAll("~0", "~"));
          if (parts.some((part) => RESERVED_KEYS.has(part))) throw new Error("Bindings cannot reference prototype properties.");
        }
      }
      const children = "children" in component ? component.children : "child" in component ? [component.child] : [];
      for (const child of children) {
        if (!graph.has(child)) throw new Error(`${component.id} references missing component ${child}.`);
        if (child === "root" || referenced.has(child)) throw new Error(`${child} must have exactly one parent.`);
        referenced.add(child);
      }
    }
    const visited = new Set<string>();
    const pending = ["root"];
    while (pending.length) {
      const id = pending.pop()!;
      const component = graph.get(id);
      if (!component) throw new Error("Add a component with id \"root\".");
      if (visited.has(id)) throw new Error(`Component cycle includes ${id}.`);
      visited.add(id);
      pending.push(...("children" in component ? component.children : "child" in component ? [component.child] : []));
    }
    if (visited.size !== graph.size) throw new Error("Every component must be reachable from root.");
    return { template, error: null };
  } catch (error) {
    return { template: null, error: error instanceof Error ? error.message : "Enter valid template JSON." };
  }
}
