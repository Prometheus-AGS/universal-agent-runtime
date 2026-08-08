import { type FC, useState } from "react";
import { Loader2, Sparkles } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

// ── Types ───────────────────────────────────────────────────────────────────

interface AgentAiBuilderProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onGenerated: (agentData: Record<string, unknown>) => void;
  generate: (description: string) => Promise<unknown>;
}

type BuilderStatus = "idle" | "loading" | "success" | "error";

// ── Helpers ─────────────────────────────────────────────────────────────────

/**
 * Try to extract a structured agent object from the compiler response.
 * The compiler may return JSON-RPC with nested result structures or
 * plain text advice -- we handle both gracefully.
 */
function extractAgentData(body: unknown): { data: Record<string, unknown> | null; text: string | null } {
  if (!body || typeof body !== "object") {
    return { data: null, text: typeof body === "string" ? body : null };
  }

  const obj = body as Record<string, unknown>;

  // JSON-RPC success envelope
  const result = obj.result as Record<string, unknown> | undefined;
  if (result) {
    // The result might contain a message with parts
    const message = result.message as Record<string, unknown> | undefined;
    if (message) {
      const parts = message.parts as Array<Record<string, unknown>> | undefined;
      if (parts && parts.length > 0) {
        for (const part of parts) {
          // Look for a structured data part
          if (part.type === "data" && part.data && typeof part.data === "object") {
            return { data: part.data as Record<string, unknown>, text: null };
          }
          // Look for JSON embedded in text
          if (part.type === "text" && typeof part.text === "string") {
            const parsed = tryParseJson(part.text as string);
            if (parsed) return { data: parsed, text: part.text as string };
            return { data: null, text: part.text as string };
          }
        }
      }
    }

    // Result itself might be the agent spec
    if (result.kind === "agent" || result.id || result.metadata) {
      return { data: result, text: null };
    }

    // Try to extract from artifacts array
    const artifacts = result.artifacts as Array<Record<string, unknown>> | undefined;
    if (artifacts && artifacts.length > 0) {
      for (const artifact of artifacts) {
        const artifactParts = artifact.parts as Array<Record<string, unknown>> | undefined;
        if (artifactParts) {
          for (const part of artifactParts) {
            if (part.type === "data" && part.data && typeof part.data === "object") {
              return { data: part.data as Record<string, unknown>, text: null };
            }
            if (part.type === "text" && typeof part.text === "string") {
              const parsed = tryParseJson(part.text as string);
              if (parsed) return { data: parsed, text: part.text as string };
            }
          }
        }
      }
    }
  }

  // JSON-RPC error
  const error = obj.error as Record<string, unknown> | undefined;
  if (error) {
    const msg = (error.message as string) ?? "Compiler returned an error";
    return { data: null, text: msg };
  }

  return { data: null, text: null };
}

function tryParseJson(text: string): Record<string, unknown> | null {
  // Try the whole string first
  try {
    const parsed = JSON.parse(text);
    if (parsed && typeof parsed === "object") return parsed as Record<string, unknown>;
  } catch {
    // Try to find a JSON block in the text (e.g. ```json ... ```)
    const jsonBlockMatch = text.match(/```(?:json)?\s*\n?([\s\S]*?)```/);
    if (jsonBlockMatch?.[1]) {
      try {
        const parsed = JSON.parse(jsonBlockMatch[1].trim());
        if (parsed && typeof parsed === "object") return parsed as Record<string, unknown>;
      } catch {
        // not valid JSON
      }
    }
  }
  return null;
}

/**
 * Map compiler output fields into the shape that AgentEditor expects
 * (matching the UarAgent type / formStateFromAgent expectations).
 */
function normalizeAgentData(raw: Record<string, unknown>): Record<string, unknown> {
  const out: Record<string, unknown> = { ...raw };

  // Ensure metadata exists
  if (!out.metadata && (out.title || out.description || out.name)) {
    out.metadata = {
      title: (out.title as string) ?? (out.name as string) ?? "",
      description: (out.description as string) ?? "",
    };
  }

  // Ensure id
  if (!out.id) {
    const meta = out.metadata as Record<string, unknown> | undefined;
    const title = (meta?.title as string) ?? (out.name as string) ?? "generated-agent";
    out.id = title.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
  }

  return out;
}

// ── Component ───────────────────────────────────────────────────────────────

export const AgentAiBuilder: FC<AgentAiBuilderProps> = ({ open, onOpenChange, onGenerated, generate }) => {
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState<BuilderStatus>("idle");
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [infoMsg, setInfoMsg] = useState<string | null>(null);

  const reset = () => {
    setDescription("");
    setStatus("idle");
    setErrorMsg(null);
    setInfoMsg(null);
  };

  const handleOpenChange = (next: boolean) => {
    if (!next) reset();
    onOpenChange(next);
  };

  const handleGenerate = async () => {
    if (!description.trim()) return;

    setStatus("loading");
    setErrorMsg(null);
    setInfoMsg(null);

    try {
      const body = await generate(description.trim());
      const { data, text } = extractAgentData(body);

      if (data) {
        const normalized = normalizeAgentData(data);
        setStatus("success");
        // Brief success flash then hand off
        setTimeout(() => {
          onGenerated(normalized);
          handleOpenChange(false);
        }, 600);
      } else if (text) {
        // Compiler returned text advice but no structured agent
        setInfoMsg(text);
        setStatus("idle");
        // Still open editor with defaults populated from description
        const fallback = normalizeAgentData({
          metadata: { title: "", description: description.trim() },
        });
        setTimeout(() => {
          onGenerated(fallback);
          handleOpenChange(false);
        }, 2500);
      } else {
        throw new Error("Could not parse agent data from compiler response");
      }
    } catch (e) {
      setStatus("error");
      setErrorMsg((e as Error).message);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 font-display">
            <Sparkles size={16} className="text-primary" />
            Create Agent with AI
          </DialogTitle>
          <DialogDescription className="font-mono text-xs">
            Describe the agent you want to build and the AI compiler will generate a definition for you.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <Textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="An agent that searches the web and summarizes research findings into structured reports..."
            rows={4}
            className="resize-none font-body text-sm"
            disabled={status === "loading"}
          />

          {status === "error" && errorMsg && (
            <p className="rounded-md bg-destructive/10 px-3 py-2 font-mono text-xs text-destructive">
              {errorMsg}
            </p>
          )}

          {status === "idle" && infoMsg && (
            <div className="rounded-md bg-muted px-3 py-2">
              <p className="mb-1 font-mono text-xs font-medium text-muted-foreground">
                Compiler response (opening editor with defaults):
              </p>
              <p className="max-h-32 overflow-y-auto font-mono text-xs text-foreground">
                {infoMsg}
              </p>
            </div>
          )}

          {status === "success" && (
            <p className="font-mono text-xs text-green-400">
              Agent definition generated. Opening editor...
            </p>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={() => handleOpenChange(false)}
            disabled={status === "loading"}
          >
            Cancel
          </Button>
          <Button
            onClick={() => void handleGenerate()}
            disabled={status === "loading" || !description.trim()}
            className="gap-1.5"
          >
            {status === "loading" ? (
              <>
                <Loader2 size={14} className="animate-spin" />
                Generating agent definition...
              </>
            ) : (
              <>
                <Sparkles size={13} />
                Generate
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
