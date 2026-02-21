# UAR Realtime — Integration Guide

_Last updated: 2026-02-21_

---

## 1. Frontend Integration (React / TypeScript)

### 1.1 Connecting to the Realtime Endpoint

Add the `UARRealtime` client (from `sdks/js/realtime/`) to your app. The recommended pattern is a singleton held in a Zustand/context store:

```typescript
// src/features/realtime/realtime-client.ts
import { UARRealtime } from '@uar/realtime';

let realtimeClient: UARRealtime | null = null;

export function getRealtimeClient(token: string): UARRealtime {
  if (!realtimeClient) {
    realtimeClient = new UARRealtime({
      url: import.meta.env.VITE_REALTIME_URL ?? `${window.location.origin.replace('http', 'ws')}/api/realtime`,
      token,
      heartbeatIntervalMs: 30_000,
      reconnectBackoffMs: [1000, 2000, 5000, 10000],
    });
  }
  return realtimeClient;
}
```

### 1.2 Tracking an Agent Run

```typescript
// src/features/chat/use-run-realtime.ts
import { useEffect, useRef } from 'react';
import { getRealtimeClient } from '@/features/realtime/realtime-client';
import { useAuthStore } from '@/stores/auth-store';
import { useChatStore } from '@/features/chat/chat-store';

export function useRunRealtime(runId: string | null) {
  const token = useAuthStore((s) => s.token);
  const appendToken = useChatStore((s) => s.appendToken);
  const setRunComplete = useChatStore((s) => s.setRunComplete);
  const channelRef = useRef<Channel | null>(null);

  useEffect(() => {
    if (!runId || !token) return;

    const rt = getRealtimeClient(token);
    const channel = rt.channel(`agent:run:${runId}`);

    channel
      .on('agent:token:delta', ({ delta }) => appendToken(delta))
      .on('agent:run:completed', (payload) => setRunComplete(runId, payload))
      .on('agent:run:failed', (payload) => setRunComplete(runId, { error: payload }))
      .subscribe();

    channelRef.current = channel;

    return () => {
      channel.unsubscribe();
    };
  }, [runId, token]);
}
```

### 1.3 Presence in a Shared Session

```typescript
// src/features/collaboration/use-session-presence.ts
import { useEffect, useState } from 'react';

export function useSessionPresence(sessionId: string) {
  const token = useAuthStore((s) => s.token);
  const [presentUsers, setPresentUsers] = useState<PresenceUser[]>([]);

  useEffect(() => {
    const rt = getRealtimeClient(token!);
    const channel = rt.channel(`session:${sessionId}:collab`, {
      presence: true,
      broadcastSelf: false,
    });

    channel
      .on('user:presence:sync', ({ presences }) => {
        setPresentUsers(Object.values(presences).flatMap((p) => p.metas));
      })
      .on('user:presence:joined', ({ user_id, meta }) => {
        setPresentUsers((prev) => [...prev, { user_id, ...meta }]);
      })
      .on('user:presence:left', ({ user_id }) => {
        setPresentUsers((prev) => prev.filter((u) => u.user_id !== user_id));
      })
      .subscribe(() => {
        // Track own presence
        channel.track({
          user_id: getCurrentUserId(),
          display_name: getCurrentUserName(),
          client_type: 'browser',
        });
      });

    return () => channel.unsubscribe();
  }, [sessionId, token]);

  return { presentUsers };
}
```

---

## 2. Backend Integration (Rust / Axum)

The UAR Rust server needs to:
1. Accept WebSocket connections at `/api/realtime`
2. Broker messages through an internal `RealtimeBroker`
3. Emit events from any subsystem by posting to the broker

### 2.1 Publishing Events from Any Rust Module

Any module with access to `AppState` can broadcast to a channel:

```rust
// Publishing from a run handler in server.rs (future implementation)
app_state.realtime
    .publish("agent:run:abc123", "agent:token:delta", json!({
        "delta": token_text,
        "token_index": index,
    }))
    .await?;
```

### 2.2 Internal Broker Architecture (Planned)

