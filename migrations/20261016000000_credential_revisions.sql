-- Credential revision metadata is intentionally nullable. Existing rows retain
-- their ciphertext, ownership, timestamps, and migration receipts until the
-- explicit trusted-host initialization operation assigns a revision.
ALTER TABLE provider_credentials
    ADD COLUMN IF NOT EXISTS row_revision TEXT;

ALTER TABLE provider_principal_credentials
    ADD COLUMN IF NOT EXISTS row_revision TEXT;

-- provider_credential_owner_migrations.receipt is existing JSONB. New receipt
-- projections may add source_kind, source_owner_key, source_revision,
-- destination_revision, and precondition without rewriting historical rows.
