import type { ReactNode } from "react";
import { ArrowLeft, Link2, Rocket } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  teamLoadKeys,
  useBindingField,
  useBindingIdsForPackage,
  useExportedCoordinatorRole,
  useExportedModelRequirements,
  usePackageExportRefusal,
  usePackageExportStatus,
  usePackageField,
  useTeamActions,
  useTeamDefinitionField,
  useTeamLoadError,
  useTeamLoadStatus,
} from "@/platform/entities";
import { formatCount, formatDateTime, formatMicrocost, formatSeconds, shortDigest } from "../model/format";
import { useBindGate, useDefinitionExportLoader, useDefinitionRefs } from "../model/use-definition";
import { useActionGate } from "../model/use-readiness";
import { GatedAction } from "./gated-action";

const NO_DEFINITION = { id: "", version: "" } as const;

function Section({ id, title, children }: { id: string; title: string; children: ReactNode }) {
  return (
    <section aria-labelledby={id} className="space-y-3">
      <h3 id={id} className="text-sm font-semibold">{title}</h3>
      {children}
    </section>
  );
}

function Facts({ items }: { items: ReadonlyArray<readonly [string, string | undefined]> }) {
  return (
    <dl className="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2">
      {items.map(([term, value]) =>
        value === undefined ? null : (
          <div key={term} className="min-w-0">
            <dt className="text-xs text-muted-foreground">{term}</dt>
            <dd className="break-words">{value}</dd>
          </div>
        ),
      )}
    </dl>
  );
}

function Heading({ defKey }: { defKey: string }) {
  const title = useTeamDefinitionField(defKey, "title");
  const purpose = useTeamDefinitionField(defKey, "purpose");
  return (
    <div className="min-w-0 space-y-1">
      <h2 id="teams-detail-heading" tabIndex={-1} className="break-words text-lg font-semibold focus-visible:outline-2 focus-visible:outline-ring">
        {title}
      </h2>
      <p className="font-mono text-xs break-all text-muted-foreground">{defKey}</p>
      {purpose && <p className="max-w-prose text-sm text-muted-foreground">{purpose}</p>}
    </div>
  );
}

function Members({ defKey }: { defKey: string }) {
  const members = useTeamDefinitionField(defKey, "members");
  const refs = useDefinitionRefs(defKey);
  const coordinator = useExportedCoordinatorRole(refs?.packageKey ?? "", refs?.definition ?? NO_DEFINITION);
  return (
    <Section id="definition-members" title="Members">
      {coordinator && <p className="text-sm">Coordinator role: <span className="font-medium">{coordinator}</span></p>}
      <ul className="space-y-2">
        {(members ?? []).map((member) => (
          <li key={`${member.role}:${member.definition.id}`} className="rounded-lg bg-muted/60 p-3 text-sm">
            <p className="flex flex-wrap items-baseline gap-x-2">
              <span className="font-medium">{member.role}</span>
              {member.role === coordinator && <span className="text-xs text-muted-foreground">(coordinator)</span>}
              <span className="text-xs text-muted-foreground">{member.kind}</span>
              <span className="text-xs text-muted-foreground">
                {member.min === member.max ? `exactly ${member.min}` : `${member.min} to ${member.max}`} per team
              </span>
            </p>
            <p className="font-mono text-xs break-all text-muted-foreground">{member.definition.id}@{member.definition.version}</p>
          </li>
        ))}
      </ul>
    </Section>
  );
}

function Requirements({ defKey }: { defKey: string }) {
  const refs = useDefinitionRefs(defKey);
  const packageKey = refs?.packageKey ?? "";
  const status = usePackageExportStatus(packageKey);
  const refusal = usePackageExportRefusal(packageKey);
  const loadStatus = useTeamLoadStatus(teamLoadKeys.packageExport(packageKey));
  const requirements = useExportedModelRequirements(packageKey, refs?.definition ?? NO_DEFINITION);
  return (
    <Section id="definition-models" title="Model requirements">
      {status === "refused" && (
        <p role="status" className="text-sm break-words text-muted-foreground">
          The runtime refused to export this package, so roles and model aliases cannot be shown.{refusal?.length ? ` ${refusal.join(" ")}` : ""}
        </p>
      )}
      {status === undefined && (
        <p role="status" className="text-sm text-muted-foreground">
          {loadStatus === "error" ? "The package export could not be loaded." : "Loading model requirements from the package export."}
        </p>
      )}
      {status === "exported" && requirements.length === 0 && (
        <p className="text-sm text-muted-foreground">The agents of this team declare no model requirement.</p>
      )}
      <ul className="space-y-2">
        {requirements.map((requirement) => (
          <li key={requirement.requirementKey} className="rounded-lg bg-muted/60 p-3 text-sm">
            <p className="font-medium">{requirement.role} model</p>
            <p className="break-words text-xs text-muted-foreground">
              Preferred aliases: {requirement.preferredAliases.length ? requirement.preferredAliases.join(", ") : "none listed"}
              {requirement.capabilities.length > 0 && `. Capabilities: ${requirement.capabilities.join(", ")}`}
            </p>
            <p className="text-xs text-muted-foreground">
              Used by {requirement.agentDefinitionId}
              {requirement.alsoUsedBy.length > 0 && ` and ${requirement.alsoUsedBy.length} other ${requirement.alsoUsedBy.length === 1 ? "agent" : "agents"}`}
            </p>
          </li>
        ))}
      </ul>
    </Section>
  );
}

