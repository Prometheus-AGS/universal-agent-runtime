import { useStore } from "zustand";
import type { InspectorStore } from "../stores/inspector-store";
export function useInspector(store: InspectorStore) { return useStore(store); }
