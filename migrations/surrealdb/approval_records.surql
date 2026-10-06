DEFINE TABLE IF NOT EXISTS approval_records SCHEMALESS;
DEFINE INDEX IF NOT EXISTS approval_records_owner_run ON approval_records FIELDS owner_key, root_run_id;