function LimitsAndBudget({ defKey }: { defKey: string }) {
  const limits = useTeamDefinitionField(defKey, "limits");
  const budget = useTeamDefinitionField(defKey, "budget");
  const count = (value: number | undefined) => (value === undefined ? undefined : formatCount(value));
  return (
    <Section id="definition-limits" title="Limits and budget">
      <Facts
        items={[
          ["Concurrent turns", count(limits?.concurrentTurns)],
          ["Maximum members", count(limits?.maxMembers)],
          ["Maximum delegation depth", count(limits?.maxDepth)],
          ["Maximum tasks (every task ever created counts)", count(limits?.maxPendingTasks)],
          ["Token budget", count(budget?.maxTokens)],
          [
            "Cost budget",
            budget?.maxCostMicrounits === undefined ? undefined : formatMicrocost(budget.maxCostMicrounits, budget.currency),
          ],
          ["Elapsed-time budget", budget?.maxElapsedSeconds === undefined ? undefined : formatSeconds(budget.maxElapsedSeconds)],
          ["Budget currency", budget?.currency],
        ]}
      />
      {budget?.currency !== undefined && budget.currency !== "USD" && (
        <p role="status" className="text-sm text-warning">
          The budget currency is {budget.currency}. The runtime admits work only when it is USD, so routing tests will be refused.
        </p>
      )}
    </Section>
  );
}

function PackageFacts({ defKey }: { defKey: string }) {
  const refs = useDefinitionRefs(defKey);
  const packageKey = refs?.packageKey ?? "";
  const installedAt = usePackageField(packageKey, "installedAt");
  const definitionKeys = usePackageField(packageKey, "definitionKeys");
  return (
    <Section id="definition-package" title="Package">
      <Facts
        items={[
          ["Package", refs ? packageKey : undefined],
          ["Digest", refs ? shortDigest(refs.package.digest) : undefined],
          ["Installed", installedAt ? formatDateTime(installedAt) : undefined],
          ["Definitions in this package", definitionKeys ? formatCount(definitionKeys.length) : undefined],
        ]}
      />
    </Section>
  );
}

function BindingItem({ workspace, bindingId }: { workspace: string; bindingId: string }) {
  const revision = useBindingField(workspace, bindingId, "revision");
  const supported = useBindingField(workspace, bindingId, "activationSupported");
  const updatedAt = useBindingField(workspace, bindingId, "updatedAt");
  return (
    <li id={`binding-row-${bindingId}`} tabIndex={-1} className="rounded-lg bg-muted/60 p-3 text-sm focus-visible:outline-2 focus-visible:outline-ring">
      <p className="font-mono text-xs break-all">{bindingId}</p>
      <p className="text-xs text-muted-foreground">
        Revision {revision}. {supported ? "Activation supported." : "Activation not supported."} Updated {formatDateTime(updatedAt)}.
      </p>
    </li>
  );
}

function Bindings({ workspace, defKey }: { workspace: string; defKey: string }) {
  const refs = useDefinitionRefs(defKey);
  const ids = useBindingIdsForPackage(workspace, refs?.packageKey ?? "");
  return (
    <Section id="definition-bindings" title={`Bindings in workspace "${workspace}"`}>
      {ids.length === 0 ? (
        <p className="text-sm text-muted-foreground">No binding in workspace "{workspace}" for this package.</p>
      ) : (
        <ul className="space-y-2">
          {ids.map((id) => <BindingItem key={id} workspace={workspace} bindingId={id} />)}
        </ul>
      )}
    </Section>
  );
}

