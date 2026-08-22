# Non-root held-work container control

Profile: `server-full` only.

The control image was built from source-only control commit
`a9b50220995d11d4cbb944e00cc3ed2274f355ae`, whose tree contains the three
runtime source edits and no dirty-worktree artifacts. The release binary was
copied into the existing non-root runtime image. Its SHA-256 was
`746cd57f32890d792f6fdf2a47726131c93e964ea12ea7a528d59665df65fb82`
both before and inside the image. The resulting image digest was
`sha256:323c761a47eb466b46422b6b663e56cf6c71e53064d7ccd09c4cd95b447545dd`.

The container started as UID `65532` with a writable SurrealKV volume and
became ready at `http://127.0.0.1:1906/readyz`; its inherited Docker
healthcheck also reported `healthy` against `/health`. A real
`/api/uar/sync/stream` response produced `event: connected` and continued to
emit heartbeats before shutdown.

The control then ran the same boundary assertions added to
`scripts/certify-release-candidate.sh`:

```text
uid=65532
health=healthy
elapsed_ms=30489
held_sse_exit=18
container_exit=0
deadline_marker=1
graceful_marker=0
die_event=1
sigkill_event=0
```

The configured UAR deadline was 30 seconds and Docker's external stop limit
was 35 seconds. `docker stop --time 35` returned after 30,489 ms. The held
curl terminated with exit 18 (`transfer closed with outstanding read data
remaining`), UAR exited 0, and the only synchronous outcome marker was:

```text
UAR_SHUTDOWN outcome=deadline_enforced
```

The captured Docker events show signal 15, `stop`, and `die` with exit code
0. They contain no signal 9/SIGKILL event. Exact structured results are in
`non-root-container.json`; the relevant raw events are in
`non-root-container-events.jsonl`.

This focused control does not replace the parent phase's clean-checkout
10,800-second certification. It establishes the child change's 30/35-second
container boundary before a new immutable candidate is committed.
