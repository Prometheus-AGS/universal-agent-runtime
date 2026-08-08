import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { useGraphStore } from "@/platform/entities";

import { useKnowledgePage } from "./use-knowledge-page";
import {
  createKnowledgeBase,
  deleteDocument,
  deleteKnowledgeBase,
  fetchDocuments,
  fetchKnowledgeBases,
  searchKnowledgeBase,
  uploadDocument,
} from "../api/knowledge-api";
import { useKnowledgeStore } from "./knowledge-store";

vi.mock("../api/knowledge-api", () => ({
  createKnowledgeBase: vi.fn(),
  deleteDocument: vi.fn(),
  deleteKnowledgeBase: vi.fn(),
  fetchDocuments: vi.fn(),
  fetchKnowledgeBases: vi.fn(),
  searchKnowledgeBase: vi.fn(),
  uploadDocument: vi.fn(),
}));

const base = {
  id: "kb-1",
  name: "Release facts",
  description: "Certified facts",
  document_count: 1,
  created_at: "2026-07-11T00:00:00Z",
  updated_at: "2026-07-11T00:00:00Z",
};

const document = {
  id: "doc-1",
  kb_id: base.id,
  filename: "facts.txt",
  chunk_count: 1,
  status: "indexed" as const,
  created_at: "2026-07-11T00:00:00Z",
  updated_at: "2026-07-11T00:00:00Z",
};

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(fetchKnowledgeBases).mockResolvedValue([]);
  useGraphStore.setState({ entities: {}, lists: {} });
  useKnowledgeStore.setState({
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
  });
});

describe("knowledge store", () => {
  test("exposes loading and reconciles an empty base response", async () => {
    let resolve!: (value: typeof base[]) => void;
    vi.mocked(fetchKnowledgeBases).mockImplementation(
      () => new Promise((done) => { resolve = done; }),
    );

    const pending = useKnowledgeStore.getState().loadBases();
    expect(useKnowledgeStore.getState().loading).toBe(true);
    resolve([]);
    await pending;

    expect(useKnowledgeStore.getState()).toMatchObject({ loading: false, error: null });
    expect(useGraphStore.getState().entities.KnowledgeBase).toBeUndefined();
  });

  test("surfaces authorization and generic load failures", async () => {
    vi.mocked(fetchKnowledgeBases).mockRejectedValue(new Error("401"));
    await expect(useKnowledgeStore.getState().loadBases()).rejects.toThrow("401");
    expect(useKnowledgeStore.getState()).toMatchObject({ loading: false, error: "401" });
  });

  test("creates a base and reconciles the authoritative list", async () => {
    vi.mocked(createKnowledgeBase).mockResolvedValue(base);
    vi.mocked(fetchKnowledgeBases).mockResolvedValue([base]);

    await useKnowledgeStore.getState().addBase({ name: base.name, description: base.description });

    expect(createKnowledgeBase).toHaveBeenCalledWith({
      name: base.name,
      description: base.description,
    });
    expect(useGraphStore.getState().entities.KnowledgeBase?.[base.id]).toMatchObject(base);
  });

  test("rolls a knowledge-base deletion back when persistence rejects", async () => {
    useGraphStore.getState().upsertEntity("KnowledgeBase", base.id, base);
    vi.mocked(deleteKnowledgeBase).mockRejectedValue(new Error("delete denied"));

    await expect(useKnowledgeStore.getState().removeBase(base.id)).rejects.toThrow(
      "delete denied",
    );
    expect(useGraphStore.getState().entities.KnowledgeBase?.[base.id]).toMatchObject(base);
  });

  test("loads documents, uploads files, and returns ranked search results", async () => {
    const pending = { ...document, id: "doc-2", status: "pending" as const };
    vi.mocked(fetchDocuments)
      .mockResolvedValueOnce([document])
      .mockResolvedValueOnce([document, pending]);
    vi.mocked(uploadDocument).mockResolvedValue(pending);
    vi.mocked(searchKnowledgeBase).mockResolvedValue([
      { content: "The launch code is ORANGE-42.", score: 0.91, document_id: document.id },
    ]);

    await useKnowledgeStore.getState().loadDocs(base.id);
    await useKnowledgeStore.getState().uploadFiles(base.id, [new File(["fact"], "facts.txt")]);
    await useKnowledgeStore.getState().runSearch(base.id, "launch code");

    expect(useKnowledgeStore.getState().docsByKb[base.id]).toHaveLength(2);
    expect(useKnowledgeStore.getState().searchResults?.[0]).toMatchObject({ score: 0.91 });
  });

  test("rolls a document deletion back in both store and graph", async () => {
    useKnowledgeStore.setState({ docsByKb: { [base.id]: [document] } });
    useGraphStore.getState().upsertEntity("Document", document.id, document);
    vi.mocked(deleteDocument).mockRejectedValue(new Error("document retained"));

    await expect(
      useKnowledgeStore.getState().removeDocument(base.id, document),
    ).rejects.toThrow("document retained");
    expect(useKnowledgeStore.getState().docsByKb[base.id]).toEqual([document]);
    expect(useGraphStore.getState().entities.Document?.[document.id]).toMatchObject(document);
  });

  test("surfaces an invalid upload without marking a document indexed", async () => {
    vi.mocked(uploadDocument).mockRejectedValue(new Error("Upload failed: 415"));

    await expect(
      useKnowledgeStore.getState().uploadFiles(base.id, [new File(["bad"], "bad.exe")]),
    ).rejects.toThrow("Upload failed: 415");
    expect(useKnowledgeStore.getState()).toMatchObject({
      uploading: false,
      docsError: "Upload failed: 415",
    });
    expect(useKnowledgeStore.getState().docsByKb[base.id] ?? []).toHaveLength(0);
  });

  test("retries a failed indexed document with the retained source bytes", async () => {
    const failed = { ...document, status: "failed" as const, error_message: "extract failed" };
    const retried = { ...document, id: "doc-retried", status: "pending" as const };
    vi.mocked(uploadDocument)
      .mockResolvedValueOnce(failed)
      .mockResolvedValueOnce(retried);
    vi.mocked(fetchDocuments)
      .mockResolvedValueOnce([failed])
      .mockResolvedValueOnce([retried]);
    vi.mocked(deleteDocument).mockResolvedValue();

    const source = new File(["corrected facts"], failed.filename, { type: "text/plain" });
    await useKnowledgeStore.getState().uploadFiles(base.id, [source]);
    await useKnowledgeStore.getState().retryDocument(base.id, failed);

    expect(deleteDocument).toHaveBeenCalledWith(base.id, failed.id);
    expect(uploadDocument).toHaveBeenLastCalledWith(base.id, source);
    expect(useKnowledgeStore.getState().docsByKb[base.id]).toEqual([retried]);
    expect(useKnowledgeStore.getState()).toMatchObject({ retryingDocId: null, docsError: null });
  });

  test("projects realtime graph reconciliation through the hook", async () => {
    vi.mocked(fetchKnowledgeBases).mockResolvedValue([base]);
    vi.mocked(fetchDocuments).mockResolvedValue([document]);
    const { result } = renderHook(() => useKnowledgePage());

    await act(async () => {
      await result.current.loadDocs(base.id);
    });
    expect(result.current.docs).toHaveLength(1);

    const realtimeDocument = { ...document, id: "doc-realtime", filename: "live.txt" };
    act(() => {
      useGraphStore.getState().upsertEntity("Document", realtimeDocument.id, realtimeDocument);
    });
    await waitFor(() => expect(result.current.docs).toHaveLength(2));
  });
});
