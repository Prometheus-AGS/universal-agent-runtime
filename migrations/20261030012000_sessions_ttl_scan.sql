-- Session TTL scan support (issue #390).
--
-- `sessions.data->>'last_activity'` is application-written text. A direct
-- `::timestamptz` cast makes one malformed row fail the whole TTL query. This
-- helper returns NULL for unreadable values so the scan can filter expired rows
-- in SQL and report malformed rows instead of aborting or loading every row.
CREATE OR REPLACE FUNCTION _uar_try_timestamptz(value TEXT)
RETURNS TIMESTAMPTZ AS $$
BEGIN
    RETURN value::timestamptz;
EXCEPTION WHEN others THEN
    RETURN NULL;
END;
$$ LANGUAGE plpgsql STABLE;
