-- Settings provenance (issue #389).
--
-- SettingsManager reconstructs API ownership on restart from whether a stored
-- setting carries an `updated_at`. The original trigger stamped `updated_at`
-- on every UPDATE, including config-driven reconciliation writes, so a value
-- seeded or refreshed from config on boot N looked API-owned on boot N+1.
-- SurrealDB stores the caller's `updated_at` verbatim and never had the bug.
--
-- `source` records who wrote the row: 'config' (bootstrap/config reconciliation,
-- caller supplied no `updated_at`) or 'api' (runtime mutation, caller supplied
-- `updated_at`). The trigger now only stamps `updated_at` for 'api' rows.
ALTER TABLE settings
    ADD COLUMN IF NOT EXISTS source TEXT NOT NULL DEFAULT 'config';

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conrelid = 'settings'::regclass AND conname = 'settings_source_check'
    ) THEN
        ALTER TABLE settings
            ADD CONSTRAINT settings_source_check CHECK (source IN ('config', 'api'));
    END IF;
END
$$;

-- Existing rows with a non-null `updated_at` were treated as API-owned by the
-- manager. Keep that behaviour for data written before this migration.
-- The updated_at trigger is paused so the backfill does not restamp the
-- revision token of every existing row.
ALTER TABLE settings DISABLE TRIGGER trg_settings_updated;
UPDATE settings SET source = 'api' WHERE updated_at IS NOT NULL AND source = 'config';
ALTER TABLE settings ENABLE TRIGGER trg_settings_updated;

CREATE OR REPLACE FUNCTION _uar_settings_set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.source = 'api' THEN
        NEW.updated_at = now();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
