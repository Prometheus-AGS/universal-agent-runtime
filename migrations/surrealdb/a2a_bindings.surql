-- Transport correlation; owner_key includes verified issuer, tenant and subject.
DEFINE TABLE IF NOT EXISTS a2a_bindings SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS owner_key ON a2a_bindings TYPE string;
DEFINE FIELD IF NOT EXISTS agent_id ON a2a_bindings TYPE string;
DEFINE FIELD IF NOT EXISTS data ON a2a_bindings TYPE string;
DEFINE INDEX IF NOT EXISTS a2a_owner_agent ON a2a_bindings FIELDS owner_key, agent_id;
DEFINE FIELD IF NOT EXISTS context_closed ON a2a_bindings TYPE bool;
DEFINE TABLE IF NOT EXISTS a2a_contexts SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS binding_key ON a2a_contexts TYPE string;
