DEFINE TABLE IF NOT EXISTS knowledge_ingestion SCHEMALESS;
DEFINE INDEX IF NOT EXISTS knowledge_ingestion_discovery ON knowledge_ingestion FIELDS state, next_attempt_at;
DEFINE INDEX IF NOT EXISTS knowledge_ingestion_owner_kb ON knowledge_ingestion FIELDS owner_id, kb_id;
-- Fresh stores must support discovery before their first upload. Existing schemas survive.
DEFINE TABLE IF NOT EXISTS knowledge_bases SCHEMALESS;
DEFINE TABLE IF NOT EXISTS knowledge_documents SCHEMALESS;
DEFINE TABLE IF NOT EXISTS knowledge_chunks SCHEMALESS;
BEGIN TRANSACTION;
FOR $document IN (SELECT * FROM knowledge_documents WHERE status.status IN ['pending','processing'] OR status IN ['pending','processing']) {
    LET $job = (SELECT * FROM knowledge_ingestion WHERE owner_id=$document.owner_id AND document_id=$document.logical_id)[0];
    IF $job = NONE {
        UPDATE $document.id SET status={status:'failed',error:'source_bytes_missing: reupload required'},updated_at=<string>time::now();
    };
};
COMMIT TRANSACTION;
