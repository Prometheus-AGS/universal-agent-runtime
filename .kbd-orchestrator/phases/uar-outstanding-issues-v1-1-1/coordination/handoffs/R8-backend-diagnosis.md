# R8 backend readiness diagnosis

## Scope and status

This is a source and retained-artifact diagnosis for the local acceptance run
`20261016-local-01`. It does not change production code, fixtures, manifests,
or private infrastructure. The authoritative waypoint places
`uar-outstanding-issues-v1-1-1` in Execute, task 6.2, with acceptance in
progress.

## Observed receipt evidence

The retained receipt records these two blocks:

- `streams` blocked from `1792151159.818623` through `1792151341.765758` with
  `isolated Surreal did not become ready: RemoteDisconnected`.
- `recovery` blocked from `1792151428.844924` through `1792151610.1908228`
  with the same message.

For the first isolated Surreal instance, command records show successful:

1. owned Docker volume creation (`command-00010`),
2. Docker launch with the pinned `surrealdb/surrealdb:v3.3.0` digest, root
   user, redacted private password, `--bind 0.0.0.0:8000`, and
   `rocksdb:/data/phase.db` (`command-00011`), and
3. Docker port resolution to a loopback address (`command-00012`).

The same owned startup sequence began again for `combined` in commands
`00021` and `00022`. None of the retained command records captures an HTTP
status or body from `/health`.

## Demonstrated harness defect

`Http.request()` returns the decoded body when `return_status` is false. For a
successful response with an empty body it returns `None`:

```python
result = json.loads(raw) if raw else None
return (status, result) if return_status else result
```

`Resources.ensure_backend("surreal")` uses that body as its readiness
predicate:

```python
wait_for("isolated Surreal", lambda: Http("http://" + address).request("GET", "/health"))
```

Therefore, an HTTP 200 `/health` response with no body can never satisfy this
readiness check. That is a demonstrated source failure path independent of
whether it happened during either observed timeout.

## Why `RemoteDisconnected` is insufficient to identify the backend cause

`wait_for()` stores the last caught exception but does not replace it after a
successful probe returns a false value. An initial `RemoteDisconnected` can
therefore remain in the final error even if every later `/health` request
reached the backend and returned an empty 200 response. The receipt's final
exception proves that at least one probe raised that exception. It does not
prove that the last probe failed, that the container exited, or that the
pinned Surreal command was invalid.

Cleanup retained three sanitized Surreal logs after the program ended. Each
reports SurrealDB 3.3.0 starting its RocksDB store, `Startup complete; instance
is now ready to serve`, and `Started web server on 0.0.0.0:8000`; the final
server was ready about 2.3 seconds after its first startup log. None records a
fatal error. The configured image, volume, credentials, bind address, and
RocksDB target therefore started successfully in all three blocked groups.

The retained artifacts do not record individual HTTP responses, so they cannot
directly prove a later empty 200 response. They do rule out the proposed
container-startup/configuration explanation for this acceptance block. No claim
about a network proxy or Docker forwarding fault is warranted from these logs.

## Minimal correction for the lead-owned support cohort

Keep `Http.request()` unchanged. Change only the isolated Surreal readiness
predicate to inspect the accepted HTTP status:

```python
wait_for(
    "isolated Surreal",
    lambda: Http("http://" + address).request(
        "GET", "/health", return_status=True
    )[0] == 200,
)
```

This preserves the existing request error handling and makes a successful empty
health response ready. It is the minimum correction for the demonstrated
failure path.

Also revise `wait_for()` so its final diagnostic describes the final failed
probe state rather than an exception retained from an earlier iteration. This
is diagnostic correctness, separate from the readiness predicate. A false
probe should overwrite the prior exception marker with a non-exception state;
otherwise future receipt messages can misattribute a timeout.

## Unresolved evidence and acceptance disposition

The completed program retained and this diagnosis inspected each sanitized
Surreal log. The only evidence-based acceptance disposition now is:

- `streams`, `recovery`, and `combined` are blocked after a server that logged
  successful startup and its web listener was rejected by the readiness loop.
- The readiness predicate is the concrete acceptance-harness defect: it rejects
  a valid empty 200 health response. The retained exception can be stale and
  cannot identify the final probe result.
- The precise request/response sequence that first raised
  `RemoteDisconnected` remains unverified because the harness did not retain
  probe observations.
- No physical supervision, backend qualification, database mutation, or rerun
  is claimed by this diagnosis.

The lead owns the shared support-code repair and any coherent corrective
acceptance cohort. A rerun must follow the phase's failed-group repair contract
after the implementation change is complete.
