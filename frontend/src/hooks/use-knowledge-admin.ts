import { useEffect } from "react";

import { useKnowledgeAdminStore } from "@/stores/knowledge-admin-store";

export function useKnowledgeAdmin() {
  const bases = useKnowledgeAdminStore((s) => s.bases);
  const loading = useKnowledgeAdminStore((s) => s.loading);
  const error = useKnowledgeAdminStore((s) => s.error);
  const saving = useKnowledgeAdminStore((s) => s.saving);
  const deleting = useKnowledgeAdminStore((s) => s.deleting);
  const docs = useKnowledgeAdminStore((s) => s.docs);
  const docsKbId = useKnowledgeAdminStore((s) => s.docsKbId);
  const docsLoading = useKnowledgeAdminStore((s) => s.docsLoading);
  const docsError = useKnowledgeAdminStore((s) => s.docsError);
  const uploading = useKnowledgeAdminStore((s) => s.uploading);
  const uploadProgress = useKnowledgeAdminStore((s) => s.uploadProgress);
  const searchResults = useKnowledgeAdminStore((s) => s.searchResults);
  const searching = useKnowledgeAdminStore((s) => s.searching);
  const deletingDoc = useKnowledgeAdminStore((s) => s.deletingDoc);
  const loadBases = useKnowledgeAdminStore((s) => s.loadBases);
  const addBase = useKnowledgeAdminStore((s) => s.addBase);
  const removeBase = useKnowledgeAdminStore((s) => s.removeBase);
  const loadDocs = useKnowledgeAdminStore((s) => s.loadDocs);
  const uploadFiles = useKnowledgeAdminStore((s) => s.uploadFiles);
  const removeDocument = useKnowledgeAdminStore((s) => s.removeDocument);
  const runSearch = useKnowledgeAdminStore((s) => s.runSearch);
  const clearSearch = useKnowledgeAdminStore((s) => s.clearSearch);
  const clearDocView = useKnowledgeAdminStore((s) => s.clearDocView);
  const setDocsError = useKnowledgeAdminStore((s) => s.setDocsError);
  const setUploadProgress = useKnowledgeAdminStore((s) => s.setUploadProgress);

  useEffect(() => {
    void loadBases();
  }, [loadBases]);

  return {
    bases,
    loading,
    error,
    saving,
    deleting,
    docs,
    docsKbId,
    docsLoading,
    docsError,
    uploading,
    uploadProgress,
    searchResults,
    searching,
    deletingDoc,
    loadBases,
    addBase,
    removeBase,
    loadDocs,
    uploadFiles,
    removeDocument,
    runSearch,
    clearSearch,
    clearDocView,
    setDocsError,
    setUploadProgress,
  };
}
