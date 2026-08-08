import { useEffect } from "react";

import { useDocuments, useKnowledgeBases } from "./use-knowledge";
import { useKnowledgeStore } from "./knowledge-store";
import type { KbSearchResult, UarKnowledgeBase, UarKnowledgeDocument } from "@/types";

const EMPTY_DOCUMENTS: UarKnowledgeDocument[] = [];

export interface KnowledgePageState {
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
  retryingDocId: string | null;
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

export function useKnowledgePage(): KnowledgePageState {
  const basesView = useKnowledgeBases();
  const activeKbId = useKnowledgeStore((state) => state.activeKbId);
  const graphDocuments = useDocuments(activeKbId ?? undefined);
  const storedDocuments = useKnowledgeStore(
    (state) => (activeKbId ? state.docsByKb[activeKbId] ?? EMPTY_DOCUMENTS : EMPTY_DOCUMENTS),
  );

  const loading = useKnowledgeStore((state) => state.loading);
  const error = useKnowledgeStore((state) => state.error);
  const saving = useKnowledgeStore((state) => state.saving);
  const deleting = useKnowledgeStore((state) => state.deleting);
  const docsLoadingKbId = useKnowledgeStore((state) => state.docsLoadingKbId);
  const docsError = useKnowledgeStore((state) => state.docsError);
  const uploading = useKnowledgeStore((state) => state.uploading);
  const uploadProgress = useKnowledgeStore((state) => state.uploadProgress);
  const searchResults = useKnowledgeStore((state) => state.searchResults);
  const searching = useKnowledgeStore((state) => state.searching);
  const deletingDoc = useKnowledgeStore((state) => state.deletingDoc);
  const retryingDocId = useKnowledgeStore((state) => state.retryingDocId);

  const loadBases = useKnowledgeStore((state) => state.loadBases);
  const addBase = useKnowledgeStore((state) => state.addBase);
  const removeBase = useKnowledgeStore((state) => state.removeBase);
  const loadDocs = useKnowledgeStore((state) => state.loadDocs);
  const uploadFiles = useKnowledgeStore((state) => state.uploadFiles);
  const removeDocument = useKnowledgeStore((state) => state.removeDocument);
  const retryDocument = useKnowledgeStore((state) => state.retryDocument);
  const runSearch = useKnowledgeStore((state) => state.runSearch);
  const clearSearch = useKnowledgeStore((state) => state.clearSearch);
  const clearDocView = useKnowledgeStore((state) => state.clearDocView);
  const setDocsError = useKnowledgeStore((state) => state.setDocsError);
  const setUploadProgress = useKnowledgeStore((state) => state.setUploadProgress);

  useEffect(() => {
    void loadBases().catch(() => undefined);
  }, [loadBases]);

  return {
    bases: basesView.items as unknown as UarKnowledgeBase[],
    loading,
    error,
    saving,
    deleting,
    docs: (graphDocuments.length > 0 ? graphDocuments : storedDocuments) as UarKnowledgeDocument[],
    docsKbId: activeKbId,
    docsLoading: docsLoadingKbId === activeKbId,
    docsError,
    uploading,
    uploadProgress,
    searchResults,
    searching,
    deletingDoc,
    retryingDocId,
    loadBases,
    addBase,
    removeBase,
    loadDocs,
    uploadFiles,
    removeDocument,
    retryDocument,
    runSearch,
    clearSearch,
    clearDocView,
    setDocsError,
    setUploadProgress,
  };
}
