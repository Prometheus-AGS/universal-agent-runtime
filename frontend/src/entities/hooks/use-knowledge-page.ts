/**
 * Compatibility hook for the knowledge admin page.
 *
 * Exposes the same API surface as the retired `useKnowledgeAdmin` hook
 * (which sat on top of `knowledge-admin-store`) but routes reads through
 * the entity graph and writes through direct service calls + optimistic
 * helpers. Lets the page consume direct-pattern infrastructure without a
 * full rewrite of its 782 LOC.
 */
import { useCallback, useEffect, useState } from "react";
import { useGraphStore } from "@prometheus-ags/prometheus-entity-management";
import { useKnowledgeBases } from "@/entities/hooks/use-knowledge";
import { loadKnowledgeBasesIntoGraph } from "@/entities/fetchers/knowledge";
import { optimisticRemove, optimisticUpsert } from "@/lib/realtime/optimistic";
import {
  createKnowledgeBase,
  deleteDocument,
  deleteKnowledgeBase,
  fetchDocuments,
  searchKnowledgeBase,
  uploadDocument,
} from "@/services/knowledge-api";
import type {
  KbSearchResult,
  UarKnowledgeBase,
  UarKnowledgeDocument,
} from "@/types";

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

export function useKnowledgePage(): KnowledgePageState {
  // Knowledge bases — live view from the graph.
  const basesView = useKnowledgeBases();
  const bases = basesView.items as unknown as UarKnowledgeBase[];

  // Local UI state (replaces the old store-level flags).
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // Documents — also routed through the graph, but the page selects a
  // single KB at a time, so we keep a query-scoped view in local state
  // for the same shape consumers expect.
  const [docs, setDocs] = useState<UarKnowledgeDocument[]>([]);
  const [docsKbId, setDocsKbId] = useState<string | null>(null);
  const [docsLoading, setDocsLoading] = useState(false);
  const [docsError, setDocsError] = useState<string | null>(null);

  const [uploading, setUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<string | null>(null);

  const [searchResults, setSearchResults] = useState<KbSearchResult[] | null>(null);
  const [searching, setSearching] = useState(false);

  const [deletingDoc, setDeletingDoc] = useState(false);

  const loadBases = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await loadKnowledgeBasesIntoGraph();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadBases();
  }, [loadBases]);

  const addBase = useCallback(
    async (form: { name: string; description: string }) => {
      setSaving(true);
      setError(null);
      try {
        await createKnowledgeBase(form);
        await loadKnowledgeBasesIntoGraph();
      } catch (e) {
        setError((e as Error).message);
        throw e;
      } finally {
        setSaving(false);
      }
    },
    [],
  );

  const removeBase = useCallback(async (id: string) => {
    setDeleting(true);
    setError(null);
    try {
      await optimisticRemove("KnowledgeBase", id, async () => {
        await deleteKnowledgeBase(id);
      });
    } catch (e) {
      setError(`Failed to remove knowledge base: ${(e as Error).message}`);
      throw e;
    } finally {
      setDeleting(false);
    }
  }, []);

  const loadDocs = useCallback(async (kbId: string) => {
    setDocsLoading(true);
    setDocsError(null);
    setDocsKbId(kbId);
    try {
      const list = await fetchDocuments(kbId);
      setDocs(list);
      // Also mirror into the graph so SSE deltas reconcile naturally.
      const { upsertEntity } = useGraphStore.getState();
      for (const d of list) {
        upsertEntity("Document", d.id, d as unknown as Record<string, unknown>);
      }
    } catch (e) {
      setDocsError((e as Error).message);
    } finally {
      setDocsLoading(false);
    }
  }, []);

  const uploadFiles = useCallback(
    async (kbId: string, files: FileList) => {
      setUploading(true);
      setUploadProgress(null);
      try {
        for (let i = 0; i < files.length; i += 1) {
          const f = files[i];
          setUploadProgress(`Uploading ${f.name} (${i + 1} of ${files.length})…`);
          await uploadDocument(kbId, f);
        }
        await loadDocs(kbId);
      } catch (e) {
        setDocsError((e as Error).message);
        throw e;
      } finally {
        setUploading(false);
        setUploadProgress(null);
      }
    },
    [loadDocs],
  );

  const removeDocument = useCallback(
    async (kbId: string, doc: UarKnowledgeDocument) => {
      setDeletingDoc(true);
      setDocsError(null);
      // Optimistic: hide locally first, restore on failure.
      const snapshot = docs;
      setDocs((prev) => prev.filter((d) => d.id !== doc.id));
      try {
        await optimisticRemove("Document", doc.id, async () => {
          await deleteDocument(kbId, doc.id);
        });
      } catch (e) {
        setDocs(snapshot);
        setDocsError((e as Error).message);
        throw e;
      } finally {
        setDeletingDoc(false);
      }
    },
    [docs],
  );

  const runSearch = useCallback(async (kbId: string, query: string) => {
    setSearching(true);
    setDocsError(null);
    try {
      const results = await searchKnowledgeBase(kbId, query);
      setSearchResults(results);
    } catch (e) {
      setDocsError((e as Error).message);
    } finally {
      setSearching(false);
    }
  }, []);

  const clearSearch = useCallback(() => setSearchResults(null), []);
  const clearDocView = useCallback(() => {
    setDocs([]);
    setDocsKbId(null);
    setDocsError(null);
    setSearchResults(null);
  }, []);

  // Silence the unused warning for optimisticUpsert (kept as a hook
  // re-export hint for future doc-edit migrations).
  void optimisticUpsert;

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
