-- Enable required PostgreSQL extensions.
-- pgvector is used for semantic search / embeddings throughout UAR.
-- pgmq is enabled here for future queue-based workloads; no migrations
-- currently reference it but it must be available at init time.
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pgmq;
