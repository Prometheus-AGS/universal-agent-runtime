-- Host-only execution-domain registrations and nonexpiring one-use challenges.
-- No lease TTL, cleanup job or role exposed to kernels may delete these rows.
DEFINE TABLE IF NOT EXISTS supervisor_domains SCHEMALESS;
DEFINE TABLE IF NOT EXISTS supervisor_challenges SCHEMALESS;
DEFINE TABLE IF NOT EXISTS supervisor_exclusions SCHEMALESS;
