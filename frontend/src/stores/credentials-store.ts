import { create } from "zustand";

import type { CredentialServiceState, CredentialView } from "@/entities/credential-types";
import { deleteCredential, listCredentials, putCredential } from "@/services/credentials-api";

interface CredentialsState {
  state: CredentialServiceState;
  credentials: CredentialView[];
  loading: boolean;
  saving: boolean;
  removing: boolean;
  error: string | null;
}

interface CredentialsActions {
  load: () => Promise<void>;
  save: (providerId: string, apiKey: string) => Promise<boolean>;
  remove: (providerId: string) => Promise<boolean>;
}

export const useCredentialsStore = create<CredentialsState & CredentialsActions>((set, get) => ({
  state: "ok",
  credentials: [],
  loading: true,
  saving: false,
  removing: false,
  error: null,

  load: async () => {
    set({ loading: true, error: null });
    try {
      const result = await listCredentials();
      set({
        state: result.state,
        credentials: [...result.credentials].sort((a, b) =>
          a.provider_id.localeCompare(b.provider_id),
        ),
        loading: false,
      });
    } catch (error) {
      set({ error: (error as Error).message, loading: false });
    }
  },

  save: async (providerId, apiKey) => {
    set({ saving: true, error: null });
    try {
      await putCredential(providerId, apiKey);
      await get().load();
      return true;
    } catch (error) {
      set({ error: (error as Error).message });
      return false;
    } finally {
      set({ saving: false });
    }
  },

  remove: async (providerId) => {
    set({ removing: true, error: null });
    try {
      await deleteCredential(providerId);
      set((current) => ({
        credentials: current.credentials.filter((credential) => credential.provider_id !== providerId),
      }));
      return true;
    } catch (error) {
      set({ error: (error as Error).message });
      return false;
    } finally {
      set({ removing: false });
    }
  },
}));
