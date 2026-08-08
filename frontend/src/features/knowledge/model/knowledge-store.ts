import { create } from "zustand";
import { serializeKey, useGraphStore } from "@/platform/entities";

import { optimisticRemove } from "@/lib/realtime/optimistic";
import {
  createKnowledgeBase,
  deleteDocument,
  deleteKnowledgeBase,
  fetchDocuments,
  fetchKnowledgeBases,
  searchKnowledgeBase,
  uploadDocument,
} from "../api/knowledge-api";
import type { KbSearchResult, UarKnowledgeBase, UarKnowledgeDocument } from "@/types";

interface KnowledgeState {
  loading: boolean;
  error: string | null;
  saving: boolean;
  deleting: boolean;
  docsByKb: Record<string, UarKnowledgeDocument[]>;
  activeKbId: string | null;
  docsLoadingKbId: string | null;
  docsError: string | null;
  uploading: boolean;
  uploadProgress: string | null;
  searchResults: KbSearchResult[] | null;
  searching: boolean;
  deletingDoc: boolean;
  retryingDocId: string | null;
}

interface KnowledgeActions {
  loadBases: () => Promise<void>;
  addBase: (form: { name: string; description: string }) => Promise<void>;
  removeBase: (id: string) => Promise<void>;
  loadDocs: (kbId: string) => Promise<void>;
  uploadFiles: (kbId: string, files: FileList | File[]) => Promise<void>;
  removeDocument: (kbId: string, doc: UarKnowledgeDocument) => Promise<void>;
  retryDocument: (kbId: string, doc: UarKnowledgeDocument, file?: File) => Promise<void>;
  runSearch: (kbId: string, query: string) => Promise<void>;
  clearSearch: () => void;
  clearDocView: (kbId?: string) => void;
  setDocsError: (message: string | null) => void;
  setUploadProgress: (message: string | null) => void;
}

export type KnowledgeStore = KnowledgeState & KnowledgeActions;

const uploadSources = new Map<string, File>();
const uploadKey = (kbId: string, filename: string) => `${kbId}\u0000${filename}`;

function reconcileBases(bases: UarKnowledgeBase[]): void {
  const graph = useGraphStore.getState();
  const nextIds = new Set(bases.map((base) => base.id));
  for (const id of Object.keys(graph.entities.KnowledgeBase ?? {})) {
    if (!nextIds.has(id)) graph.removeEntity("KnowledgeBase", id);
  }
  for (const base of bases) {
    graph.upsertEntity("KnowledgeBase", base.id, base as unknown as Record<string, unknown>);
  }
  graph.setListResult(serializeKey(["knowledge-bases", ""]), [...nextIds], {
    total: nextIds.size,
  });
}

function reconcileDocuments(kbId: string, documents: UarKnowledgeDocument[]): void {
  const graph = useGraphStore.getState();
  const nextIds = new Set(documents.map((document) => document.id));
  for (const [id, entity] of Object.entries(graph.entities.Document ?? {})) {
    if (entity.kb_id === kbId && !nextIds.has(id)) graph.removeEntity("Document", id);
  }
  for (const document of documents) {
    graph.upsertEntity("Document", document.id, document as unknown as Record<string, unknown>);
  }
}

export const useKnowledgeStore = create<KnowledgeStore>((set, get) => ({
  loading: false,
  error: null,
  saving: false,
  deleting: false,
  docsByKb: {},
  activeKbId: null,
  docsLoadingKbId: null,
  docsError: null,
  uploading: false,
  uploadProgress: null,
  searchResults: null,
  searching: false,
  deletingDoc: false,
  retryingDocId: null,

  loadBases: async () => {
    set({ loading: true, error: null });
    try {
      reconcileBases(await fetchKnowledgeBases());
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ loading: false });
    }
  },
  addBase: async (form) => {
    set({ saving: true, error: null });
    try {
      const created = await createKnowledgeBase(form);
      useGraphStore.getState().upsertEntity(
        "KnowledgeBase",
        created.id,
        created as unknown as Record<string, unknown>,
      );
      await get().loadBases();
    } catch (error) {
      set({ error: (error as Error).message });
      throw error;
    } finally {
      set({ saving: false });
    }
  },
  removeBase: async (id) => {
    set({ deleting: true, error: null });
    try {
      await optimisticRemove("KnowledgeBase", id, () => deleteKnowledgeBase(id));
      set((state) => {
        const docsByKb = Object.fromEntries(
          Object.entries(state.docsByKb).filter(([kbId]) => kbId !== id),
        );
        return { docsByKb };
      });
    } catch (error) {
      set({ error: `Failed to remove knowledge base: ${(error as Error).message}` });
      throw error;
    } finally {
      set({ deleting: false });
    }
  },
  loadDocs: async (kbId) => {
    set({ activeKbId: kbId, docsLoadingKbId: kbId, docsError: null });
    try {
      const documents = await fetchDocuments(kbId);
      reconcileDocuments(kbId, documents);
      set((state) => ({ docsByKb: { ...state.docsByKb, [kbId]: documents } }));
    } catch (error) {
      set({ docsError: (error as Error).message });
      throw error;
    } finally {
      set({ docsLoadingKbId: null });
    }
  },
  uploadFiles: async (kbId, files) => {
    const selected = Array.from(files);
    set({ uploading: true, uploadProgress: null, docsError: null });
    try {
      for (const [index, file] of selected.entries()) {
        set({ uploadProgress: `Uploading ${file.name} (${index + 1} of ${selected.length})…` });
        uploadSources.set(uploadKey(kbId, file.name), file);
        await uploadDocument(kbId, file);
      }
      await get().loadDocs(kbId);
    } catch (error) {
      set({ docsError: (error as Error).message });
      throw error;
    } finally {
      set({ uploading: false, uploadProgress: null });
    }
  },
  removeDocument: async (kbId, doc) => {
    const snapshot = get().docsByKb[kbId] ?? [];
    set({
      deletingDoc: true,
      docsError: null,
      docsByKb: { ...get().docsByKb, [kbId]: snapshot.filter((item) => item.id !== doc.id) },
    });
    try {
      await optimisticRemove("Document", doc.id, () => deleteDocument(kbId, doc.id));
    } catch (error) {
      set({ docsByKb: { ...get().docsByKb, [kbId]: snapshot }, docsError: (error as Error).message });
      throw error;
    } finally {
      set({ deletingDoc: false });
    }
  },
  retryDocument: async (kbId, doc, replacement) => {
    const file = replacement ?? uploadSources.get(uploadKey(kbId, doc.filename));
    if (!file) throw new Error("Select the original file to retry this document.");
    set({ retryingDocId: doc.id, docsError: null });
    try {
      await get().removeDocument(kbId, doc);
      await get().uploadFiles(kbId, [file]);
    } catch (error) {
      set({ docsError: (error as Error).message });
      throw error;
    } finally {
      set({ retryingDocId: null });
    }
  },
  runSearch: async (kbId, query) => {
    set({ searching: true, docsError: null });
    try {
      set({ searchResults: await searchKnowledgeBase(kbId, query) });
    } catch (error) {
      set({ docsError: (error as Error).message });
      throw error;
    } finally {
      set({ searching: false });
    }
  },
  clearSearch: () => set({ searchResults: null }),
  clearDocView: (kbId) => set((state) => ({
    activeKbId: null,
    docsByKb: kbId ? { ...state.docsByKb, [kbId]: [] } : state.docsByKb,
    docsError: null,
    searchResults: null,
  })),
  setDocsError: (docsError) => set({ docsError }),
  setUploadProgress: (uploadProgress) => set({ uploadProgress }),
}));
