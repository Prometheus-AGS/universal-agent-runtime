# API Key Authentication

The UAR uses a **Personal Access Token (PAT)** system for API authentication. Tokens are exchanged for short-lived JWTs for request authorization.

## Flow

```
Client                          UAR Auth API
  │                                  │
  │── POST /api/auth/keys ──────────▶│  (create PAT)
  │◀── { key_id, token } ────────────│
  │                                  │
  │── POST /api/auth/exchange ───────▶│  (exchange PAT → JWT)
  │   { token: "<pat>" }             │
  │◀── { access_token, expires_in } ─│
  │                                  │
  │── GET /api/uar/specs ────────────▶│  (use JWT)
  │   Authorization: Bearer <jwt>    │
  │◀── [ ... ] ──────────────────────│
```

## Creating a PAT

```bash
curl -X POST http://localhost:3928/api/auth/keys \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-ci-token",
    "description": "Used in CI pipeline"
  }'
```

Response:
```json
{
  "key_id": "key_abc123",
  "token": "uar_pat_xxxxxxxxxxxxxxxxxxxx",
  "name": "my-ci-token",
  "created_at": "2026-02-18T09:00:00Z"
}
```

> **Important:** The `token` value is only shown once. Store it securely.

## Exchanging for a JWT

```bash
curl -X POST http://localhost:3928/api/auth/exchange \
  -H "Content-Type: application/json" \
  -d '{ "token": "uar_pat_xxxxxxxxxxxxxxxxxxxx" }'
```

Response:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 3600
}
```

## Using the JWT

Include the JWT in the `Authorization` header for all API requests:

```bash
curl http://localhost:3928/api/uar/specs \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## Listing Keys

```bash
curl http://localhost:3928/api/auth/keys \
  -H "Authorization: Bearer <jwt>"
```

## Revoking a Key

```bash
curl -X DELETE http://localhost:3928/api/auth/keys/<key_id> \
  -H "Authorization: Bearer <jwt>"
```

## Configuration

```toml
[security]
# Set to true to require JWT for all requests
require_auth = true

# JWT signing secret (generate with: openssl rand -hex 32)
jwt_secret = "your-secret-here"

# JWT expiry in seconds (default: 3600)
jwt_expiry_secs = 3600
```

To disable authentication for local development:

```toml
[security]
require_auth = false
```
