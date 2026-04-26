import { create } from "zustand";

import type { KbSearchResult } from "@/types";
import { onSettingsChanged } from "@/services/settings-change-bus";
import {
  createKnowledgeBase,
  deleteDocument,
  deleteKnowledgeBase,
  fetchDocuments,
  fetchKnowledgeBases,
  searchKnowledgeBase,
  uploadDocument,
} from "@/services/knowledge-api";
import type { UarKnowledgeBase, UarKnowledgeDocument } from "@/types";

interface KnowledgeAdminState {
  bases: UarKnowledgeBase[];
  loading: boolean;
  error: string | null;
  saving: boolean;
  deleting: boolean;
  docs: UarKnowledgeDocument[];
  docsKbId: string | null;
  docsLoading: boolean;
  docsError: string | null;
  uploading: boolean;
  uploadProgress: string | null;
  searchResults: KbSearchResult[] | null;
  searching: boolean;
  deletingDoc: boolean;
}

interface KnowledgeAdminActions {
  loadBases: () => Promise<void>;
  addBase: (form: { name: string; description: string }) => Promise<void>;
  removeBase: (id: string) => Promise<void>;
  loadDocs: (kbId: string) => Promise<void>;
  uploadFiles: (kbId: string, files: FileList) => Promise<void>;
  removeDocument: (kbId: string, doc: UarKnowledgeDocument) => Promise<void>;
  runSearch: (kbId: string, query: string) => Promise<void>;
  clearSearch: () => void;
  clearDocView: () => void;
  setDocsError: (msg: string | null) => void;
  setUploadProgress: (msg: string | null) => void;
}

export const useKnowledgeAdminStore = create<
  KnowledgeAdminState & KnowledgeAdminActions
>((set, get) => ({
  bases: [],
  loading: true,
  error: null,
  saving: false,
  deleting: false,
  docs: [],
  docsKbId: null,
  docsLoading: false,
  docsError: null,
  uploading: false,
  uploadProgress: null,
  searchResults: null,
  searching: false,
  deletingDoc: false,

  setDocsError: (msg) => set({ docsError: msg }),
  setUploadProgress: (msg) => set({ uploadProgress: msg }),
  clearSearch: () => set({ searchResults: null }),

  clearDocView: () =>
    set({ docs: [], docsKbId: null, docsError: null, searchResults: null }),

  loadBases: async () => {
    set({ loading: true, error: null });
    try {
      const bases = await fetchKnowledgeBases();
      set({ bases, loading: false });
    } catch (e) {
      set({ error: (e as Error).message, loading: false });
    }
  },

  addBase: async (form) => {
    set({ saving: true, error: null });
    try {
      await createKnowledgeBase(form);
      await get().loadBases();
      set({ saving: false });
    } catch (e) {
      set({ error: (e as Error).message, saving: false });
      throw e;
    }
  },

  removeBase: async (id) => {
    set({ deleting: true, error: null });
    try {
      await deleteKnowledgeBase(id);
      set((s) => ({
        bases: s.bases.filter((b) => b.id !== id),
        docs: s.docsKbId === id ? [] : s.docs,
        docsKbId: s.docsKbId === id ? null : s.docsKbId,
        deleting: false,
      }));
      await get().loadBases();
    } catch {
      set({ deleting: false });
    }
  },

  loadDocs: async (kbId) => {
    set({ docsLoading: true, docsError: null, docsKbId: kbId });
    try {
      const docs = await fetchDocuments(kbId);
      set({ docs, docsLoading: false });
    } catch (e) {
      set({ docsError: (e as Error).message, docsLoading: false });
    }
  },

  uploadFiles: async (kbId, files) => {
    set({ uploading: true, docsError: null });
    try {
      for (let i = 0; i < files.length; i++) {
        set({
          uploadProgress: `Uploading ${i + 1}/${files.length}: ${files[i].name}`,
        });
        await uploadDocument(kbId, files[i]);
      }
      set({ uploadProgress: null });
      await get().loadDocs(kbId);
      await get().loadBases();
    } catch (e) {
      set({ docsError: (e as Error).message, uploadProgress: null });
      throw e;
    } finally {
      set({ uploading: false });
    }
  },

  removeDocument: async (kbId, doc) => {
    set({ deletingDoc: true, docsError: null });
    try {
      await deleteDocument(kbId, doc.id);
      await get().loadDocs(kbId);
      await get().loadBases();
      set({ deletingDoc: false });
    } catch {
      set({ deletingDoc: false });
    }
  },

  runSearch: async (kbId, query) => {
    set({ searching: true, docsError: null });
    try {
      const searchResults = await searchKnowledgeBase(kbId, query.trim(), 10);
      set({ searchResults, searching: false });
    } catch (e) {
      set({ docsError: (e as Error).message, searching: false });
    }
  },
}));

if (typeof window !== "undefined") {
  onSettingsChanged((detail) => {
    if (detail.impact !== "rag") return;
    const state = useKnowledgeAdminStore.getState();
    void state.loadBases();
    if (state.docsKbId) {
      void state.loadDocs(state.docsKbId);
    }
  });
}
