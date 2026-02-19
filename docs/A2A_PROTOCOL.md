# A2A Protocol Integration

The UAR implements **Google's Agent-to-Agent (A2A) Protocol RC v1.0**, enabling interoperability with any A2A-compatible agent framework.

## AgentCard

Every UAR instance exposes its capabilities at:

```
GET /.well-known/agent.json
```

Example response:

```json
{
  "name": "UAR Compiler Agent",
  "description": "Compiles UAR-AGENT-MD specifications into signed agent descriptors",
  "url": "https://your-uar-instance.example.com/a2a/compiler",
  "version": "1.0.0",
  "capabilities": {
    "streaming": false,
    "pushNotifications": false,
    "stateTransitionHistory": false
  },
  "skills": [
    {
      "id": "uar.compile",
      "name": "Compile Agent Spec",
      "description": "Compiles a UAR-AGENT-MD document into a signed descriptor"
    }
  ]
}
```

## JSON-RPC Endpoint

```
POST /a2a/compiler
Content-Type: application/json
Authorization: Bearer <token>
```

Supports the A2A JSON-RPC 2.0 protocol with these methods:

| Method | Description |
|--------|-------------|
| `message/send` | Send a message (creates or continues a session) |
| `tasks/get` | Get task/session status |
| `tasks/cancel` | Cancel a running task |

## Multi-Turn Sessions

A2A `contextId` maps directly to a UAR `CompilerSession.id`. The first `message/send` creates a new session; subsequent messages with the same `contextId` continue the session.

```
Client                          UAR A2A Handler
  │                                    │
  │── message/send (no contextId) ────▶│
  │                                    │── create_session()
  │◀── Task{id, contextId, Working} ───│
  │                                    │
  │── message/send (contextId=X) ─────▶│
  │                                    │── get_session(X) → update
  │◀── Task{id, contextId, Working} ───│
  │                                    │
  │── tasks/get (taskId=Y) ───────────▶│
  │◀── Task{Completed, artifacts=[descriptor.json]} ─│
```

## Agent Registry & Discovery

The UAR provides a federation registry for discovering other A2A agents:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/a2a/registry/register` | Register or update an agent |
| `GET`  | `/a2a/registry/agents` | List all registered agents |
| `GET`  | `/a2a/registry/agents/:id` | Get a specific agent |
| `GET`  | `/a2a/registry/skills` | List all skills across all agents |

### Registering an Agent

```bash
curl -X POST http://localhost:3928/a2a/registry/register \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Custom Agent",
    "description": "Does something useful",
    "base_url": "https://my-agent.example.com",
    "capabilities": ["my.skill.one", "my.skill.two"]
  }'
```

### Discovering Skills

```bash
curl http://localhost:3928/a2a/registry/skills \
  -H "Authorization: Bearer <token>"
```

Response:
```json
[
  {
    "agent_id": "abc-123",
    "agent_name": "My Custom Agent",
    "skill_id": "my.skill.one",
    "agent_base_url": "https://my-agent.example.com"
  }
]
```
