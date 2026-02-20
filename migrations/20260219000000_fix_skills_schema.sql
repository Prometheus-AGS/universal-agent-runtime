-- Fix skills table: rename primary key column 'id' → 'skill_id' to match application code.
-- The Rust postgres provider uses 'skill_id' throughout, but the original migration
-- created the column as 'id'. This migration aligns the schema with the code.

-- Drop the HNSW index before renaming (indexes reference column names)
DROP INDEX IF EXISTS skills_embedding_idx;

-- Rename the primary key column
ALTER TABLE skills RENAME COLUMN id TO skill_id;

-- Recreate the HNSW vector index on the corrected column name
CREATE INDEX IF NOT EXISTS skills_embedding_idx
    ON skills USING hnsw (embedding vector_cosine_ops);
