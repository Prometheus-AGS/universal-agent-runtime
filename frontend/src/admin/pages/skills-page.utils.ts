export interface SkillEditorFormState {
  title: string;
  version: string;
  description: string;
  promptOverlay: string;
  keywords: string;
  preferredTools: string;
  enabled: boolean;
  /** "provider/model" string or empty string for no override */
  preferredModel: string;
}

export interface SkillExecutionConfigRequest {
  preferred_model?: string | null;
}

export interface SkillCreateRequest {
  name: string;
  version: string;
  description: string;
  triggers: {
    keywords: string[];
    semantic?: string | null;
  };
  prompt_overlay: string;
  preferred_tools: string[];
  enabled: boolean;
  execution_config: SkillExecutionConfigRequest;
}

export interface SkillUpdateRequest {
  version: string;
  title: string;
  description: string;
  triggers: {
    keywords: string[];
    semantic?: string | null;
  };
  prompt_overlay: string;
  preferred_tools: string[];
  enabled: boolean;
  execution_config: SkillExecutionConfigRequest;
}

export const DEFAULT_SKILL_FORM: SkillEditorFormState = {
  title: "",
  version: "1.0.0",
  description: "",
  promptOverlay: "",
  keywords: "",
  preferredTools: "",
  enabled: true,
  preferredModel: "",
};

export function parseCommaSeparated(value: string): string[] {
  return value
    .split(",")
    .map((entry) => entry.trim())
    .filter((entry) => entry.length > 0);
}

export function joinCommaSeparated(values: string[] | undefined): string {
  if (!values || values.length === 0) return "";
  return values.join(", ");
}

function buildExecutionConfig(form: SkillEditorFormState): SkillExecutionConfigRequest {
  const model = form.preferredModel.trim() || null;
  return { preferred_model: model };
}

export function buildCreateSkillRequest(form: SkillEditorFormState): SkillCreateRequest {
  return {
    name: form.title.trim(),
    version: form.version.trim() || "1.0.0",
    description: form.description,
    triggers: {
      keywords: parseCommaSeparated(form.keywords),
    },
    prompt_overlay: form.promptOverlay,
    preferred_tools: parseCommaSeparated(form.preferredTools),
    enabled: form.enabled,
    execution_config: buildExecutionConfig(form),
  };
}

export function buildUpdateSkillRequest(form: SkillEditorFormState): SkillUpdateRequest {
  return {
    version: form.version.trim() || "1.0.0",
    title: form.title.trim(),
    description: form.description,
    triggers: {
      keywords: parseCommaSeparated(form.keywords),
    },
    prompt_overlay: form.promptOverlay,
    preferred_tools: parseCommaSeparated(form.preferredTools),
    enabled: form.enabled,
    execution_config: buildExecutionConfig(form),
  };
}
