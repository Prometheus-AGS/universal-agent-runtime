import { AlertTriangle, CheckCircle2, Info, OctagonX } from "lucide-react";
import type { ConversionDiagnostic, ConversionDisposition, TeamsErrorInfo } from "@/platform/entities";
import { DISPOSITION_LABEL, sortBlockingFirst } from "../model/binding-diagnostics";

/** The runtime's own refusal, verbatim: its code and its detail. The operator's input is never touched. */
export function RuntimeNotice({ error, title = "The runtime did not accept this request" }: { error: TeamsErrorInfo | null; title?: string }) {
  if (!error) return null;
  const showDetail = error.detail && error.detail !== error.message;
  return (
    <div role="alert" className="space-y-1.5 rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
      <p className="flex items-center gap-2 font-medium">
        <OctagonX aria-hidden="true" className="size-4 shrink-0" />
        {title}
      </p>
      <p className="break-words">{error.message}</p>
      {showDetail && <p className="break-words">{error.detail}</p>}
      {error.capability && (
        <p className="break-words">
          Missing runtime capability: <span className="font-mono text-xs">{error.capability}</span>
        </p>
      )}
      <p className="font-mono text-xs break-all">
        {error.code}
        {error.status ? ` (HTTP ${error.status})` : ""}
      </p>
      {error.diagnostic?.action && (
        <p className="break-words">
          Runtime's suggested action: <span className="font-mono text-xs">{error.diagnostic.action}</span>
          {error.diagnostic.retryable === false ? " (retrying unchanged will not help)" : ""}
        </p>
      )}
      {error.uncertain && (
        <p>The outcome is unconfirmed: the request may have been committed. Check the list before sending it again.</p>
      )}
    </div>
  );
}

const DISPOSITION_ICON: Record<ConversionDisposition, typeof Info> = {
  "required-unsupported": OctagonX,
  "optional-unsupported": AlertTriangle,
  translated: Info,
  exact: CheckCircle2,
};

const DISPOSITION_TONE: Record<ConversionDisposition, string> = {
  "required-unsupported": "text-destructive",
  "optional-unsupported": "text-warning",
  translated: "text-muted-foreground",
  exact: "text-success",
};

interface DiagnosticGroup {
  key: string;
  diagnostic: ConversionDiagnostic;
  count: number;
}

/** The runtime reports a package diagnostic once per agent definition; identical entries are shown once with a count. */
function collapseIdentical(diagnostics: readonly ConversionDiagnostic[]): DiagnosticGroup[] {
  const groups = new Map<string, DiagnosticGroup>();
  for (const diagnostic of diagnostics) {
    const key = JSON.stringify([diagnostic.disposition, diagnostic.field, diagnostic.message]);
    const existing = groups.get(key);
    if (existing) existing.count += 1;
    else groups.set(key, { key, diagnostic, count: 1 });
  }
  return [...groups.values()];
}

/** Preflight diagnostics, blocking first. Each carries icon and text, never colour alone. */
export function DiagnosticList({ diagnostics, label }: { diagnostics: readonly ConversionDiagnostic[]; label: string }) {
  if (diagnostics.length === 0) return null;
  return (
    <ul aria-label={label} className="space-y-2">
      {collapseIdentical(sortBlockingFirst(diagnostics)).map(({ key, diagnostic, count }) => {
        const Icon = DISPOSITION_ICON[diagnostic.disposition];
        return (
          <li key={key} className="flex gap-2 rounded-lg bg-muted/60 p-3 text-sm">
            <Icon aria-hidden="true" className={`mt-0.5 size-4 shrink-0 ${DISPOSITION_TONE[diagnostic.disposition]}`} />
            <div className="min-w-0 space-y-0.5">
              <p className="font-medium">
                {DISPOSITION_LABEL[diagnostic.disposition]}
                {count > 1 && <span className="ml-2 text-xs font-normal text-muted-foreground">{count} definitions</span>}
              </p>
              <p className="break-words">{diagnostic.message}</p>
              <p className="font-mono text-xs break-all text-muted-foreground">{diagnostic.field || "(document)"}</p>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
