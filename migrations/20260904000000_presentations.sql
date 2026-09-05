-- Owner-scoped reusable UI templates. Content is a complete revision snapshot.
CREATE TABLE IF NOT EXISTS presentations (
    owner_id TEXT NOT NULL,
    presentation_id TEXT NOT NULL,
    revision BIGINT NOT NULL CHECK (revision > 0),
    data JSONB NOT NULL,
    PRIMARY KEY (owner_id, presentation_id)
);
