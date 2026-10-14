-- Source and job share one row and are inserted with the document in one transaction.
CREATE TABLE IF NOT EXISTS knowledge_ingestion (
    owner_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    kb_id TEXT NOT NULL,
    state TEXT NOT NULL,
    next_attempt_at BIGINT NOT NULL,
    data JSONB NOT NULL,
    source BYTEA NOT NULL,
    PRIMARY KEY (owner_id, document_id),
    FOREIGN KEY (owner_id, document_id) REFERENCES knowledge_documents(owner_id, id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id, kb_id) REFERENCES knowledge_bases(owner_id, id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS knowledge_ingestion_discovery ON knowledge_ingestion(state, next_attempt_at);
-- Old accepted metadata has no recoverable source; do not imply it can resume.
UPDATE knowledge_documents d
SET status='failed', error_message='source_bytes_missing: reupload required', updated_at=NOW()
WHERE status IN ('pending','processing')
  AND NOT EXISTS (SELECT 1 FROM knowledge_ingestion j WHERE j.owner_id=d.owner_id AND j.document_id=d.id);
