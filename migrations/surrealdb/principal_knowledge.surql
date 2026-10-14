-- No adoption or rename of legacy rows, even when owner_id resembles v1:k.
DEFINE TABLE IF NOT EXISTS principal_knowledge_bases SCHEMALESS;
DEFINE TABLE IF NOT EXISTS principal_knowledge_documents SCHEMALESS;
DEFINE TABLE IF NOT EXISTS principal_knowledge_chunks SCHEMALESS;
DEFINE TABLE IF NOT EXISTS principal_knowledge_ingestion SCHEMALESS;
DEFINE INDEX IF NOT EXISTS principal_knowledge_base_name ON principal_knowledge_bases FIELDS owner_id, name UNIQUE;
DEFINE INDEX IF NOT EXISTS principal_knowledge_document_kb ON principal_knowledge_documents FIELDS owner_id, kb_id;
DEFINE INDEX IF NOT EXISTS principal_knowledge_chunk_kb ON principal_knowledge_chunks FIELDS owner_id, kb_id;
DEFINE INDEX IF NOT EXISTS principal_knowledge_ingestion_discovery ON principal_knowledge_ingestion FIELDS state, next_attempt_at;
