export interface SkillEditorFormState {
  title: string;
  version: string;
  description: string;
  promptOverlay: string;
  keywords: string;
  preferredTools: string;
  enabled: boolean;
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
}

export const DEFAULT_SKILL_FORM: SkillEditorFormState = {
  title: "",
  version: "1.0.0",
  description: "",
  promptOverlay: "",
  keywords: "",
  preferredTools: "",
  enabled: true,
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
  };
}
