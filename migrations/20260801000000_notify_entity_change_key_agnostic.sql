-- Make the entity-change notifier work on tables whose primary key is not `id`.
--
-- # The defect
--
-- `uar_notify_entity_change()` (migration 20260220000002) does:
--
--     row_id := rec.id::text;
--
-- `skills` has no `id` column — its key is `skill_id`. The trigger
-- `trg_notify_skills` therefore raised on EVERY insert:
--
--     ERROR:  record "rec" has no field "id"
--     CONTEXT:  PL/pgSQL assignment "row_id := rec.id::text"
--
-- Because the trigger fires AFTER INSERT, the whole statement aborted and the
-- row was never written. `SkillRegistry::register` logs a persist failure but
-- does not propagate it, so the skill silently vanished: 0 rows in Postgres
-- while memory and SurrealDB held all 3.
--
-- Found by change-uhe-008's postgres provider test, which only became runnable
-- once a PG18 + pgvector image replaced the unavailable PG16 extension. The
-- defect had been latent behind a BLOCKED test.
--
-- # The fix
--
-- Resolve the row key generically, preferring `id` and falling back to the
-- first column named `<something>_id`, then to the table's actual primary key.
-- `to_jsonb(rec)` lets us read a field whose name is not known at compile time,
-- which plain `rec.<field>` cannot do in PL/pgSQL.
--
-- A trigger that cannot identify a row must NOT abort the write. Notification
-- is a side channel; losing a notify is a degraded feature, losing the row is
-- data loss. The exception handler makes that trade explicit.

CREATE OR REPLACE FUNCTION uar_notify_entity_change() RETURNS trigger AS $$
DECLARE
    rec       RECORD;
    payload   JSONB;
    row_id    TEXT;
    topic     TEXT := TG_ARGV[0];
    pk_col    TEXT;
BEGIN
    IF (TG_OP = 'DELETE') THEN
        rec := OLD;
    ELSE
        rec := NEW;
    END IF;

    payload := to_jsonb(rec);

    -- 1. The common case.
    row_id := payload ->> 'id';

    -- 2. Tables keyed by `<entity>_id` (skills.skill_id, agents.agent_id, …).
    IF row_id IS NULL THEN
        SELECT key INTO pk_col
        FROM jsonb_object_keys(payload) AS key
        WHERE key LIKE '%\_id'
        ORDER BY key
        LIMIT 1;

        IF pk_col IS NOT NULL THEN
            row_id := payload ->> pk_col;
        END IF;
    END IF;

    -- 3. Ask the catalogue for the real primary key.
    IF row_id IS NULL THEN
        SELECT a.attname INTO pk_col
        FROM pg_index i
        JOIN pg_attribute a
          ON a.attrelid = i.indrelid
         AND a.attnum = ANY (i.indkey)
        WHERE i.indrelid = TG_RELID
          AND i.indisprimary
        LIMIT 1;

        IF pk_col IS NOT NULL THEN
            row_id := payload ->> pk_col;
        END IF;
    END IF;

    -- A notification we cannot address is not worth losing the write over.
    IF row_id IS NOT NULL THEN
        PERFORM pg_notify(
            'uar_entity_change',
            json_build_object(
                'topic',  topic,
                'id',     row_id,
                'action', lower(TG_OP)
            )::text
        );
    END IF;

    IF (TG_OP = 'DELETE') THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
EXCEPTION
    WHEN OTHERS THEN
        -- Never let the side channel take down the write. Surfaced as a
        -- WARNING so it is visible in logs rather than silently swallowed.
        RAISE WARNING 'uar_notify_entity_change(%) failed: %', topic, SQLERRM;
        IF (TG_OP = 'DELETE') THEN
            RETURN OLD;
        END IF;
        RETURN NEW;
END;
$$ LANGUAGE plpgsql;
