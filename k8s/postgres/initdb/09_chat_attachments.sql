-- Chat attachments: metadata for files uploaded during chat sessions.
-- File bytes live on disk at file_path; served via GET /api/attachments/{id}.
CREATE TABLE IF NOT EXISTS chat_attachments (
    id           TEXT        PRIMARY KEY,
    session_id   TEXT        NOT NULL,
    filename     TEXT        NOT NULL,
    content_type TEXT        NOT NULL,
    file_path    TEXT        NOT NULL,
    file_size    BIGINT      NOT NULL,
    is_image     BOOLEAN     NOT NULL DEFAULT FALSE,
    text_content TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chat_attachments_session
    ON chat_attachments (session_id);

CREATE INDEX IF NOT EXISTS idx_chat_attachments_created
    ON chat_attachments (created_at DESC);
