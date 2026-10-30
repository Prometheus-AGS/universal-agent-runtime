-- Skill embeddings must follow the active embedding space (issue #395).
--
-- `skills.embedding` was VECTOR(384) with a global HNSW index, so any embedder
-- with another dimension made `save_skill` fail. Mirror the knowledge fix in
-- 20261025010000: an unconstrained `vector` column, no fixed-dimension index,
-- and an identity fence at query time.
-- https://github.com/pgvector/pgvector#can-i-store-vectors-with-different-dimensions-in-the-same-column
--
-- `embedding_fingerprint` records the EmbeddingFingerprint
-- ({"backend","model","dimension"}) the vector was produced in. Rows whose
-- fingerprint differs from the active embedder (including legacy rows, which
-- have none) are re-embedded by the runtime and are excluded from search until
-- then. A skill with no embedding keeps embedding and fingerprint NULL.
DROP INDEX IF EXISTS skills_embedding_idx;
ALTER TABLE skills ALTER COLUMN embedding TYPE vector;
ALTER TABLE skills ADD COLUMN IF NOT EXISTS embedding_fingerprint JSONB;
