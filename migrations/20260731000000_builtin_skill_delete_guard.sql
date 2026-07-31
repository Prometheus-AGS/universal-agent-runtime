-- Refuse deletion of builtin skills at the DATABASE, not just in the service.
--
-- WHY A TRIGGER AND NOT A COLUMN
--
-- The plan allowed for adding real `origin` / `enabled` columns. A probe against
-- Postgres 18.4 (change-uhe-006 task 1) showed they are unnecessary: a
-- BEFORE DELETE trigger reads `definition->>'origin'` directly, because
-- `postgres.rs:77` already serialises the whole `Skill` into `definition` JSONB.
-- Columns would have been a schema migration, a backfill, and a provider
-- round-trip change to reach the same guarantee.
--
-- WHY NOT A CHECK CONSTRAINT
--
-- Also probed, also wrong: a CHECK fires on INSERT and UPDATE, so
-- `CHECK (definition->>'origin' <> 'builtin')` blocks *loading* builtin skills —
-- the exact opposite of the requirement. Deletion needs a DELETE trigger.
--
-- WHY THE LITERAL IS LOWERCASE
--
-- `SkillOrigin` carries `#[serde(rename_all = "lowercase")]`, so the wire value
-- is `"builtin"`, NOT `"Builtin"`. A trigger written against the Rust variant's
-- spelling would never fire — and would read as protection while providing
-- none, which is worse than no trigger. Asserted by
-- `provenance::tests::builtin_origin_serialises_lowercase`, so a future rename
-- breaks a test rather than silently disarming this guard.
--
-- WHAT THIS DOES NOT DO
--
-- It does not prevent *disabling* a builtin skill. `enabled` stays freely
-- writable: the requirement is "can never be deleted, only turned off".

CREATE OR REPLACE FUNCTION refuse_builtin_skill_delete() RETURNS trigger AS $$
BEGIN
    IF OLD.definition->>'origin' = 'builtin' THEN
        RAISE EXCEPTION
            'system_skill_immutable: skill "%" ships with the pack and cannot be deleted; disable it instead',
            OLD.skill_id
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS skills_refuse_builtin_delete ON skills;

CREATE TRIGGER skills_refuse_builtin_delete
    BEFORE DELETE ON skills
    FOR EACH ROW
    EXECUTE FUNCTION refuse_builtin_skill_delete();

COMMENT ON FUNCTION refuse_builtin_skill_delete() IS
    'Refuses DELETE of skills whose definition->>''origin'' is ''builtin''. '
    'Guards the storage layer so a caller bypassing SkillService cannot remove '
    'a pack-shipped skill. Disabling remains permitted.';
