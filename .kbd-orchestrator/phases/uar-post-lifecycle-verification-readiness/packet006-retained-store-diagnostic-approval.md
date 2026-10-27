# Proposed retained-store diagnostic — approval required, NOT RUN

## Why this is the next step

No architecture redesign is supported by current evidence. The frozen SSE parser and default production SSE projection agree on their schema. An earlier public terminal error may prevent both expected budget classification and successful history completion, but actual response bodies were not retained. KB's shared publication fence is a localized contention candidate; the retained hash does not prove its cause. Public Failed can represent a durable Retryable job; peer receipts exclude embedding-hold expiration for006.

Earlier repairs fixed real fixture/adapter issues, but they did not establish these remaining causes. Making another runtime change without the actual errors risks another symptom-only repair. No new production or fixture source was edited in this investigation. Two isolated source-only handoffs are in evidence/v1.3.0-execution-006/.

## Exact requested exception and limits

Approve only this bounded diagnostic exception to AGENTS.md's closed verification/service-probe gate: start the same owned pinned local store, send one read-only SELECT batch in its existing namespace/database, save private owner-scoped evidence, then exact-owned stop/down even on failure. At most15connection-refused startup attempts with one-second spacing, never an HTTP/query-error or timeout retry. No main/peer launch, provider/tool/model call, test, compiler/build, INSERT/UPDATE/DELETE, schema change, row cleanup or acceptance credit. Original A6epoch480calls/240active-minute limits/floors remain; actual budget must be accounted before execution and stop if exhausted/unknown. This does not grant a sixth repair or verification rerun.

Unfinished implementation/evidence items: ERROR01/02's actual earlier error code/cause, HISTORY terminal failure, KB underlying publication/job failure. Source inspection cannot recover discarded response bytes or deleted records. This diagnostic can expose retained005run record/events and any remaining owned ingestion job metadata; it cannot prove a fix, current runtime correctness or full phase acceptance. KB cleanup deletes its ingestion records, so an empty third result is expected and must be reported honestly; it does not prove successful ingestion. Further controlled reproduction would need a separate exact diagnostic/repair authority and improved evidence capture. No source-complete inventory or newly runnable exit contract is claimed here.

The two005run IDs come from that attempt's retained Starting new run log events. History SELECTs require the exact history-owner key AND those exact IDs; event rows join through that owner proof. Ingestion SELECT is restricted to the exact synthetic principal in the isolated fixture namespace, excludes immutable source bytes, and caps50rows. Query SQL/HTTP headers follow the pinned source and official protocol documentation: https://surrealdb.com/docs/reference/rest-api/http-protocol . No credential values appear in results or terminal summary. Private evidence may contain the controlled fixture's run/event/error payloads and is saved mode0600 inside a0700directory. No external publication.

## Exact commands in order

```sh
docker compose --project-name uar-post-lifecycle-v130-001 --file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml up --detach --no-deps store
```

```sh
/Users/gqadonis/.pyenv/versions/3.11.6/bin/python3.11 - <<'PY_DIAGNOSTIC'
import base64, errno, json, os, time, urllib.error, urllib.request
from pathlib import Path
root = Path('/Users/gqadonis/Projects/prometheus/universal-agent-runtime')
evidence = root / '.kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/evidence/retained-store-diagnostic-001'
evidence.mkdir(mode=0o700, exist_ok=False)
queries = ['SELECT run_id,data FROM durable_run_records WHERE owner_key="v1:t:25:client-tools-local-tenant:s:24:uar-post-lifecycle-owner" AND run_id IN ["817f52e2-5ec0-404c-b432-a555a2604488", "24bb7ca2-7a08-4375-a233-76564fe5d084"] LIMIT 2;', 'SELECT run_id,event_id,data FROM durable_run_events WHERE run_id IN (SELECT VALUE run_id FROM durable_run_records WHERE owner_key="v1:t:25:client-tools-local-tenant:s:24:uar-post-lifecycle-owner" AND run_id IN ["817f52e2-5ec0-404c-b432-a555a2604488", "24bb7ca2-7a08-4375-a233-76564fe5d084"]) ORDER BY event_id ASC LIMIT 100;', 'SELECT document_id,state,data FROM principal_knowledge_ingestion WHERE owner_id="v1:k:issuer:i:27:uar-client-tools-local-gate:t:25:client-tools-local-tenant:s:24:uar-post-lifecycle-owner" LIMIT 50;']
namespace = 'uar_post_lifecycle_v130_001'
database = 'uar_post_lifecycle_v130_001'
auth = base64.b64encode(b'root:uar-post-lifecycle-v130-001-public-fixture-store').decode('ascii')
request = urllib.request.Request('http://127.0.0.1:49312/sql',
    data=('\n'.join(queries)).encode('utf-8'), method='POST',
    headers={'Authorization': 'Basic ' + auth, 'Accept': 'application/json',
             'Content-Type': 'text/plain', 'Surreal-NS': namespace, 'Surreal-DB': database})
for attempt in range(1, 16):
    try:
        with urllib.request.urlopen(request, timeout=5) as response:
            raw = response.read(4 * 1024 * 1024 + 1)
            if len(raw) > 4 * 1024 * 1024:
                raise RuntimeError('diagnostic_response_exceeds_reviewed_bound')
        break
    except urllib.error.HTTPError:
        raise
    except urllib.error.URLError as error:
        if getattr(error.reason, 'errno', None) != errno.ECONNREFUSED or attempt == 15:
            raise
        time.sleep(1)
result = json.loads(raw)
if not isinstance(result, list) or len(result) != len(queries):
    raise RuntimeError('unexpected_diagnostic_result_shape')
for item in result:
    if item.get('status') != 'OK':
        raise RuntimeError('diagnostic_query_failed_no_retry')
fd = os.open(evidence / 'owner-scoped-results.private.json', os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
with os.fdopen(fd, 'wb') as output:
    output.write(raw)
print(json.dumps({'diagnosticOnly': True, 'rowCounts': [len(item.get('result', [])) for item in result],
                  'privateReceipt': str(evidence / 'owner-scoped-results.private.json')}))
PY_DIAGNOSTIC
```

Cleanup after success or first failure:

```sh
docker compose --project-name uar-post-lifecycle-v130-001 --file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml stop --timeout 30 store
docker compose --project-name uar-post-lifecycle-v130-001 --file .kbd-orchestrator/phases/uar-post-lifecycle-verification-readiness/v1.3.0-fixtures/local-store.yaml down --timeout 30
```

No extra preflight, install, service inspection or query follows automatically. One diagnostic invocation only; absent data and failures are retained, never substituted with guessed causes. Actual scoped source repairs and their inventory/amendment would be prepared only from established evidence.

## Approval text

> Approve the exact retained-store diagnostic001 exception in this document: owned pinned store up, the bounded owner-scoped read-only SQL batch and private evidence capture, then owned stop/down. Preserve A6budget/consumption and all source/acceptance boundaries. No tests/builds/main/model calls, database mutation, sixth repair, verification retry, publication or extra preflight.

Reason approval is necessary: AGENTS.md Operator verification lock explicitly prohibits unnamed service/readiness/preflight commands while CLOSED, and the previous approval was for packet006 only. This is not claimed permission inferred from the request to fix the bugs.
