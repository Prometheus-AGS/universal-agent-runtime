import { create } from "zustand";

import { checkHasConfiguredProviders, checkHasKnowledgeBases } from "@/services/onboarding-api";

interface OnboardingWelcomeActions {
  checkConfiguredProviders: () => Promise<boolean>;
  checkKnowledgeBases: () => Promise<boolean>;
}

export const useOnboardingWelcomeStore = create<OnboardingWelcomeActions>(() => ({
  checkConfiguredProviders: () => checkHasConfiguredProviders(),
  checkKnowledgeBases: () => checkHasKnowledgeBases(),
}));
