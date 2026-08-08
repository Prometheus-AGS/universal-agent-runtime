import { type FC, useState } from "react";
import { FolderOpen, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface ImportSkillResult {
  parsed: {
    name: string;
    title: string;
    description: string;
    version: string;
    triggers: { keywords: string[] };
    prompt_overlay: string;
    source: string;
    source_path: string;
    references: string[];
    detected_format: string;
  };
  validation: { valid: boolean; warnings: string[] };
}

interface SkillImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImported: () => void;
  parseImport: (path: string) => Promise<ImportSkillResult>;
  importParsed: (result: ImportSkillResult) => Promise<void>;
}

export const SkillImportDialog: FC<SkillImportDialogProps> = ({
  open,
  onOpenChange,
  onImported,
  parseImport,
  importParsed,
}) => {
  const [path, setPath] = useState("");
  const [parsing, setParsing] = useState(false);
  const [importing, setImporting] = useState(false);
  const [result, setResult] = useState<ImportSkillResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const reset = () => {
    setPath("");
    setResult(null);
    setError(null);
    setParsing(false);
    setImporting(false);
  };

  const handleOpenChange = (nextOpen: boolean) => {
    onOpenChange(nextOpen);
    if (!nextOpen) reset();
  };

  const handleParse = async () => {
    const trimmed = path.trim();
    if (!trimmed) return;

    setParsing(true);
    setError(null);
    setResult(null);

    try {
      const data = await parseImport(trimmed);
      setResult(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to parse skill directory");
    } finally {
      setParsing(false);
    }
  };

  const handleImport = async () => {
    if (!result) return;

    setImporting(true);
    setError(null);

    try {
      await importParsed(result);
      handleOpenChange(false);
      onImported();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to import skill");
    } finally {
      setImporting(false);
    }
  };

  const parsed = result?.parsed;
  const validation = result?.validation;

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FolderOpen size={16} />
            Import Skill from Disk
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* Path input */}
          <div className="space-y-1.5">
            <Label htmlFor="import-path" className="font-mono text-xs text-muted-foreground">
              Directory path
            </Label>
            <div className="flex gap-2">
              <Input
                id="import-path"
                value={path}
                onChange={(e) => setPath(e.target.value)}
                placeholder="/absolute/path/to/skill"
                className="flex-1 font-mono text-xs"
                onKeyDown={(e) => {
                  if (e.key === "Enter") void handleParse();
                }}
              />
              <Button
                size="sm"
                variant="outline"
                onClick={() => void handleParse()}
                disabled={parsing || !path.trim()}
              >
                {parsing ? <Loader2 size={13} className="mr-1 animate-spin" /> : null}
                Parse
              </Button>
            </div>
          </div>

          {/* Error message */}
          {error && (
            <p className="text-xs text-destructive">{error}</p>
          )}

          {/* Preview card */}
          {parsed && (
            <div className="rounded-lg border border-border bg-card p-4 space-y-3">
              {/* Header row: name + format badge */}
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <p className="font-display text-sm font-semibold text-foreground">
                    {parsed.title || parsed.name}
                  </p>
                  {parsed.title && parsed.name !== parsed.title && (
                    <p className="font-mono text-[10px] text-muted-foreground">{parsed.name}</p>
                  )}
                </div>
                <span className="shrink-0 rounded-md bg-primary/15 px-2 py-0.5 text-xs font-medium text-primary">
                  {parsed.detected_format}
                </span>
              </div>

              {/* Description */}
              {parsed.description && (
                <p className="text-xs text-muted-foreground">{parsed.description}</p>
              )}

              {/* Version + source path */}
              <div className="flex flex-wrap gap-x-4 gap-y-1 text-[10px] text-muted-foreground font-mono">
                <span>v{parsed.version}</span>
                <span className="truncate max-w-[280px]" title={parsed.source_path}>
                  {parsed.source_path}
                </span>
              </div>

              {/* Keywords */}
              {parsed.triggers.keywords.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  {parsed.triggers.keywords.map((kw) => (
                    <span
                      key={kw}
                      className="rounded-full bg-muted px-2 py-0.5 text-[10px] font-mono text-muted-foreground"
                    >
                      {kw}
                    </span>
                  ))}
                </div>
              )}

              {/* Validation warnings */}
              {validation && validation.warnings.length > 0 && (
                <div className="space-y-1">
                  {validation.warnings.map((w, i) => (
                    <p key={i} className="text-xs text-amber-400">
                      Warning: {w}
                    </p>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={() => handleOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => void handleImport()}
            disabled={!result || importing || (validation && !validation.valid)}
          >
            {importing && <Loader2 size={14} className="mr-1 animate-spin" />}
            Import
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
