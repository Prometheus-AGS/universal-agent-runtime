-- Rename skills primary key column 'id' → 'skill_id' to match Rust code.
-- The HNSW index must be dropped and recreated because indexes reference
-- column names directly.
DROP INDEX IF EXISTS skills_embedding_idx;

ALTER TABLE skills RENAME COLUMN id TO skill_id;

CREATE INDEX IF NOT EXISTS skills_embedding_idx
    ON skills USING hnsw (embedding vector_cosine_ops);