```
┌─────────────────────────────────────────────────────────────────┐
│                        UAR Server Process                       │
│                                                                 │
│  WebSocket Handler (/api/realtime)                              │
│       │                                                         │
│       ▼                                                         │
│  RealtimeBroker (tokio::broadcast + topic registry)             │
│       │                    ▲                                     │
│       │                    │ publish()                           │
│       ▼                    │                                     │
│  Channel Registry      Run Handlers / Tool Handlers             │
│  (HashMap<Topic, Subs>)    Agent Subsystem                      │
│       │                    Session Manager                       │
│       │                    Plugin System                         │
│       ▼                                                         │
│  Per-Connection Writer Task                                     │
│  (tokio::task per WebSocket connection)                         │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Topic Access Control (Planned)

```rust
// Each topic pattern has a resolver function
broker.register_topic_pattern(
    "agent:run:{run_id}",
    |ctx: &AuthContext, run_id: &str| async move {
        // Check: does the user own the session that contains this run?
        app_state.db.user_owns_run(&ctx.user_id, run_id).await
    }
);
```

---

## 3. Plugin Integration

Plugins can both **emit** and **subscribe** to events via the Plugin Realtime API.

### 3.1 Plugin Manifest Declaration

```yaml
# plugin.yaml
name: visual-canvas
version: 1.0.0
realtime:
  channels:
    - topic: "plugin:visual-canvas:board:{board_id}"
      description: "Real-time canvas updates for a specific board"
      presence: true
      events:
        publish:
          - name: "plugin:visual-canvas:element:created"
            schema: "./schemas/element-created.json"
          - name: "plugin:visual-canvas:element:moved"
            schema: "./schemas/element-moved.json"
        subscribe:
          - name: "session:{session_id}"
            filter: ["agent:run:completed"]  # listen to agent completions
```

### 3.2 Plugin JavaScript API

In a plugin's sandboxed JavaScript environment:

```javascript
// Available via the UAR Plugin Runtime global
const rt = UAR.realtime;

// Publish to your plugin's channel
await rt.publish(`plugin:visual-canvas:board:${boardId}`, 'plugin:visual-canvas:element:created', {
  element_id: 'el_123',
  type: 'rect',
  x: 100,
  y: 200,
});

// Subscribe to another channel you declared as a consumer
const unsub = rt.subscribe(`session:${sessionId}`, 'agent:run:completed', (payload) => {
  console.log('Agent run completed:', payload.run_id);
  refreshCanvas();
});

// Track presence on your channel
rt.track(`plugin:visual-canvas:board:${boardId}`, {
  display_name: currentUser.name,
  cursor: { x: 0, y: 0 },
});
```

---

## 4. External API Caller Integration (REST + WebSocket)

For third-party developers using the UAR as a backend:

### Step 1 — Get an API Key

```bash
POST /api/keys
Authorization: Bearer <user_jwt>

{"name": "my-app", "scopes": ["chat:write", "realtime:read"]}
```

### Step 2 — Connect to Realtime

```javascript
// Node.js / Deno
const ws = new WebSocket('wss://your-uar-instance.example.com/api/realtime?token=uar_key_...');
ws.addEventListener('open', () => {
  ws.send(JSON.stringify({
    v: 1,
    join_ref: '1',
    ref: '1',
    topic: 'session:sess_abc123',
    event: 'uar_join',
    payload: { config: { presence: false } },
  }));
});

ws.addEventListener('message', (msg) => {
  const envelope = JSON.parse(msg.data);
  if (envelope.event === 'session:message:created') {
    console.log('New message in session:', envelope.payload);
  }
});
```

### Step 3 — Start a Chat Run and Watch Events

```bash
# Start an AI run via REST
POST /api/chat/completion
Authorization: Bearer uar_key_...
Content-Type: application/json
X-UAR-Session-ID: sess_abc123

{"model": "gpt-4o", "messages": [...], "stream": false}
```

The returned `run_id` can then be used to subscribe to `agent:run:{run_id}` over WebSocket for real-time token streaming and tool call events.

---

## 5. AsyncAPI Spec Consumption

The machine-readable spec is available at:

```
GET /api/realtime/spec
```

You can use it to:
- Generate client SDKs in any language using AsyncAPI Generator
- Validate your integration with AsyncAPI Diff
- Auto-generate documentation with AsyncAPI React component
- Lint custom event schemas

```bash
# Generate a TypeScript client from the spec
npx @asyncapi/generator /api/realtime/spec @asyncapi/typescript-nats-template -o ./sdk-out
```

---

## 6. Connection Lifecycle Best Practices

### Reconnection

Always implement exponential backoff reconnection:

```typescript
const BACKOFF = [1000, 2000, 5000, 10000, 30000];
let attempt = 0;

function reconnect() {
  const delay = BACKOFF[Math.min(attempt++, BACKOFF.length - 1)];
  setTimeout(() => {
    rt.connect().then(() => { attempt = 0; });
  }, delay);
}

rt.onClose(() => reconnect());
```

### Rejoin after reconnect

Channel subscriptions must be re-established after a reconnect. The SDK handles this automatically via `autoRejoin: true` (default).

### Presence cleanup

The server automatically removes presence entries when a WebSocket closes — no explicit cleanup needed.

### Heartbeat failure

If the server doesn't receive a heartbeat within `realtime.heartbeat_timeout_seconds` (default 60s), it closes the connection with `uar_close`. The client SDK automatically sends heartbeats every 30s.
