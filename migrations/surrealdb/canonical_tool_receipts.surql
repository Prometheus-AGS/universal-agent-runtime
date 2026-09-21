-- Applied by the provider at connection time, including embedded deployments.
-- The run envelope supplies one compare-and-set point for parallel admissions.
DEFINE TABLE IF NOT EXISTS canonical_tool_receipt_runs SCHEMALESS;
DEFINE INDEX IF NOT EXISTS canonical_receipt_run_owner_id
    ON canonical_tool_receipt_runs FIELDS owner_id, run_id UNIQUE;

DEFINE TABLE IF NOT EXISTS canonical_tool_receipts SCHEMALESS;
DEFINE INDEX IF NOT EXISTS canonical_receipt_owner_call
    ON canonical_tool_receipts FIELDS owner_id, run_id, call_id UNIQUE;
DEFINE INDEX IF NOT EXISTS canonical_receipt_owner_order
    ON canonical_tool_receipts FIELDS owner_id, run_id, sequence, call_id;
