#!/bin/bash
# First-boot init for the flint-forge PostgreSQL 18 image: give UAR its own
# role and database. The image's own init creates the flint extensions in
# POSTGRES_DB (`flint`) and pins pg_cron there, so UAR must never use `flint`.
#
# Mount as /docker-entrypoint-initdb.d/50-uar-database.sh and pass
# UAR_DB_PASSWORD in the container environment. Runs once, on an empty PGDATA.
set -euo pipefail

: "${UAR_DB_PASSWORD:?UAR_DB_PASSWORD must be set for the uar database role}"
UAR_DB_NAME="${UAR_DB_NAME:-uar}"
UAR_DB_USER="${UAR_DB_USER:-uar}"

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" \
  --set=uar_user="$UAR_DB_USER" --set=uar_db="$UAR_DB_NAME" --set=uar_password="$UAR_DB_PASSWORD" <<'SQL'
CREATE ROLE :"uar_user" LOGIN PASSWORD :'uar_password';
CREATE DATABASE :"uar_db" OWNER :"uar_user";
SQL

# pgvector is installed in the image; creating it needs a superuser, so the
# extension exists before UAR's migrations run as the uar role.
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$UAR_DB_NAME" <<'SQL'
CREATE EXTENSION IF NOT EXISTS vector;
SQL
