-- Realtime change-notification triggers for the Postgres LISTEN/NOTIFY bus.
--
-- The UAR `PostgresNotifyBus` (`src/uar/realtime/postgres_bus.rs`) opens a single
-- `LISTEN uar_entity_change` connection and fans notifications out to the SSE
-- endpoint `GET /api/live/{topic}`. These triggers emit one thin notification per
-- row change so the SPA can refetch the affected entity.
--
-- Payload shape (consumed by `NotifyPayload` in postgres_bus.rs):
--   { "topic": "<entity_topic>", "id": "<row id>", "action": "insert|update|delete" }
--
-- The payload deliberately carries NO row data. The client refetches on signal,
-- which keeps every notification well under Postgres' 8000-byte NOTIFY limit.

-- ---------------------------------------------------------------------------
-- Generic trigger: topic is supplied as the first trigger argument (TG_ARGV[0]).
-- Used by every entity table whose live topic is fixed.
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION uar_notify_entity_change() RETURNS trigger AS $$
DECLARE
    rec     RECORD;
    row_id  TEXT;
    topic   TEXT := TG_ARGV[0];
BEGIN
    IF (TG_OP = 'DELETE') THEN
        rec := OLD;
    ELSE
        rec := NEW;
    END IF;
    row_id := rec.id::text;

    PERFORM pg_notify(
        'uar_entity_change',
        json_build_object(
            'topic',  topic,
            'id',     row_id,
            'action', lower(TG_OP)
        )::text
    );

    IF (TG_OP = 'DELETE') THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ---------------------------------------------------------------------------
-- Settings trigger: the `settings` table backs three live topics. Derive the
-- topic from the dotted key prefix ("provider.*" -> providers, "model.*" ->
-- models, everything else -> settings) so provider/model views refresh
-- independently.
-- ---------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION uar_notify_settings_change() RETURNS trigger AS $$
DECLARE
    rec     RECORD;
    row_key TEXT;
    topic   TEXT;
BEGIN
    IF (TG_OP = 'DELETE') THEN
        rec := OLD;
    ELSE
        rec := NEW;
    END IF;
    row_key := rec.key;

    IF row_key LIKE 'provider.%' THEN
        topic := 'providers';
    ELSIF row_key LIKE 'model.%' THEN
        topic := 'models';
    ELSE
        topic := 'settings';
    END IF;

    PERFORM pg_notify(
        'uar_entity_change',
        json_build_object(
            'topic',  topic,
            'id',     rec.id::text,
            'action', lower(TG_OP)
        )::text
    );

    IF (TG_OP = 'DELETE') THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ---------------------------------------------------------------------------
-- Attach triggers. Only tables that exist in this schema get a trigger; the
-- topic string passed to the generic function matches `EntityTopic::from_str`.
-- ---------------------------------------------------------------------------

DROP TRIGGER IF EXISTS trg_notify_agents ON agents;
CREATE TRIGGER trg_notify_agents
    AFTER INSERT OR UPDATE OR DELETE ON agents
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('agents');

DROP TRIGGER IF EXISTS trg_notify_sessions ON sessions;
CREATE TRIGGER trg_notify_sessions
    AFTER INSERT OR UPDATE OR DELETE ON sessions
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('sessions');

DROP TRIGGER IF EXISTS trg_notify_skills ON skills;
CREATE TRIGGER trg_notify_skills
    AFTER INSERT OR UPDATE OR DELETE ON skills
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('skills');

DROP TRIGGER IF EXISTS trg_notify_knowledge_bases ON knowledge_bases;
CREATE TRIGGER trg_notify_knowledge_bases
    AFTER INSERT OR UPDATE OR DELETE ON knowledge_bases
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('knowledge_bases');

DROP TRIGGER IF EXISTS trg_notify_knowledge_documents ON knowledge_documents;
CREATE TRIGGER trg_notify_knowledge_documents
    AFTER INSERT OR UPDATE OR DELETE ON knowledge_documents
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('knowledge_documents');

-- EntityTopic::Memory -> Postgres `memories` table.
DROP TRIGGER IF EXISTS trg_notify_memories ON memories;
CREATE TRIGGER trg_notify_memories
    AFTER INSERT OR UPDATE OR DELETE ON memories
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('memory');

-- EntityTopic::CompilerSessions -> Postgres `uar_compiler_sessions` table.
DROP TRIGGER IF EXISTS trg_notify_compiler_sessions ON uar_compiler_sessions;
CREATE TRIGGER trg_notify_compiler_sessions
    AFTER INSERT OR UPDATE OR DELETE ON uar_compiler_sessions
    FOR EACH ROW EXECUTE FUNCTION uar_notify_entity_change('compiler_sessions');

-- Settings backs settings + providers + models (topic derived from key prefix).
DROP TRIGGER IF EXISTS trg_notify_settings ON settings;
CREATE TRIGGER trg_notify_settings
    AFTER INSERT OR UPDATE OR DELETE ON settings
    FOR EACH ROW EXECUTE FUNCTION uar_notify_settings_change();
