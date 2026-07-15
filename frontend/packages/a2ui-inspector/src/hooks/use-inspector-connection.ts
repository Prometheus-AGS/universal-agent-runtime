import { useEffect } from "react";
import type { A2uiStreamService } from "../services/a2ui-stream-service";
import type { InspectorStore } from "../stores/inspector-store";
export function useInspectorConnection(store: InspectorStore, service: A2uiStreamService) { useEffect(() => store.getState().connect(service), [service, store]); }
