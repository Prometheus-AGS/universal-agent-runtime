# UAR Realtime — Standard Channel Taxonomy

_Last updated: 2026-02-21_

This document defines all first-party channels shipped with UAR. Plugin developers define channels in the `plugin:` namespace (see [integration.md](./integration.md)).

---

## Channel Naming Convention

```
{namespace}:{resource}[:{id}][:{subresource}]
```

- Segments are separated by `:` (colon)
- `{id}` is typically a UUID or slug
- Wildcards in subscriptions: `*` = one segment, `**` = any depth
- All channels are case-sensitive, lowercase

---

## 1. System Channels

### `system:notifications`

Global system-wide notifications. All authenticated users may subscribe (read-only server push).

| Event | Payload |
|---|---|
| `system:notification` | `{ level, title, body, action? }` |
| `system:maintenance:warning` | `{ starts_at, message, estimated_duration_minutes }` |
| `system:maintenance:started` | `{ message }` |
| `system:maintenance:ended` | `{ message }` |
| `system:version:updated` | `{ old_version, new_version, changelog_url }` |

### `system:metrics`

Real-time server metrics (admin only).

| Event | Payload |
|---|---|
| `system:metrics:snapshot` | `{ active_connections, active_runs, memory_mb, cpu_percent, timestamp }` |

---

## 2. Agent Run Channels

### `agent:run:{run_id}`

Per-run channel. Subscribe to track a single agent execution in real time.

| Event | Payload |
|---|---|
| `agent:run:started` | `{ run_id, agent_id, session_id, model, started_at }` |
| `agent:run:completed` | `{ run_id, finish_reason, total_tokens, duration_ms }` |
| `agent:run:failed` | `{ run_id, error, code }` |
| `agent:run:cancelled` | `{ run_id, reason }` |
| `agent:token:delta` | `{ delta, token_index, position }` |
| `agent:token:complete` | `{ full_text, total_tokens }` |
| `agent:tool:called` | `{ tool_name, tool_call_id, arguments, called_at }` |
| `agent:tool:result` | `{ tool_call_id, result, duration_ms }` |
| `agent:tool:error` | `{ tool_call_id, error, code }` |
| `agent:thinking:start` | `{ step }` |
| `agent:thinking:end` | `{ step, summary? }` |

**Access:** User must own the session associated with the run, or have `runs:read` permission.

---

## 3. Session Channels

### `session:{session_id}`

Per-session channel. Tracks conversation-level events.

| Event | Payload |
|---|---|
| `session:created` | `{ session_id, created_at, title }` |
| `session:updated` | `{ session_id, changes }` |
| `session:deleted` | `{ session_id }` |
| `session:message:created` | `{ message_id, role, created_at }` |
| `session:run:queued` | `{ run_id, queued_at }` |
| `session:run:started` | `{ run_id, started_at }` |
| `session:run:completed` | `{ run_id, completed_at }` |
| `session:attachment:uploaded` | `{ attachment_id, filename, mime_type, url }` |
| `session:attachment:deleted` | `{ attachment_id }` |
| `session:error` | `{ code, message }` |

**Access:** User must be the owner of the session, or admin.

---

## 4. User Channels

### `user:{user_id}:activity`

Personal channel for a single user. Useful for cross-device sync, notifications, and activity.

| Event | Payload |
|---|---|
| `user:session:created` | `{ session_id }` |
| `user:session:deleted` | `{ session_id }` |
| `user:notification` | `{ id, level, title, body, read, created_at }` |
| `user:api_key:created` | `{ key_id, name, created_at }` |
| `user:api_key:revoked` | `{ key_id }` |
| `user:settings:updated` | `{ changed_keys }` |

**Access:** Only the user themselves (or admins with `users:read`).

### `user:{user_id}:presence`

Reports the user's own cross-device presence.

| Event | Payload |
|---|---|
| `user:device:connected` | `{ device_id, client_type, connected_at }` |
| `user:device:disconnected` | `{ device_id, disconnected_at }` |

---

## 5. Collaborative Channels

### `session:{session_id}:collab`

Multi-user collaborative presence within a shared session. Presence is enabled on this channel by default.

| Event | Payload |
|---|---|
| `user:presence:sync` | Full presence state snapshot (see protocol.md §5) |
| `user:presence:joined` | `{ user_id, meta }` |
| `user:presence:left` | `{ user_id }` |
| `broadcast` | Ephemeral user-to-user messages (cursor, annotation, typing indicator) |

**Access:** Users with `sessions:collab` permission on this session.

---

## 6. Plugin Channels

### `plugin:{plugin_name}:{scope}`

Plugin-defined channels. The `{scope}` is plugin-defined and may be hierarchical using additional colons:

```
plugin:visual-canvas:board:brd_123
plugin:code-interpreter:execution:exec_456
plugin:workflow:run:wf_789
```

| Behavior | Detail |
|---|---|
| Namespace isolation | Plugins may only publish to their own `plugin:{name}:*` channels |
| Cross-plugin subscribe | Allowed if plugin declares `consumer_of: ["plugin:other:*"]` in its manifest |
| Custom events | Plugin defines its own event names; they must be prefixed with `plugin:{name}:` |
| Presence | Opt-in per channel via plugin manifest |

---

## 7. Channel Access Matrix

| Channel Pattern | Auth Required | Min Permission |
|---|---|---|
| `system:notifications` | Yes | `authenticated` |
| `system:metrics` | Yes | `admin` |
| `agent:run:{id}` | Yes | `runs:read` on that run |
| `session:{id}` | Yes | `sessions:read` on that session |
| `session:{id}:collab` | Yes | `sessions:collab` |
| `user:{id}:activity` | Yes | `self` or `users:read` (admin) |
| `user:{id}:presence` | Yes | `self` |
| `plugin:*` | Yes | Plugin-defined |

---

## 8. Wildcard Subscriptions (Internal / SDK Use)

Internal services (e.g. analytics, audit log) may subscribe to aggregate patterns. This is only available to server-side consumers:

```
agent:run:**          # All events for all runs
session:**            # All session events
user:**               # All user events
```

These patterns are not available to external WebSocket clients.
