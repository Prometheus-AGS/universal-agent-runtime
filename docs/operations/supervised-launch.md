# Starting UAR from another program

A desktop application can run UAR as a child process, so its user never starts
a server by hand. UAR calls this *supervised* mode. The host process owns the
child: it hands over a secret at launch, learns the port from one line of
output, and stops the child by closing a pipe.

Two commands start the same mode:

```bash
universal-agent-runtime --supervised --port 0 --config /path/to/host/uar.yaml
```

```bash
uar-sidecar --port 0 --config /path/to/host/uar.yaml
```

A standard install ships `universal-agent-runtime`. `uar-sidecar` is the same
launch contract packaged on its own, for hosts that bundle it. Two things
differ: `uar-sidecar` loads a `.env` file from its working directory or any
parent directory, and only `uar-sidecar` has the embedded-database maintenance
commands (`--lock-embedded-database`, `--migrate-embedded-database`).

## The launch contract

1. **Start the child** with stdin and stdout as pipes. Send stderr wherever
   you keep logs; logging goes there, as JSON lines. Leave `UAR_SIDECAR` and
   `UAR_LIFECYCLE_STDOUT` unset, because they change what is written to
   stdout. `UAR_LOG_FILE` sends the log to a file instead of stderr.
2. **Write the launch token** as the first line of the child's stdin: exactly
   64 lowercase hexadecimal characters (256 random bits) and a newline. Keep
   stdin open afterwards.
3. **Read stdout** until a line `READY:<port>` arrives. It is printed once,
   after UAR can serve requests, so there is nothing to poll. The only other
   lines on stdout are `LIFECYCLE:<json>` replies to control commands.
4. **Call the API** at `http://127.0.0.1:<port>`.
5. **Close stdin** to stop the child. It shuts down and exits with status 0.
   If the host crashes, the pipe closes and the child stops too.

UAR never reads the token from an argument, an environment variable or a file,
and never writes it anywhere. A missing or malformed token line ends the
process with exit status 2 before any socket is bound, and stderr then says
"refused to start". An invalid command line also exits with status 2, with a
usage message instead. An invalid `UAR_SIDECAR_BOUND_PRINCIPAL` ends the
process with status 3.

### Choosing the port

| `--port` | Result |
|---|---|
| `0` | The operating system assigns a free port. Use this unless the user needs a predictable address. |
| omitted | 1906, or the next free port above it. |
| any other value | That port, or the next free port above it. |

The `PORT` environment variable is read as `--port` when the flag is absent,
so clear it if the host's own environment sets it. Always take the port from
the `READY` line.

## What every request needs

| Header | Rule |
|---|---|
| `Authorization: Bearer <launch token>` | Required on every route, including `/health`. Missing or wrong: 401. |
| `Host` | Must be `127.0.0.1:<port>` or `localhost:<port>`. Anything else: 403. |
| `Origin` | Must be absent. Any value: 403. A webview therefore cannot call UAR directly; send requests from the host process. |
| `x-uar-principal: <subject>` | Names the user a request acts for. Needed for owner-scoped calls such as chat completions, unless the process was started with a bound principal. |

A subject is 1 to 128 ASCII characters, starting with a letter or digit, then
letters, digits, `.`, `_`, `:` or `-`. `anonymous` is refused.

To bind one user for the life of the process, start the child with the
environment variable `UAR_SIDECAR_BOUND_PRINCIPAL=<subject>`. Requests may
then omit `x-uar-principal`; a different subject is refused with 403.

## Configuration the host must supply

A supervised child reads configuration like any other UAR process. Pass
`--config` with a file the host owns. Without it UAR loads `./config.yaml` or
the user's `~/.uar/config.yaml`, which belong to a different installation.

`universal-agent-runtime --supervised` loads no `.env` file unless the host
names one with `--env-file` or `UAR_ENV_FILE`. `uar-sidecar` does load one
implicitly, from its working directory or the nearest parent directory that
has one, before it reads its arguments. Such a file can set the port, the
configuration file and the bound principal, and a host cannot rule one out
by owning the working directory alone. Prefer `--supervised` where that
matters.

Set at least these, to locations the host owns and can write:

```yaml
persistence:
  provider: "surreal"
  database_url: "surrealkv:///absolute/path/to/host/data/uar.db"
server:
  ingest_directory: "/absolute/path/to/host/data/ingest"
security:
  settings_mutation_auth_required: false
llm:
  model: "provider/model"
```

