-- Host-only wakeups. The durable rows are authoritative; connection loss and
-- channel lag trigger reconciliation. Unlike the legacy public notification
-- function, failure is not swallowed: a committed work mutation must not lose
-- its notification while the listener is healthy.
CREATE OR REPLACE FUNCTION uar_notify_lifecycle_work() RETURNS trigger AS $$
BEGIN
    PERFORM pg_notify('uar_entity_change', json_build_object(
        'topic', TG_ARGV[0], 'id', '*', 'action', lower(TG_OP)
    )::text);
    IF TG_OP = 'DELETE' THEN RETURN OLD; END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
DECLARE
    item TEXT[];
BEGIN
    FOREACH item SLICE 1 IN ARRAY ARRAY[
        ['uar_collaboration_state', 'runtime_collaboration_state'],
        ['observer_subscriptions', 'runtime_observer_subscriptions'],
        ['agent_instance_occurrences', 'runtime_agent_occurrences'],
        ['knowledge_ingestion', 'runtime_ingestion_jobs'],
        ['principal_knowledge_ingestion', 'runtime_principal_ingestion_jobs'],
        ['principal_knowledge_bases', 'runtime_principal_knowledge_bases'],
        ['principal_knowledge_documents', 'runtime_principal_knowledge_documents'],
        ['a2a_bindings', 'runtime_a2a_bindings'],
        ['agui_runs', 'runtime_agui_runs'],
        ['mcp_elicitations', 'runtime_mcp_elicitations']
    ] LOOP
        IF to_regclass(item[1]) IS NOT NULL THEN
            EXECUTE format('CREATE TRIGGER %I AFTER INSERT OR UPDATE OR DELETE ON %I '
                'FOR EACH ROW EXECUTE FUNCTION uar_notify_lifecycle_work(%L)',
                'trg_lifecycle_' || item[1], item[1], item[2]);
        END IF;
    END LOOP;
END;
$$;
