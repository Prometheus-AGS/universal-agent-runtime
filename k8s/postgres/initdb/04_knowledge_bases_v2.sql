-- Knowledge Bases v2: named KBs and document-level tracking.

ALTER TABLE knowledge_bases ADD COLUMN IF NOT EXISTS name TEXT UNIQUE;
ALTER TABLE knowledge_bases ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE knowledge_bases ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

CREATE INDEX IF NOT EXISTS knowledge_bases_name_idx ON knowledge_bases(name);

CREATE TABLE IF NOT EXISTS knowledge_documents (
    id TEXT PRIMARY KEY,
    kb_id TEXT NOT NULL REFERENCES knowledge_bases(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    file_path TEXT,
    mime_type TEXT NOT NULL,
    chunk_count INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS knowledge_documents_kb_idx ON knowledge_documents(kb_id);

ALTER TABLE knowledge_chunks
    ADD COLUMN IF NOT EXISTS document_id TEXT REFERENCES knowledge_documents(id) ON DELETE SET NULL;
