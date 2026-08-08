export type SettingsImpact =
  | "llm"
  | "providers"
  | "rag"
  | "memory"
  | "tools"
  | "security"
  | "runtime"
  | "persistence"
  | "unknown";

export interface SettingsChangedDetail {
  namespace: string;
  key: string;
  value: unknown;
  source: "local" | "remote";
  updated_at?: string;
  impact: SettingsImpact;
}

export const SETTINGS_CHANGED_EVENT = "uar:settings-changed";

export function impactForSettingsNamespace(namespace: string): SettingsImpact {
  if (
    namespace === "llm" ||
    namespace === "llm_failover" ||
    namespace === "context_strategy"
  )
    return "llm";
  if (namespace === "provider" || namespace === "providers") return "providers";
  if (
    namespace === "rag" ||
    namespace === "knowledge_bases" ||
    namespace === "file_processing" ||
    namespace === "unstructured" ||
    namespace === "mistral_ocr" ||
    namespace === "kreuzberg"
  )
    return "rag";
  if (namespace === "memory") return "memory";
  if (
    namespace === "native_tools" ||
    namespace === "skill_config" ||
    namespace === "skill_evolution"
  )
    return "tools";
  if (
    namespace === "security" ||
    namespace === "sycophancy" ||
    namespace === "governance"
  )
    return "security";
  if (
    namespace === "server" ||
    namespace === "resilience" ||
    namespace === "sandbox" ||
    namespace === "acp"
  )
    return "runtime";
  if (namespace === "persistence" || namespace === "models")
    return "persistence";
  return "unknown";
}

export function emitSettingsChanged(
  detail: Omit<SettingsChangedDetail, "impact"> & { impact?: SettingsImpact },
) {
  if (typeof window === "undefined") return;
  const payload: SettingsChangedDetail = {
    ...detail,
    impact: detail.impact ?? impactForSettingsNamespace(detail.namespace),
  };
  window.dispatchEvent(
    new CustomEvent<SettingsChangedDetail>(SETTINGS_CHANGED_EVENT, {
      detail: payload,
    }),
  );
}

export function onSettingsChanged(
  handler: (detail: SettingsChangedDetail) => void,
): () => void {
  if (typeof window === "undefined") return () => {};
  const listener = (event: Event) => {
    handler((event as CustomEvent<SettingsChangedDetail>).detail);
  };
  window.addEventListener(SETTINGS_CHANGED_EVENT, listener);
  return () => window.removeEventListener(SETTINGS_CHANGED_EVENT, listener);
}
