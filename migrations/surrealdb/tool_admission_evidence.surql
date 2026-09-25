DEFINE TABLE IF NOT EXISTS tool_admission_evidence SCHEMALESS;
DEFINE INDEX IF NOT EXISTS tool_admission_evidence_identity
    ON tool_admission_evidence FIELDS owner_id, invocation_id, state UNIQUE;
DEFINE INDEX IF NOT EXISTS tool_admission_evidence_owner_time
    ON tool_admission_evidence FIELDS owner_id, occurred_at, invocation_id;