function NextStep({ workspace, defKey, onCreateBinding, onLaunch }: {
  workspace: string;
  defKey: string;
  onCreateBinding: () => void;
  onLaunch: () => void;
}) {
  const refs = useDefinitionRefs(defKey);
  const bindingCount = useBindingIdsForPackage(workspace, refs?.packageKey ?? "").length;
  const bindReason = useBindGate(workspace, defKey);
  const launchGate = useActionGate(workspace, "launch");
  const launchReason = launchGate ?? (bindingCount === 0 ? `Create a binding in workspace "${workspace}" first.` : null);
  const bind = (
    <GatedAction reason={bindReason} variant={bindingCount === 0 ? "default" : "secondary"} onClick={onCreateBinding}>
      <Link2 aria-hidden="true" />
      {bindingCount === 0 ? "Create binding" : "Create another binding"}
    </GatedAction>
  );
  const launch = (
    <GatedAction reason={launchReason} onClick={onLaunch}>
      <Rocket aria-hidden="true" />
      Launch team
    </GatedAction>
  );
  return (
    <Section id="definition-next-step" title="Next step">
      <p className="max-w-prose text-sm text-muted-foreground">
        {bindingCount === 0
          ? `This definition needs a binding in workspace "${workspace}" before a team can launch. A binding maps its model aliases to a provider and model.`
          : "Launch a team from a binding, then send it a routing test from the team page."}
      </p>
      <div className="flex flex-wrap items-start gap-3">
        {bindingCount === 0 ? bind : launch}
        {bindingCount === 0 ? launch : bind}
      </div>
    </Section>
  );
}

interface DefinitionDetailProps {
  workspace: string;
  defKey: string;
  onBack: () => void;
  onCreateBinding: () => void;
  onLaunch: () => void;
}

/** Shown when the definition record is absent: a failed catalog load is reported as such, never as "not found". */
function Missing({ defKey }: { defKey: string }) {
  const actions = useTeamActions();
  const error = useTeamLoadError(teamLoadKeys.definitions);
  const loading = useTeamLoadStatus(teamLoadKeys.definitions) === "loading";
  if (error) {
    return (
      <div className="space-y-2">
        <h2 id="teams-detail-heading" tabIndex={-1} className="text-lg font-semibold">The team definitions could not be loaded</h2>
        <p role="alert" className="max-w-prose text-sm break-words text-destructive">
          {error.message} <span className="font-mono text-xs">({error.code})</span>
        </p>
        <p className="max-w-prose text-sm text-muted-foreground">
          The definition <span className="font-mono break-all">{defKey}</span> cannot be shown until the list loads.
        </p>
        {error.kind !== "unauthorized" && (
          <Button type="button" variant="secondary" className="min-h-11" disabled={loading} onClick={() => void actions.loadTeamDefinitions().catch(() => undefined)}>
            Retry
          </Button>
        )}
      </div>
    );
  }
  return (
    <div className="space-y-2">
      <h2 id="teams-detail-heading" tabIndex={-1} className="text-lg font-semibold">Definition not found</h2>
      <p className="max-w-prose text-sm text-muted-foreground">
        No installed team definition has the key <span className="font-mono break-all">{defKey}</span>. It may belong to another owner, or the package that carries it is not a team package.
      </p>
    </div>
  );
}

export function DefinitionDetail({ workspace, defKey, onBack, onCreateBinding, onLaunch }: DefinitionDetailProps) {
  useDefinitionExportLoader(defKey);
  const title = useTeamDefinitionField(defKey, "title");
  const status = useTeamLoadStatus(teamLoadKeys.definitions);
  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="space-y-6 p-4 sm:p-6">
        <Button type="button" variant="ghost" className="min-h-11 md:hidden" onClick={onBack}>
          <ArrowLeft aria-hidden="true" />
          Back to list
        </Button>
        {title === undefined ? (
          status === "ready" || status === "error" ? (
            <Missing defKey={defKey} />
          ) : (
            <div role="status" aria-label="Loading definition" className="space-y-3">
              <Skeleton className="h-6 w-1/2" />
              <Skeleton className="h-4 w-3/4" />
            </div>
          )
        ) : (
          <>
            <Heading defKey={defKey} />
            <NextStep workspace={workspace} defKey={defKey} onCreateBinding={onCreateBinding} onLaunch={onLaunch} />
            <Members defKey={defKey} />
            <Requirements defKey={defKey} />
            <LimitsAndBudget defKey={defKey} />
            <Bindings workspace={workspace} defKey={defKey} />
            <PackageFacts defKey={defKey} />
          </>
        )}
      </div>
    </ScrollArea>
  );
}
