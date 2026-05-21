import { CheckCircle2Icon, Loader2Icon, PanelTopOpenIcon, SendIcon } from "lucide-react";
import { type FC, useId, useMemo, useState } from "react";
import { useToolApprovalActions } from "@/hooks/use-tool-approval";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

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

function readString(obj: Record<string, unknown>, key: string): string | null {
  const value = obj[key];
  return typeof value === "string" ? value : null;
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
  runId,
  artifactId,
  artifactType,
  title,
  content,
  metadata,
  status,
  result,
}) => {
  const { submitArtifactResponse } = useToolApprovalActions();
  const contentObj = useMemo(() => parseJsonObject(content), [content]);
  const inputObj = contentObj ?? metadata;

  const [textValue, setTextValue] = useState(readString(inputObj, "text") ?? "");
  const [selectValue, setSelectValue] = useState("");
  const [formJson, setFormJson] = useState("{}");
  const [submitting, setSubmitting] = useState(false);
  const [submitted, setSubmitted] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const baseId = useId();
  const selectFieldId = `${baseId}-select`;
  const textFieldId = `${baseId}-text`;

  const options = useMemo(() => {
    const raw = inputObj.options;
    if (!Array.isArray(raw)) return [] as Array<{ value: string; label: string }>;
    return raw
      .map((item) => (isRecord(item) ? item : null))
      .filter((item): item is Record<string, unknown> => item !== null)
      .map((item) => {
        const value = typeof item.value === "string" ? item.value : "";
        const label = typeof item.label === "string" ? item.label : value;
        return { value, label };
      })
      .filter((item) => item.value.length > 0);
  }, [inputObj]);

  const confirmMessage = readString(inputObj, "message") ?? readString(inputObj, "prompt") ?? "Please confirm";
  const acceptLabel = readString(inputObj, "accept_label") ?? "Accept";
  const cancelLabel = readString(inputObj, "cancel_label") ?? "Cancel";
  const prompt = readString(inputObj, "prompt") ?? "Provide input";
  const placeholder = readString(inputObj, "placeholder") ?? "";
  const multiline = inputObj.multiline === true;

  const submitResponse = async (response: Record<string, unknown>) => {
    setSubmitting(true);
    setSubmitError(null);
    try {
      const res = await submitArtifactResponse(runId, {
        artifact_id: artifactId,
        response,
      });
      if (!res.ok) {
        const body = await res.text().catch(() => "Failed to submit artifact response");
        throw new Error(body || `HTTP ${res.status}`);
      }
      setSubmitted(true);
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : "Failed to submit response");
    } finally {
      setSubmitting(false);
    }
  };

  const resolved = status === "complete";

  return (
    <div className="my-2 rounded-lg border border-primary/25 bg-primary/5 px-3 py-2.5">
      <div className="mb-2 flex items-center gap-2">
        <PanelTopOpenIcon size={12} className="text-primary" />
        <span className="font-mono text-[10px] uppercase tracking-widest text-primary/90">
          A2UI Input
        </span>
        <span className="ml-auto font-mono text-[9px] text-primary/70">{artifactType}</span>
      </div>

      <p className="font-display text-sm font-semibold text-foreground">{title || "User input required"}</p>

      {artifactType === "confirm" && (
        <div className="mt-2 space-y-2">
          <p className="font-body text-sm text-muted-foreground">{confirmMessage}</p>
          <div className="flex gap-2">
            <Button
              type="button"
              size="sm"
              disabled={submitting || resolved}
              onClick={() => void submitResponse({ accepted: true })}
            >
              {acceptLabel}
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={submitting || resolved}
              onClick={() => void submitResponse({ accepted: false })}
            >
              {cancelLabel}
            </Button>
          </div>
        </div>
      )}

      {artifactType === "select" && (
        <div className="mt-2 space-y-2">
          <Label htmlFor={selectFieldId} className="font-body text-sm font-normal text-muted-foreground">
            {prompt}
          </Label>
          {options.length > 0 ? (
            <Select
              value={selectValue === "" ? undefined : selectValue}
              onValueChange={(v) => v != null && setSelectValue(v)}
              disabled={submitting || resolved}
            >
              <SelectTrigger id={selectFieldId} className="h-9 w-full">
                <SelectValue placeholder="Choose an option" />
              </SelectTrigger>
              <SelectContent>
                {options.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            <p className="font-mono text-xs text-muted-foreground">No options defined for this select.</p>
          )}
          <Button
            type="button"
            size="sm"
            disabled={!selectValue || submitting || resolved}
            onClick={() => void submitResponse({ value: selectValue })}
          >
            <SendIcon size={12} className="mr-1" />
            Submit
          </Button>
        </div>
      )}

      {artifactType === "text_input" && (
        <div className="mt-2 space-y-2">
          <Label htmlFor={textFieldId} className="font-body text-sm font-normal text-muted-foreground">
            {prompt}
          </Label>
          {multiline ? (
            <Textarea
              id={textFieldId}
              value={textValue}
              onChange={(e) => setTextValue(e.target.value)}
              placeholder={placeholder}
              disabled={submitting || resolved}
              rows={4}
            />
          ) : (
            <Input
              id={textFieldId}
              value={textValue}
              onChange={(e) => setTextValue(e.target.value)}
              placeholder={placeholder}
              disabled={submitting || resolved}
            />
          )}
          <Button
            type="button"
            size="sm"
            disabled={!textValue.trim() || submitting || resolved}
            onClick={() => void submitResponse({ text: textValue })}
          >
            <SendIcon size={12} className="mr-1" />
            Submit
          </Button>
        </div>
      )}

      {artifactType === "form" && (
        <div className="mt-2 space-y-2">
          <p className="font-body text-sm text-muted-foreground">
            Structured form received. Submit JSON response:
          </p>
          <Textarea
            value={formJson}
            onChange={(e) => setFormJson(e.target.value)}
            disabled={submitting || resolved}
            rows={6}
          />
          <Button
            type="button"
            size="sm"
            disabled={submitting || resolved}
            onClick={() => {
              const parsed = parseJsonObject(formJson);
              if (!parsed) {
                setSubmitError("Form JSON must be a valid object");
                return;
              }
              void submitResponse(parsed);
            }}
          >
            <SendIcon size={12} className="mr-1" />
            Submit
          </Button>
        </div>
      )}

      {artifactType !== "confirm" &&
        artifactType !== "select" &&
        artifactType !== "text_input" &&
        artifactType !== "form" && (
          <div className="mt-2 space-y-2">
            <p className="font-body text-sm text-muted-foreground">
              Unsupported artifact type `{artifactType}`. Submit raw JSON:
            </p>
            <Textarea
              value={formJson}
              onChange={(e) => setFormJson(e.target.value)}
              disabled={submitting || resolved}
              rows={6}
            />
            <Button
              type="button"
              size="sm"
              disabled={submitting || resolved}
              onClick={() => {
                const parsed = parseJsonObject(formJson);
                if (!parsed) {
                  setSubmitError("JSON must be a valid object");
                  return;
                }
                void submitResponse(parsed);
              }}
            >
              <SendIcon size={12} className="mr-1" />
              Submit
            </Button>
          </div>
        )}

      <div className="mt-2 flex items-center gap-2">
        {submitting && (
          <span className="inline-flex items-center gap-1 font-mono text-[10px] text-muted-foreground/70">
            <Loader2Icon size={10} className="animate-spin" />
            Sending response...
          </span>
        )}
        {!submitting && (submitted || resolved) && (
          <span className="inline-flex items-center gap-1 font-mono text-[10px] text-success">
            <CheckCircle2Icon size={10} />
            Response captured
          </span>
        )}
        {submitError && <span className="font-mono text-[10px] text-destructive">{submitError}</span>}
      </div>

      {result && (
        <pre className="mt-2 overflow-x-auto rounded-md border border-border/40 bg-background/70 p-2 font-mono text-[10px] text-muted-foreground">
          {result}
        </pre>
      )}
    </div>
  );
};

interface A2uiDisplayBlockProps {
  artifactType: string;
  title: string;
  content: string;
  language?: string;
}

export const A2uiDisplayBlock: FC<A2uiDisplayBlockProps> = ({ artifactType, title, content, language }) => {
  return (
    <div className="my-2 rounded-lg border border-border/50 bg-muted/20 px-3 py-2.5">
      <div className="mb-2 flex items-center gap-2">
        <PanelTopOpenIcon size={12} className="text-muted-foreground" />
        <span className="font-mono text-[10px] uppercase tracking-widest text-muted-foreground">
          Artifact
        </span>
        <span className="ml-auto font-mono text-[9px] text-muted-foreground/70">
          {artifactType}
          {language ? ` · ${language}` : ""}
        </span>
      </div>
      <p className="font-display text-sm font-semibold text-foreground">{title || "Artifact"}</p>
      <pre className="mt-2 whitespace-pre-wrap rounded-md border border-border/40 bg-background/70 p-2 font-body text-xs text-muted-foreground">
        {content}
      </pre>
    </div>
  );
};
