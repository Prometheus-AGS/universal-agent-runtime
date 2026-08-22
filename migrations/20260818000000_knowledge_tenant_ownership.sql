ALTER TABLE knowledge_bases
    ADD COLUMN IF NOT EXISTS owner_id TEXT NOT NULL DEFAULT 'anonymous';
ALTER TABLE knowledge_documents
    ADD COLUMN IF NOT EXISTS owner_id TEXT NOT NULL DEFAULT 'anonymous';
ALTER TABLE knowledge_chunks
    ADD COLUMN IF NOT EXISTS owner_id TEXT NOT NULL DEFAULT 'anonymous';

UPDATE sessions
SET id = '9:anonymous:' || id,
    data = jsonb_set(data, '{owner_id}', '"anonymous"'::jsonb, true);

ALTER TABLE conversation_policies
    ADD COLUMN IF NOT EXISTS owner_id TEXT NOT NULL DEFAULT 'anonymous';
ALTER TABLE conversation_policies
    DROP CONSTRAINT IF EXISTS conversation_policies_pkey;
ALTER TABLE conversation_policies
    ADD PRIMARY KEY (owner_id, conversation_id);

ALTER TABLE knowledge_chunks
    DROP CONSTRAINT IF EXISTS knowledge_chunks_document_id_fkey;
ALTER TABLE knowledge_chunks
    DROP CONSTRAINT IF EXISTS knowledge_chunks_kb_id_fkey;
ALTER TABLE knowledge_documents
    DROP CONSTRAINT IF EXISTS knowledge_documents_kb_id_fkey;

ALTER TABLE knowledge_chunks DROP CONSTRAINT IF EXISTS knowledge_chunks_pkey;
ALTER TABLE knowledge_documents DROP CONSTRAINT IF EXISTS knowledge_documents_pkey;
ALTER TABLE knowledge_bases DROP CONSTRAINT IF EXISTS knowledge_bases_pkey;

ALTER TABLE knowledge_bases ADD PRIMARY KEY (owner_id, id);
ALTER TABLE knowledge_documents ADD PRIMARY KEY (owner_id, id);
ALTER TABLE knowledge_chunks ADD PRIMARY KEY (owner_id, id);

ALTER TABLE knowledge_documents
    ADD CONSTRAINT knowledge_documents_owner_kb_fkey
    FOREIGN KEY (owner_id, kb_id)
    REFERENCES knowledge_bases(owner_id, id)
    ON DELETE CASCADE;
ALTER TABLE knowledge_chunks
    ADD CONSTRAINT knowledge_chunks_owner_kb_fkey
    FOREIGN KEY (owner_id, kb_id)
    REFERENCES knowledge_bases(owner_id, id)
    ON DELETE CASCADE;
ALTER TABLE knowledge_chunks
    ADD CONSTRAINT knowledge_chunks_owner_document_fkey
    FOREIGN KEY (owner_id, document_id)
    REFERENCES knowledge_documents(owner_id, id)
    ON DELETE CASCADE;

ALTER TABLE knowledge_bases DROP CONSTRAINT IF EXISTS knowledge_bases_name_key;
DROP INDEX IF EXISTS knowledge_bases_name_idx;

CREATE UNIQUE INDEX IF NOT EXISTS knowledge_bases_owner_name_idx
    ON knowledge_bases(owner_id, name);
CREATE INDEX IF NOT EXISTS knowledge_documents_owner_kb_idx
    ON knowledge_documents(owner_id, kb_id);
CREATE INDEX IF NOT EXISTS knowledge_chunks_owner_kb_idx
    ON knowledge_chunks(owner_id, kb_id);
