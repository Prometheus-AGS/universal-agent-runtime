import { useCallback } from "react";
import { useOnboardingWelcomeStore } from "./onboarding-welcome-store";

/** Check one onboarding milestone through its owning store or local UI preference. */
export function useOnboardingWelcome() {
  const checkProviders = useOnboardingWelcomeStore((state) => state.checkConfiguredProviders);
  const checkDefaultModel = useOnboardingWelcomeStore((state) => state.checkDefaultModel);
  const checkKnowledge = useOnboardingWelcomeStore((state) => state.checkKnowledgeBases);

  return useCallback(async (id: string) => {
    if (id === "provider") return checkProviders();
    if (id === "default-model") return checkDefaultModel();
    if (id === "knowledge") return checkKnowledge();
    try {
      return localStorage.getItem("uar-onboarding-settings-visited") === "1";
    } catch {
      return false;
    }
  }, [checkDefaultModel, checkKnowledge, checkProviders]);
}
