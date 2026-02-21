# Custom PostgreSQL 17 image with pgvector and pgmq extensions.
# All UAR migrations are baked in via docker-entrypoint-initdb.d and
# run automatically on first container start.
#
# Build:
#   docker build -t tribehealth/uar-postgres:pg17 k8s/postgres/
# Push:
#   docker push tribehealth/uar-postgres:pg17

FROM pgvector/pgvector:pg17

# Install pgmq from the Tembo Fury apt repository.
# pgvector is already present in the base image.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        curl \
        ca-certificates \
        gnupg \
    && curl -fsSL https://apt.fury.io/tembo/gpg.key \
        | gpg --dearmor -o /usr/share/keyrings/tembo.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/tembo.gpg trusted=yes] https://apt.fury.io/tembo/ /" \
        > /etc/apt/sources.list.d/tembo.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends postgresql-17-pgmq \
    && apt-get purge -y --auto-remove curl gnupg \
    && rm -rf /var/lib/apt/lists/*

# Copy ordered init scripts.
# The Postgres entrypoint runs files in /docker-entrypoint-initdb.d
# in lexicographic order on the very first start (when PGDATA is empty).
COPY initdb/ /docker-entrypoint-initdb.d/
