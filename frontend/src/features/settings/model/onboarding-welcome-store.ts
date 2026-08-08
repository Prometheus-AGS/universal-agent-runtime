import { create } from "zustand";

import { checkHasConfiguredProviders, checkHasKnowledgeBases } from "../api/onboarding-api";
import { fetchResolveModel } from "@/features/models/api";

interface OnboardingWelcomeActions {
  checkConfiguredProviders: () => Promise<boolean>;
  checkKnowledgeBases: () => Promise<boolean>;
  checkDefaultModel: () => Promise<boolean>;
}

export const useOnboardingWelcomeStore = create<OnboardingWelcomeActions>(() => ({
  checkConfiguredProviders: () => checkHasConfiguredProviders(),
  checkKnowledgeBases: () => checkHasKnowledgeBases(),
  checkDefaultModel: () => fetchResolveModel().then((r) => r.ok).catch(() => false),
}));
