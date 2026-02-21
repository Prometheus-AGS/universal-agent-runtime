# Custom PostgreSQL 17 image with pgvector and pgmq extensions.
# All UAR migrations are baked in via docker-entrypoint-initdb.d and
# run automatically on first container start.
#
# Build:
#   docker build -t tribehealth/uar-postgres:20212026 k8s/postgres/
# Push:
#   docker push tribehealth/uar-postgres:20212026

FROM pgvector/pgvector:pg17

# Build and install pgmq from the official GitHub release.
# pgmq-extension is a pure PGXS (SQL + server-side plpgsql) extension —
# no Rust/pgrx toolchain required.
ARG PGMQ_VERSION=1.11.0

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    curl \
    postgresql-server-dev-17 \
    && curl -fsSL \
    "https://github.com/tembo-io/pgmq/archive/refs/tags/v${PGMQ_VERSION}.tar.gz" \
    -o /tmp/pgmq.tar.gz \
    && tar -xzf /tmp/pgmq.tar.gz -C /tmp \
    && cd "/tmp/pgmq-${PGMQ_VERSION}/pgmq-extension" \
    && make \
    && make install \
    && apt-get purge -y --auto-remove \
    build-essential \
    postgresql-server-dev-17 \
    && rm -rf /tmp/pgmq* /var/lib/apt/lists/*

# Copy ordered init scripts.
# The Postgres entrypoint runs files in /docker-entrypoint-initdb.d
# in lexicographic order on the very first start (when PGDATA is empty).
COPY initdb/ /docker-entrypoint-initdb.d/
