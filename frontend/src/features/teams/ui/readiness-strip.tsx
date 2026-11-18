import { CheckCircle2, HelpCircle, Loader2, OctagonX, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { FactState, ReadinessFact } from "../model/readiness";
import { useReadiness } from "../model/use-readiness";

const STATE_TEXT: Record<FactState, string> = {
  ok: "Ready",
  blocked: "Blocked",
  unknown: "Unknown",
  checking: "Checking",
};

const STATE_TONE: Record<FactState, string> = {
  ok: "text-success",
  blocked: "text-destructive",
  unknown: "text-warning",
  checking: "text-muted-foreground",
};

function StateIcon({ state }: { state: FactState }) {
  const className = cn("size-4 shrink-0", STATE_TONE[state]);
  if (state === "ok") return <CheckCircle2 aria-hidden="true" className={className} />;
  if (state === "blocked") return <OctagonX aria-hidden="true" className={className} />;
  if (state === "unknown") return <HelpCircle aria-hidden="true" className={className} />;
  return <Loader2 aria-hidden="true" className={cn(className, "motion-safe:animate-spin")} />;
}

function Fact({ fact }: { fact: ReadinessFact }) {
  const settled = fact.state === "ok";
  return (
    <li className={cn("min-w-0 space-y-1 rounded-lg p-3", settled ? "bg-muted/30" : "bg-muted")}>
      <p className="flex flex-wrap items-center gap-x-2 gap-y-0.5 text-sm font-medium">
        <StateIcon state={fact.state} />
        <span>{fact.label}</span>
        <span className={cn("text-xs font-normal", STATE_TONE[fact.state])}>{STATE_TEXT[fact.state]}</span>
      </p>
      <p className="break-words text-xs text-muted-foreground">{fact.summary}</p>
      {!settled && <p className="text-xs text-muted-foreground">Gates: {fact.blocks}</p>}
    </li>
  );
}

function FactGrid({ facts }: { facts: readonly ReadinessFact[] }) {
  return (
    <ul className="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
      {facts.map((fact) => (
        <Fact key={fact.id} fact={fact} />
      ))}
    </ul>
  );
}

/**
 * Four runtime facts with icon and text each, and what each one gates. Execution ownership is read-only here.
 * While every fact is ready the strip is a single short summary inside the page header row that opens on
 * request; as soon as one is not ready it takes the full row and shows every reason without a click.
 */
export function ReadinessStrip({ workspace }: { workspace: string }) {
  const facts = useReadiness(workspace);
  const ready = facts.filter((fact) => fact.state === "ok").length;
  const allReady = facts.length > 0 && ready === facts.length;
  const needsIdentity = facts[0]?.state === "blocked";
  return (
    <section
      aria-labelledby="teams-readiness-heading"
      className={cn("min-w-0", allReady ? "md:ml-auto has-[details[open]]:ml-0 has-[details[open]]:w-full" : "w-full space-y-2")}
    >
      <div className={allReady ? "sr-only" : "flex flex-wrap items-baseline gap-x-3"}>
        <h2 id="teams-readiness-heading" className="text-sm font-medium">Runtime readiness</h2>
        <p role="status" className="text-xs text-muted-foreground">{ready} of {facts.length} ready</p>
      </div>
      {allReady ? (
        <details className="rounded-lg bg-muted/30">
          <summary className="flex min-h-11 cursor-pointer flex-wrap items-center gap-x-2 gap-y-1 rounded-lg px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-ring md:min-h-9 md:py-1">
            <StateIcon state="ok" />
            <span>Runtime ready</span>
            <span className="text-xs text-muted-foreground">{ready} of {facts.length}. Show details</span>
          </summary>
          <div className="p-2 pt-0">
            <FactGrid facts={facts} />
          </div>
        </details>
      ) : (
        <FactGrid facts={facts} />
      )}
      {needsIdentity && (
        <Button type="button" variant="secondary" className="min-h-11" onClick={() => window.location.reload()}>
          <RefreshCw aria-hidden="true" />
          Reload after signing in
        </Button>
      )}
    </section>
  );
}
