# Staged unanswered-demo report

A normal authenticated UAR run can offer `unanswered_demo_report` for one operator-selected principal and mailbox scope. The tool recognizes explicit English demo requests in subjects. Coverage includes only the selected inbox and sent folders through the configured cutoff. The result contains opaque evidence handles, a coverage indicator and a truncation indicator. A partial scan cannot label a thread unanswered.

## Configure the connector and executor

Install the MailCraft CLI with restricted header paging, the reviewed BunBox MCP provider and its qualified microsandbox runtime. Obtain the provider binary hash from a trusted build. To verify the binary on macOS or Linux, use `shasum -a 256 /opt/pilot/bunbox-mcp`. The recipe source digest is pinned in UAR and must match the BunBox executable's embedded recipe.

Provision a restricted skill-local MCP server and activate its skill through normal run admission. Set the trusted server command to an absolute path and supply these arguments in order. Use an empty declared MCP environment.

```json
{
  "command": "/opt/pilot/mailcraft-cli",
  "args": ["mcp", "--file", "/opt/pilot/fixture.json", "--header-scope", "1:11,12",
           "--header-cutoff", "1768478400", "--header-key-file", "/opt/pilot/header.key"]
}
```

The optional key is 32 random bytes in an operator-owned mode-0600 file. Supply it on both connections to resolve evidence independently within the same logical snapshot and field selection. Restrict the report to the authenticated principal's exact issuer kind, issuer, subject and optional tenant. The following example uses a local JWT principal.

```yaml
harness:
  unanswered_demo:
    issuer_kind: local-jwt
    issuer: "urn:uar:local:jwt"
    subject: pilot-operator
    tenant: null
    mailcraft_tool: pilotmail__restricted__page_message_headers
    mailcraft_command: /opt/pilot/mailcraft-cli
    mailbox_file: /opt/pilot/fixture.json
    header_key_file: /opt/pilot/header.key
    account: 1
    folders: [11, 12]
    cutoff: 1768478400
    bunbox_command: /opt/pilot/bunbox-mcp
    bunbox_sha256: REPLACE_WITH_VERIFIED_LOWERCASE_SHA256
    state_dir: /opt/pilot/private/jobs
    msb_command: /opt/pilot/msb
    msb_home: /opt/pilot/private/msb
```

Use the provider-visible tool name from the granted MCP catalog. If the run policy selects individual tools, include both `unanswered_demo_report` and the scoped pager. The staged tool stays hidden when the policy excludes either one or the live pager grant is revoked. The jobs directory and qualified microsandbox home belong to the UAR operating-system account. The private jobs directory must permit only the owner. Its parent must already be accessible. BunBox controls the VM and enforces its pinned image. Only an explicit, admitted call to the staged tool starts BunBox.

## Run and verify

Start an authenticated run through `POST /api/uar/runs` and activate the connector. The matching root run can call `unanswered_demo_report` with `{}`. UAR applies the ordinary outer-tool approval and admission to every inner read. In the staged profile the model sees the report tool and receives a terminal receipt. The restricted connector's headers stay outside model history. To run the direct comparison with the same connector grant, omit `harness.unanswered_demo` and use the paged MCP tool through the same authenticated runtime.

A successful receipt includes `jobId`, `recipeSha256`, `cutoff`, `evidenceDigest` and `result`. Resolve each `sourceRef` with the restricted connector's `resolve_message_evidence`, using the original account, folder IDs, cutoff and fields. Compare the findings with the expected fixture threads. The scan accepts up to 20 pages of 100 headers, 2,000 records and 2 MiB of staged input. Truncated subjects, changed coverage, repeated cursors, malformed projections and revoked bindings fail before report delivery. Each result list has a limit of 20 items. Check `result.truncated` when reporting coverage.

A missing provider returns `executor_unavailable`. A binary digest mismatch returns `executor_pin_mismatch`. An uncertain submission or unconfirmed cleanup returns `outcome_unknown` or `cleanup_unknown`. UAR persists its submit intent before dispatch and can read a previously recorded job for the same owner, run, recipe and input. Preserve the private job directory for operator reconciliation after an uncertain outcome.
