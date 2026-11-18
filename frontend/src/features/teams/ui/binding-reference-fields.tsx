import { useId } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { ConversionDiagnostic } from "@/platform/entities";
import type { NumberFields } from "../model/binding-draft";
import { DiagnosticList } from "./runtime-notice";

interface ReferenceFieldProps {
  label: string;
  explanation: string;
  example: string;
  value: string;
  diagnostics: readonly ConversionDiagnostic[];
  onChange: (value: string) => void;
}

/** A reference the operator owns. The example is offered, never applied until the operator presses its button. */
export function ReferenceField({ label, explanation, example, value, diagnostics, onChange }: ReferenceFieldProps) {
  const id = useId();
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <p id={`${id}-hint`} className="text-xs text-muted-foreground">{explanation}</p>
      <div className="flex flex-wrap items-center gap-2">
        <Input id={id} className="min-h-11 min-w-0 flex-1 font-mono text-sm" value={value} spellCheck={false}
          aria-describedby={`${id}-hint`} onChange={(event) => onChange(event.target.value)} />
        <Button type="button" variant="secondary" className="min-h-11" aria-label={`Use example value for ${label}: ${example}`} onClick={() => onChange(example)}>
          Use example value
        </Button>
      </div>
      <p className="font-mono text-xs break-all text-muted-foreground">Example: {example}</p>
      <DiagnosticList label={`Check results for ${label}`} diagnostics={diagnostics} />
    </div>
  );
}

interface NumberEntryProps {
  label: string;
  hint?: string;
  value: string;
  problem?: string;
  inputMode: "numeric" | "decimal";
  onChange: (value: string) => void;
}

export function NumberEntry({ label, hint, value, problem, inputMode, onChange }: NumberEntryProps) {
  const id = useId();
  const describedBy = [hint ? `${id}-hint` : null, problem ? `${id}-problem` : null].filter(Boolean).join(" ") || undefined;
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <Input id={id} className="min-h-11" inputMode={inputMode} value={value} aria-invalid={problem ? true : undefined}
        aria-describedby={describedBy} onChange={(event) => onChange(event.target.value)} />
      {hint && <p id={`${id}-hint`} className="text-xs text-muted-foreground">{hint}</p>}
      {problem && <p id={`${id}-problem`} className="text-xs text-destructive">{problem}</p>}
    </div>
  );
}

interface LimitsFieldsProps {
  numbers: NumberFields;
  problems: Partial<Record<keyof NumberFields, string>>;
  diagnostics: readonly ConversionDiagnostic[];
  onChange: (key: keyof NumberFields, value: string) => void;
}

export function LimitsFields({ numbers, problems, diagnostics, onChange }: LimitsFieldsProps) {
  const entry = (key: keyof NumberFields, label: string, inputMode: "numeric" | "decimal" = "numeric", hint?: string) => (
    <NumberEntry key={key} label={label} hint={hint} value={numbers[key]} problem={problems[key]} inputMode={inputMode} onChange={(value) => onChange(key, value)} />
  );
  return (
    <div className="space-y-4">
      <p className="text-xs text-muted-foreground">Prefilled from the team definition. Leave an entry empty to set no value.</p>
      <div className="grid gap-4 sm:grid-cols-2">
        {entry("concurrentTurns", "Concurrent turns")}
        {entry("maxMembers", "Maximum members")}
        {entry("maxDepth", "Maximum delegation depth")}
        {entry("maxPendingTasks", "Maximum tasks", "numeric", "Every task ever created counts against this.")}
        {entry("maxTokens", "Token budget")}
        {entry("maxCostUsd", "Cost budget (USD)", "decimal", "Budgets are in USD; the runtime admits no other currency.")}
        {entry("maxElapsedSeconds", "Elapsed-time budget (seconds)")}
      </div>
      <DiagnosticList label="Check results for limits and budget" diagnostics={diagnostics} />
    </div>
  );
}
