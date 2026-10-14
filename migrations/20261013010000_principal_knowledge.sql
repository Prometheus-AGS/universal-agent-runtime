-- Never copy legacy rows based on owner text: a subject may literally be a
-- principal storage key. Authenticated knowledge starts in disjoint tables.
CREATE TABLE principal_knowledge_bases (
    owner_id TEXT NOT NULL,
    id TEXT NOT NULL,
    name TEXT,
    description TEXT,
    config JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (owner_id, id),
    UNIQUE (owner_id, name)
);
CREATE TABLE principal_knowledge_documents (
    owner_id TEXT NOT NULL,
    id TEXT NOT NULL,
    kb_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    file_path TEXT,
    mime_type TEXT NOT NULL,
    chunk_count INTEGER DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (owner_id, id),
    FOREIGN KEY (owner_id, kb_id) REFERENCES principal_knowledge_bases(owner_id, id) ON DELETE CASCADE
);
CREATE TABLE principal_knowledge_chunks (
    owner_id TEXT NOT NULL,
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    kb_id TEXT NOT NULL,
    document_id TEXT,
    content TEXT NOT NULL,
    metadata JSONB,
    embedding VECTOR(384),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (owner_id, id),
    FOREIGN KEY (owner_id, kb_id) REFERENCES principal_knowledge_bases(owner_id, id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id, document_id) REFERENCES principal_knowledge_documents(owner_id, id) ON DELETE CASCADE
);
CREATE TABLE principal_knowledge_ingestion (
    owner_id TEXT NOT NULL,
    document_id TEXT NOT NULL,
    kb_id TEXT NOT NULL,
    state TEXT NOT NULL,
    next_attempt_at BIGINT NOT NULL,
    data JSONB NOT NULL,
    source BYTEA NOT NULL,
    PRIMARY KEY (owner_id, document_id),
    FOREIGN KEY (owner_id, document_id) REFERENCES principal_knowledge_documents(owner_id, id) ON DELETE CASCADE,
    FOREIGN KEY (owner_id, kb_id) REFERENCES principal_knowledge_bases(owner_id, id) ON DELETE CASCADE
);
CREATE INDEX principal_knowledge_documents_owner_kb ON principal_knowledge_documents(owner_id, kb_id);
CREATE INDEX principal_knowledge_chunks_owner_kb ON principal_knowledge_chunks(owner_id, kb_id);
CREATE INDEX principal_knowledge_chunks_embedding ON principal_knowledge_chunks USING hnsw (embedding vector_cosine_ops);
CREATE INDEX principal_knowledge_ingestion_discovery ON principal_knowledge_ingestion(state, next_attempt_at);
