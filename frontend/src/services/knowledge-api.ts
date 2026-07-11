import type {
  KbSearchResult,
  UarKnowledgeBase,
  UarKnowledgeDocument,
} from "@/types";

type KbListResponse =
  | UarKnowledgeBase[]
  | {
      knowledge_bases?: UarKnowledgeBase[];
      data?: { knowledge_bases?: UarKnowledgeBase[] };
    };

export function parseKbList(data: KbListResponse): UarKnowledgeBase[] {
  if (Array.isArray(data)) return data;
  return (
    data.data?.knowledge_bases ??
    (data as { knowledge_bases?: UarKnowledgeBase[] }).knowledge_bases ??
    []
  );
}

type DocListResponse =
  | UarKnowledgeDocument[]
  | {
      documents?: UarKnowledgeDocument[];
      data?: { documents?: UarKnowledgeDocument[] };
    };

export function parseDocList(data: DocListResponse): UarKnowledgeDocument[] {
  if (Array.isArray(data)) return data;
  return (
    data.data?.documents ??
    (data as { documents?: UarKnowledgeDocument[] }).documents ??
    []
  );
}

export async function fetchKnowledgeBases(): Promise<UarKnowledgeBase[]> {
  const res = await fetch("/api/knowledge");
  if (!res.ok) throw new Error(`${res.status}`);
  const data = (await res.json()) as KbListResponse;
  return parseKbList(data);
}

export async function createKnowledgeBase(body: {
  name: string;
  description: string;
}): Promise<UarKnowledgeBase> {
  const res = await fetch("/api/knowledge", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<UarKnowledgeBase>;
}

export async function deleteKnowledgeBase(id: string): Promise<void> {
  const res = await fetch(`/api/knowledge/${encodeURIComponent(id)}`, { method: "DELETE" });
  if (!res.ok) throw new Error(`${res.status}`);
}

export async function fetchDocuments(kbId: string): Promise<UarKnowledgeDocument[]> {
  const res = await fetch(
    `/api/knowledge/${encodeURIComponent(kbId)}/documents`,
  );
  if (!res.ok) throw new Error(`${res.status}`);
  const data = (await res.json()) as DocListResponse;
  return parseDocList(data);
}

export async function uploadDocument(
  kbId: string,
  file: File,
): Promise<UarKnowledgeDocument> {
  const fd = new FormData();
  fd.append("file", file);
  const res = await fetch(
    `/api/knowledge/${encodeURIComponent(kbId)}/documents`,
    { method: "POST", body: fd },
  );
  if (!res.ok) throw new Error(`Upload failed: ${res.status}`);
  return res.json() as Promise<UarKnowledgeDocument>;
}

export async function deleteDocument(kbId: string, docId: string): Promise<void> {
  const res = await fetch(
    `/api/knowledge/${encodeURIComponent(kbId)}/documents/${encodeURIComponent(docId)}`,
    { method: "DELETE" },
  );
  if (!res.ok) throw new Error(`${res.status}`);
}

export async function searchKnowledgeBase(
  kbId: string,
  query: string,
  limit = 10,
): Promise<KbSearchResult[]> {
  const res = await fetch(
    `/api/knowledge/${encodeURIComponent(kbId)}/search`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ query, limit }),
    },
  );
  if (!res.ok) throw new Error(`${res.status}`);
  const data = (await res.json()) as { results?: KbSearchResult[] };
  return data.results ?? [];
}
