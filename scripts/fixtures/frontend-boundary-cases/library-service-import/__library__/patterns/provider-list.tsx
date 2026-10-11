import { listProviders } from "./api/providers-api";

export function ProviderList() {
  return <span>{String(listProviders)}</span>;
}
