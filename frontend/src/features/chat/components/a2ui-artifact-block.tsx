import { CheckCircle2Icon, Loader2Icon, PanelTopOpenIcon } from "lucide-react";
import { type FC, useMemo, useState } from "react";
import { A2uiMessageSchema, MessageProcessor } from "@prometheus-ags/a2ui-core/v0_9";
import { UarSurface, uarBasicCatalog } from "@prometheus-ags/a2ui-uar";

import {
  A2UI_PROFILE,
  A2UI_VERSION,
  type A2uiComponent,
} from "@/features/a2ui/a2ui-protocol";
import {
  MAX_A2UI_COMPONENTS,
  MAX_A2UI_MESSAGES,
  MAX_A2UI_SOURCE_BYTES,
  MAX_A2UI_SURFACES,
} from "@/features/a2ui/a2ui-rendering-limits";
import { A2uiSurfaceRenderer } from "@/features/a2ui/a2ui-surface-renderer";
import { JsonSource } from "@/features/chat/ui/chunks/chunk-surface";
import { useTheme } from "@/hooks/use-theme";
import { useToolApprovalActions } from "@/hooks/use-tool-approval";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseJsonObject(value: string): Record<string, unknown> | null {
  try {
    const parsed = JSON.parse(value);
    return isRecord(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function readString(value: Record<string, unknown>, key: string, fallback: string): string {
  return typeof value[key] === "string" ? value[key] : fallback;
}

function setDataPath(data: Record<string, unknown>, path: string, value: unknown): Record<string, unknown> {
  if (!path.startsWith("/")) return data;
  const segments = path.slice(1).split("/");
  const next = structuredClone(data);
  let parent = next;
  for (const segment of segments.slice(0, -1)) {
    const existing = parent[segment];
    parent[segment] = isRecord(existing) ? existing : {};
    parent = parent[segment] as Record<string, unknown>;
  }
  parent[segments.at(-1)!] = value;
  return next;
}

function legacyInputSurface(
  artifactType: string,
  title: string,
  input: Record<string, unknown>,
): { components: A2uiComponent[]; data: Record<string, unknown> } | null {
  const heading: A2uiComponent = { id: "heading", component: "Text", text: title || "User input required", variant: "h2" };
  const components: A2uiComponent[] = [heading];
  const children = ["heading"];
  const button = (id: string, label: string, actionName: string, variant: "primary" | "secondary" = "primary") => {
    components.push({ id: `${id}-label`, component: "Text", text: label });
    components.push({ id, component: "Button", child: `${id}-label`, variant, action: { event: { name: actionName } } });
    return id;
  };

  let data: Record<string, unknown> = {};
  if (artifactType === "confirm") {
    components.push({ id: "message", component: "Text", text: readString(input, "message", readString(input, "prompt", "Please confirm")) });
    components.push({
      id: "actions",
      component: "Row",
      children: [
        button("accept", readString(input, "accept_label", "Accept"), "accept"),
        button("cancel", readString(input, "cancel_label", "Cancel"), "cancel", "secondary"),
      ],
    });
    children.push("message", "actions");
  } else if (artifactType === "select") {
    const options = Array.isArray(input.options)
      ? input.options.flatMap((option) => isRecord(option) && typeof option.value === "string"
        ? [{ value: option.value, label: readString(option, "label", option.value) }]
        : [])
      : [];
    if (options.length === 0) return null;
    data = { selection: [] };
    components.push({
      id: "selection",
      component: "ChoicePicker",
      label: readString(input, "prompt", "Choose an option"),
      value: { path: "/selection" },
      variant: "mutuallyExclusive",
      options,
    });
    children.push("selection", button("submit", "Submit", "submit"));
  } else if (artifactType === "text_input") {
    data = { text: readString(input, "text", "") };
    components.push({
      id: "text",
      component: "TextField",
      label: readString(input, "prompt", "Provide input"),
      value: { path: "/text" },
      variant: input.multiline === true ? "longText" : "shortText",
      placeholder: readString(input, "placeholder", ""),
    });
    children.push("text", button("submit", "Submit", "submit"));
  } else if (artifactType === "form") {
    data = { json: "{}" };
    components.push({
      id: "json",
      component: "TextField",
      label: "Structured response JSON",
      value: { path: "/json" },
      variant: "longText",
    });
    children.push("json", button("submit", "Submit", "submit"));
  } else {
    return null;
  }
  components.push({ id: "root", component: "Column", children });
  return { components, data };
}

interface A2uiInputBlockProps {
  runId: string;
  artifactId: string;
  artifactType: string;
  title: string;
  content: string;
  metadata: Record<string, unknown>;
  status: "running" | "complete" | "failed";
  result?: string;
}

export const A2uiInputBlock: FC<A2uiInputBlockProps> = ({
  runId, artifactId, artifactType, title, content, metadata, status, result,
}) => {
  const { submitArtifactResponse } = useToolApprovalActions();
  const input = useMemo(() => parseJsonObject(content) ?? metadata, [content, metadata]);
  const surface = useMemo(() => legacyInputSurface(artifactType, title, input), [artifactType, input, title]);
  const [data, setData] = useState<Record<string, unknown>>(() => surface?.data ?? {});
  const [submitting, setSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const submit = async (name: string) => {
    let response: Record<string, unknown>;
    if (name === "accept" || name === "cancel") response = { accepted: name === "accept" };
    else if (artifactType === "select") response = { value: Array.isArray(data.selection) ? data.selection[0] : undefined };
    else if (artifactType === "text_input") response = { text: String(data.text ?? "") };
    else {
      const parsed = parseJsonObject(String(data.json ?? ""));
      if (!parsed) {
        setSubmitError("Response must be a valid JSON object.");
        return;
      }
      response = parsed;
    }
    setSubmitting(true);
    setSubmitError(null);
    try {
      const result = await submitArtifactResponse(runId, { artifact_id: artifactId, response });
      if (!result.ok) throw new Error(await result.text().catch(() => `HTTP ${result.status}`));
      setSubmitted(true);
    } catch (error) {
      setSubmitError(error instanceof Error ? error.message : "Failed to submit response");
    } finally {
      setSubmitting(false);
    }
  };

  const resolved = status === "complete" || submitted;
  return (
    <section className="my-2 rounded-xl bg-card px-3 py-3" aria-label="A2UI input">
      <div className="mb-3 flex items-center gap-2">
        <PanelTopOpenIcon size={14} className="text-primary" aria-hidden="true" />
        <span className="eyebrow">A2UI input</span>
        <span className="ml-auto font-mono text-[10px] text-fg-faint">{artifactType}</span>
      </div>
      {surface ? (
        <A2uiSurfaceRenderer
          components={surface.components}
          data={data}
          onDataChange={(path, value) => setData((current) => setDataPath(current, path, value))}
          onAction={(name) => void submit(name)}
          actionPending={submitting || resolved}
          statusMessage={submitting ? "Sending response…" : resolved ? "Response captured" : submitError}
        />
      ) : (
        <p className="text-sm text-destructive" role="alert">Unsupported or invalid A2UI artifact type: {artifactType}</p>
      )}
      {submitting ? <Loader2Icon size={14} className="mt-2 animate-spin text-muted-foreground" aria-hidden="true" /> : null}
      {resolved ? <CheckCircle2Icon size={14} className="mt-2 text-success" aria-hidden="true" /> : null}
      {result ? <pre className="mt-2 overflow-x-auto rounded-md bg-surface p-2 text-xs text-fg-sub">{result}</pre> : null}
    </section>
  );
};

interface A2uiDisplayBlockProps {
  artifactType: string;
  title: string;
  content: string;
  language?: string;
  profile?: string;
  validation?: "valid" | "invalid" | "unknown-component";
  validationError?: string;
}

function parseMessageSource(content: string): unknown[] {
  if (new TextEncoder().encode(content).byteLength > MAX_A2UI_SOURCE_BYTES) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_SOURCE_BYTES / 1024} KiB rendering limit.`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(content);
  } catch {
    const lines = content.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
    if (lines.length === 0) throw new Error("The artifact contains no A2UI messages.");
    parsed = lines.map((line) => JSON.parse(line) as unknown);
  }

  const messages = Array.isArray(parsed)
    ? parsed
    : isRecord(parsed) && Array.isArray(parsed.messages)
      ? parsed.messages
      : [parsed];
  if (messages.length > MAX_A2UI_MESSAGES) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_MESSAGES}-message rendering limit.`);
  }
  return messages;
}

function boundedDiagnosticSource(content: string): string {
  const encoder = new TextEncoder();
  const sourceBytes = encoder.encode(content);
  if (sourceBytes.byteLength <= MAX_A2UI_SOURCE_BYTES) return content;

  const notice = `\n\n[A2UI source truncated at ${MAX_A2UI_SOURCE_BYTES / 1024} KiB.]`;
  const noticeBytes = encoder.encode(notice);
  const excerptBytes = sourceBytes.slice(0, MAX_A2UI_SOURCE_BYTES - noticeBytes.byteLength - 4);
  return `${new TextDecoder().decode(excerptBytes)}${notice}`;
}

function validateMessage(raw: unknown) {
  if (!isRecord(raw)) throw new Error("Each A2UI message must be a JSON object.");
  const { profile, ...message } = raw;
  if (profile !== undefined && profile !== A2UI_PROFILE) {
    throw new Error(`Unsupported A2UI profile: ${String(profile)}`);
  }
  if (message.version !== A2UI_VERSION) {
    throw new Error(`Unsupported A2UI version: ${String(message.version ?? "missing")}. Expected ${A2UI_VERSION}.`);
  }
  const parsed = A2uiMessageSchema.parse(message);
  if ("updateComponents" in parsed) {
    for (const component of parsed.updateComponents.components) {
      if (component.component && !uarBasicCatalog.components.has(component.component)) {
        throw new Error(`Unapproved A2UI component: ${component.component}`);
      }
    }
  }
  return parsed;
}

function processA2uiDisplayContent(content: string) {
  const processor = new MessageProcessor(
    [uarBasicCatalog],
    undefined,
    { version: A2UI_VERSION },
  );
  const messages = parseMessageSource(content).map(validateMessage);
  if (messages.length === 0) throw new Error("The artifact contains no A2UI messages.");
  const componentCount = messages.reduce((count, message) => (
    "updateComponents" in message
      ? count + message.updateComponents.components.length
      : count
  ), 0);
  if (componentCount > MAX_A2UI_COMPONENTS) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_COMPONENTS}-component rendering limit.`);
  }
  processor.processMessages(messages);
  const surfaces = [...processor.model.surfacesMap.values()];
  if (surfaces.length > MAX_A2UI_SURFACES) {
    throw new Error(`A2UI source exceeds the ${MAX_A2UI_SURFACES}-surface rendering limit.`);
  }
  const deleted = surfaces.length === 0 && messages.some((message) => "deleteSurface" in message);
  if (surfaces.length === 0 && !deleted) throw new Error("The artifact did not create an A2UI surface.");
  return { surfaces, deleted };
}

export const A2uiDisplayBlock: FC<A2uiDisplayBlockProps> = ({
  artifactType,
  title,
  content,
  language,
  profile,
  validation = "valid",
  validationError,
}) => {
  const { resolved: theme } = useTheme();
  const diagnosticSource = useMemo(() => boundedDiagnosticSource(content), [content]);
  const rendered = useMemo(() => {
    if (validation !== "valid") {
      return { surfaces: [], error: validationError ?? "The A2UI artifact failed validation." };
    }
    if (profile !== A2UI_PROFILE) {
      return {
        surfaces: [],
        error: profile
          ? `Unsupported A2UI profile: ${profile}`
          : "The artifact is missing its required UAR A2UI profile.",
      };
    }
    try {
      return { ...processA2uiDisplayContent(content), error: null };
    } catch (error) {
      return {
        surfaces: [],
        deleted: false,
        error: error instanceof Error ? error.message : "The A2UI artifact could not be rendered.",
      };
    }
  }, [content, profile, validation, validationError]);

  const renderedSuccessfully = rendered.error === null;
  return (
    <section className="my-2 rounded-xl bg-card px-3 py-3" aria-label="A2UI display artifact">
      <div className="mb-3 flex min-w-0 items-center gap-2">
        <PanelTopOpenIcon size={14} className="text-primary" aria-hidden="true" />
        <span className="eyebrow">Artifact</span>
        <span className="ml-auto truncate font-mono text-[10px] text-fg-faint">
          {renderedSuccessfully ? `A2UI ${A2UI_VERSION} · ${rendered.deleted ? "removed" : "rendered"}` : "Invalid surface"}
          {artifactType ? ` · ${artifactType}` : ""}
          {language && language !== "a2ui" ? ` · ${language}` : ""}
        </span>
      </div>
      {renderedSuccessfully && rendered.deleted ? (
        <p className="rounded-lg bg-surface px-3 py-3 text-sm text-fg-sub" role="status">Surface removed.</p>
      ) : renderedSuccessfully ? (
        <div className="space-y-3">
          {rendered.surfaces.map((surface) => (
            <UarSurface
              key={surface.id}
              surface={surface}
              theme={theme}
              resetKey={`${surface.id}:${content.length}`}
            />
          ))}
        </div>
      ) : (
        <div className="space-y-3">
          <div role="alert" className="rounded-lg bg-destructive/10 px-3 py-2">
            <p className="text-sm font-medium text-destructive">{title || "Artifact"} could not be rendered.</p>
            <p className="mt-1 break-words text-xs text-fg-sub">{rendered.error}</p>
          </div>
          <details className="rounded-lg bg-surface px-3 py-2 text-sm [&_pre]:whitespace-pre-wrap [&_pre]:break-all">
            <summary className="min-h-11 cursor-pointer py-3 font-medium text-fg-sub">View A2UI source</summary>
            <JsonSource value={diagnosticSource} label={`${title || "Artifact"} A2UI source`} />
          </details>
        </div>
      )}
    </section>
  );
};