- `security.settings_mutation_auth_required` defaults to `true`, and UAR then
  refuses to start without `security.settings_admin_key`. Either turn the
  requirement off, as above, because the launch token already guards every
  route, or pass a key in the environment variable
  `UAR_SECURITY__SETTINGS_ADMIN_KEY` so that none is stored on disk.
- `server.ingest_directory` defaults to `/data/ingest`, a container mount.
  Start-up fails where that path cannot be created, which includes macOS.
- `persistence.database_url` defaults to a path relative to the working
  directory. The embedded store takes an exclusive lock, so two UAR processes
  cannot share one database.

### A build with local embedding needs its model files

A build that includes local embedding, such as the `server-full` build the
native packages use, checks for embedding model files at start-up and stops
when they are missing, with "embedding model assets ... not found" or "strict
local embedding assets ... are not all present". The default location is
`src/uar/runtime/matching/models` relative to the working directory, which a
host's own directory does not have.

Two components read the files, each through its own key. In the configuration,
set both keys, usually to the same directory:

```yaml
models:
  models_dir: "/absolute/path/to/models"   # skill matching
llm:
  embedding:
    models_dir: "/absolute/path/to/models" # vector retrieval
```

Setting only one of them stops start-up. The environment variable
`UAR_MODELS_DIR` covers both for the default `fastembed` backend. The files are
`bg-small-en-v1.5.onnx`, `tokenizer.json`, `config.json`,
`special_tokens_map.json` and `tokenizer_config.json`; the repository keeps
them in `src/uar/runtime/matching/models`.

### The working directory needs the governance policies

Tool governance is mandatory in supervised mode, and UAR loads its Cedar
policy set from a directory named `policies` in the child's working
directory. Start-up fails with "Governed startup requires a readable nonempty
valid Cedar policy set" when it is missing or empty.

Start the child in a directory the host owns that contains a copy of the
repository's `policies/*.cedar` files (`default.cedar`, `tool-approval.cedar`
and `skill-mutation.cedar`). A standard install does not put them anywhere,
so the host has to ship them. The Boss sidecar payload already includes them.

## What supervised mode changes

- One IPv4 loopback listener. The IPv6 companion listener and the A2A gRPC
  listener (port 50051) are not opened, so a supervised child does not collide
  with another UAR on the machine.
- JWT authentication is off unless `UAR_SECURITY__JWT_REQUIRED` or
  `JWT_REQUIRED` is set; the launch token replaces it.
- The operator web interface is not served and no CORS headers are sent.
- No global MCP server is connected. Definitions from `mcp.json` and earlier
  saved settings can still be listed, as destinations a run may be granted.
  Saving or deleting a global definition is refused with 409
  `sidecar_mode_global_mcp_disabled`.
- Skill evolution is off, and memory is off unless a principal is bound.
- Tool governance is mandatory.

## Checking what you started

`GET /api/uar/version` returns the build identity:

```json
{
  "name": "universal-agent-runtime",
  "version": "1.4.0-rc.6",
  "git_commit": null,
  "build_profile": "release",
  "target": "aarch64-apple-darwin",
  "features": ["server", "surreal-backend"],
  "protocols": {
    "a2a": null,
    "ag_ui": "1.0",
    "ag_ui_profile": "uar.agui/1",
    "a2ui": "v0.9.1",
    "mcp": "2025-11-25"
  },
  "administration_schema": 6
}
```

`git_commit` is the commit the binary was built from, or `null` when the build
recorded none. A build records the `UAR_GIT_SHA` build variable when it is
set, which is how a build without a checkout, such as a container build, gets
one. Otherwise a release build records the checked-out commit and a debug
build records nothing.

On a standalone server this route needs no credentials, so an installer can
check which version is running. It is rate limited like other routes. A
supervised child requires the launch token for it, like every other route.

`GET /api/uar/capabilities` returns the named behaviours this build
implements and the administration route manifest. Registered agents, skills
and tools are listed under `/api/uar/discovery/agents`,
`/api/uar/discovery/skills` and `/api/uar/discovery/tools`.

## Windows

The child's stdin must be an inherited named pipe opened with
`FILE_FLAG_OVERLAPPED`, and the environment must contain
`UAR_SIDECAR_STDIN_OVERLAPPED=1`. A plain anonymous pipe is refused at start.

## Not available

- A start mode with every tool disabled by the server. Supervised mode starts
  with no MCP server connected and with the native file, web and terminal tools off by
  default, but built-in tools remain.
- A file that receives the port. Read it from the `READY` line.
