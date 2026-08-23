import { graphStore } from "@prometheus-ags/prometheus-entity-management";

export function useSessionModelField() {
  return { value: graphStore.getState().entities.AgentSession?.current?.model ?? "" };
}

export function setSessionModel(value: string) {
  graphStore.getState().replaceEntity("AgentSession", "current", {
    id: "current",
    model: value,
  });
}
