import { create as makeStore } from "zustand";

interface ConfiguredModel {
  id: string;
}

export const useConfiguredModelCache = makeStore<{ models: ConfiguredModel[] }>(() => ({
  models: [],
}));
