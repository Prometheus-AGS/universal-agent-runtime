-- KBs can independently regenerate into different model dimensions. A fixed
-- typemod/global HNSW index cannot represent that contract. Retain owner/KB
-- btree indexes and use identity/dimension-fenced exact search for correctness.
-- https://github.com/pgvector/pgvector#can-i-store-vectors-with-different-dimensions-in-the-same-column
DROP INDEX IF EXISTS chunks_embedding_idx;
DROP INDEX IF EXISTS principal_knowledge_chunks_embedding;
ALTER TABLE knowledge_chunks ALTER COLUMN embedding TYPE vector;
ALTER TABLE principal_knowledge_chunks ALTER COLUMN embedding TYPE vector;
