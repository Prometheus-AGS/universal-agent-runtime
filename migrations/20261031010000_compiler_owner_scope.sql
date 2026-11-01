-- Owner scoping for compiler specs, reports and authoring sessions (issue #387).
--
-- `owner` holds the canonical credential-principal storage key of the creator.
-- It is nullable on purpose: rows written before this migration keep NULL and
-- stay readable only by the installation-local `local-host` owner. No existing
-- row is rewritten or deleted.
ALTER TABLE uar_specs ADD COLUMN IF NOT EXISTS owner TEXT;
ALTER TABLE uar_reports ADD COLUMN IF NOT EXISTS owner TEXT;
ALTER TABLE uar_compiler_sessions ADD COLUMN IF NOT EXISTS owner TEXT;

CREATE INDEX IF NOT EXISTS idx_uar_specs_owner ON uar_specs(owner);
CREATE INDEX IF NOT EXISTS idx_uar_reports_owner ON uar_reports(owner);
CREATE INDEX IF NOT EXISTS idx_uar_compiler_sessions_owner ON uar_compiler_sessions(owner);
