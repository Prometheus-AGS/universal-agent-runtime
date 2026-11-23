import { useId } from "react";
import { ModelPicker } from "@/features/models/pickers";
import { Label } from "@/components/ui/label";
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select";
import { useConfiguredModels, type ConversionDiagnostic, type ModelRequirement } from "@/platform/entities";
import { aliasFor, type ModelChoice } from "../model/binding-draft";
import { DiagnosticList } from "./runtime-notice";

interface RequirementRowProps {
  requirement: ModelRequirement;
  choice: ModelChoice | undefined;
  diagnostics: readonly ConversionDiagnostic[];
  onChange: (patch: Partial<ModelChoice>) => void;
}

/** One row per agent model requirement: which preferred alias to satisfy, and the provider and model that answers it. */
function RequirementRow({ requirement, choice, diagnostics, onChange }: RequirementRowProps) {
  const aliasId = useId();
  const modelId = useId();
  const { items } = useConfiguredModels();
  const alias = aliasFor(requirement, choice);
  const choose = (value: string) => {
    const model = items.find((candidate) => candidate.id === value);
    onChange({ route: model ? { value, providerId: model.provider_id, modelId: model.model_id } : null });
  };
  return (
    <li className="space-y-3 rounded-lg bg-muted/60 p-3">
      <div>
        <p className="text-sm font-medium">{requirement.role} model</p>
        <p className="text-xs break-words text-muted-foreground">
          Used by {requirement.agentDefinitionId}
          {requirement.alsoUsedBy.length > 0 && ` and ${requirement.alsoUsedBy.length} other ${requirement.alsoUsedBy.length === 1 ? "agent" : "agents"}`}
        </p>
      </div>
      {requirement.preferredAliases.length > 1 ? (
        <div className="space-y-1.5">
          <Label htmlFor={aliasId}>Alias this binding answers</Label>
          <NativeSelect className="w-full" id={aliasId} value={alias ?? ""} onChange={(event) => onChange({ alias: event.target.value })}>
            {requirement.preferredAliases.map((name) => (
              <NativeSelectOption key={name} value={name}>{name}</NativeSelectOption>
            ))}
          </NativeSelect>
        </div>
      ) : (
        <p className="text-sm">Alias: <span className="font-mono">{alias}</span></p>
      )}
      <div className="space-y-1.5">
        <p id={modelId} className="text-sm leading-none font-medium">Provider and model</p>
        <div role="group" aria-labelledby={modelId}>
          <ModelPicker value={choice?.route?.value ?? ""} onChange={choose} ariaLabel="Provider and model" placeholder="Choose a provider and model" className="min-h-11" />
        </div>
      </div>
      <DiagnosticList label={`Check results for the ${requirement.role} model`} diagnostics={diagnostics} />
    </li>
  );
}

interface ModelStepProps {
  requirements: readonly ModelRequirement[];
  choices: Record<string, ModelChoice>;
  byRequirement: Record<number, readonly ConversionDiagnostic[]>;
  /** `/models/N` diagnostics point into an agent definition, so they are grouped here rather than beside one row. */
  modelDiagnostics: readonly ConversionDiagnostic[];
  onChange: (requirementKey: string, patch: Partial<ModelChoice>) => void;
}

export function ModelStep({ requirements, choices, byRequirement, modelDiagnostics, onChange }: ModelStepProps) {
  return (
    <section aria-labelledby="binding-models-heading" className="space-y-3">
      <h3 id="binding-models-heading" className="text-sm font-semibold">Step 1. Models</h3>
      <p className="text-sm text-muted-foreground">
        Each agent in the team asks for a model by alias. Choose the provider and model that answers each request.
      </p>
      {modelDiagnostics.length > 0 && (
        <div className="space-y-2">
          <p className="text-sm font-medium">Package model checks (these refer to the agents' definitions, not to one row below)</p>
          <DiagnosticList label="Package model checks" diagnostics={modelDiagnostics} />
        </div>
      )}
      <ul className="space-y-3">
        {requirements.map((requirement, index) => (
          <RequirementRow
            key={requirement.requirementKey}
            requirement={requirement}
            choice={choices[requirement.requirementKey]}
            diagnostics={byRequirement[index] ?? []}
            onChange={(patch) => onChange(requirement.requirementKey, patch)}
          />
        ))}
      </ul>
    </section>
  );
}
