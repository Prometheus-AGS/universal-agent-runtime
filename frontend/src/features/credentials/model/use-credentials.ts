import { useCredentialsStore } from "./credentials-store";

/** Expose credential state and actions to the admin UI. */
export function useCredentials() {
  return {
    state: useCredentialsStore((current) => current.state),
    credentials: useCredentialsStore((current) => current.credentials),
    loading: useCredentialsStore((current) => current.loading),
    saving: useCredentialsStore((current) => current.saving),
    removing: useCredentialsStore((current) => current.removing),
    error: useCredentialsStore((current) => current.error),
    load: useCredentialsStore((current) => current.load),
    save: useCredentialsStore((current) => current.save),
    remove: useCredentialsStore((current) => current.remove),
  };
}
