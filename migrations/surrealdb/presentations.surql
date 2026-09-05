-- Applied at connection time so an empty catalog can be listed on first use.
DEFINE TABLE IF NOT EXISTS presentations SCHEMALESS;
DEFINE INDEX IF NOT EXISTS presentation_owner_id ON presentations FIELDS owner_id, presentation_id UNIQUE;
