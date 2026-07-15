import type { A2uiStreamService } from "../services/a2ui-stream-service";
import type { InspectorStore } from "../stores/inspector-store";
import { useInspectorConnection } from "../hooks/use-inspector-connection";
import { InspectorPanel } from "./InspectorPanel";
export function InspectorApp({ store, service }: { store: InspectorStore; service: A2uiStreamService }) { useInspectorConnection(store, service); return <InspectorPanel store={store} />; }
