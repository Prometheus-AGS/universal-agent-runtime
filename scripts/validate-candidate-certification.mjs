#!/usr/bin/env node

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const [directory] = process.argv.slice(2);
if (!directory) {
  console.error("usage: validate-candidate-certification.mjs <evidence-directory>");
  process.exit(2);
}

const failures = [];
const readJson = (name) => {
  const path = join(directory, name);
  if (!existsSync(path)) {
    failures.push(`missing evidence: ${name}`);
    return {};
  }
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    failures.push(`invalid JSON in ${name}: ${error.message}`);
    return {};
  }
};

const result = readJson("results.json");
const lifecycle = readJson("lifecycle.json");
const mcp = readJson("mcp-process-boundary.json");
const load = readJson("parallel-load.json");
const soak = readJson("soak.json");
const upgrade = readJson("upgrade.json");

if (result.schema_version !== 2 || result.outcome !== "passed") failures.push("candidate result is not schema v2/passed");
if (!/^[0-9a-f]{40}$/.test(result.source_sha ?? "")) failures.push("candidate result has invalid source SHA");
if (!result.candidate_tag) failures.push("candidate result has no candidate tag");
if (result.backup_sha256 !== result.restored_sha256) failures.push("backup and restored datastore digests differ");
for (const [name, evidence] of [["lifecycle", lifecycle], ["soak", soak]]) {
  if (evidence.source_sha !== result.source_sha) failures.push(`${name} source SHA does not match candidate result`);
  if (evidence.candidate_tag !== result.candidate_tag) failures.push(`${name} tag does not match candidate result`);
}
if (!lifecycle.startup || !lifecycle.readiness || !lifecycle.restart || lifecycle.sigterm_exit_code !== 0) {
  failures.push("lifecycle evidence did not pass startup/readiness/restart/SIGTERM");
}
if (load.source_sha !== result.source_sha || load.candidate_tag !== result.candidate_tag || load.failures !== 0 || !(load.requests > 0)) {
  failures.push("parallel load evidence is source-mismatched, has failures, or has no requests");
}
try {
  const failureRows = readFileSync(join(directory, "failure-recovery.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  const expected = new Set(["provider-outage", "rate-limit", "malformed-provider-stream"]);
  for (const row of failureRows) {
    if (row.source_sha !== result.source_sha || row.candidate_tag !== result.candidate_tag || !row.surfaced) {
      failures.push("provider failure/recovery evidence is source-mismatched or unsurfaced");
    }
    expected.delete(row.failure);
  }
  if (expected.size > 0) failures.push(`provider failure/recovery evidence is missing: ${[...expected].join(", ")}`);
} catch (error) {
  failures.push(`invalid provider failure/recovery evidence: ${error.message}`);
}
if (
  mcp.source_sha !== result.source_sha ||
  mcp.candidate_tag !== result.candidate_tag ||
  !mcp.stdio_discovery ||
  !mcp.tool_call ||
  !mcp.transport_loss_surfaced ||
  mcp.crash_failure_events !== 1 ||
  mcp.crashed_call_replayed !== false ||
  !mcp.reconnected_after_crash ||
  mcp.tool_timeout_seconds !== 30 ||
  mcp.observed_timeout_seconds < 30 ||
  mcp.observed_timeout_seconds >= 45 ||
  mcp.timeout_failure_events !== 1 ||
  mcp.timed_out_call_replayed !== false ||
  !mcp.reconnected_after_timeout
) {
  failures.push("MCP process/transport/reconnect/timeout evidence is incomplete or source-mismatched");
}
const mcpEvidenceValidation = spawnSync(
  process.execPath,
  [
    fileURLToPath(new URL("./validate-mcp-process-boundary-evidence.mjs", import.meta.url)),
    join(directory, mcp.raw_evidence?.crash_stream ?? ""),
    join(directory, mcp.raw_evidence?.timeout_stream ?? ""),
    join(directory, mcp.raw_evidence?.process_trace ?? ""),
  ],
  { encoding: "utf8" },
);
if (mcpEvidenceValidation.status !== 0) {
  failures.push(
    `MCP raw process-boundary evidence failed replay: ${mcpEvidenceValidation.stderr.trim() || mcpEvidenceValidation.error?.message || "unknown failure"}`,
  );
}
if (!(soak.configured_duration_seconds > 0) || soak.observed_duration_seconds < soak.configured_duration_seconds) {
  failures.push("streaming soak did not run for its configured duration");
}
if (soak.errors !== 0 || soak.duplicate_events !== 0) failures.push("streaming soak recorded errors or duplicate events");
if (soak.p95_latency_ms > soak.thresholds?.p95_latency_ms) failures.push("streaming soak exceeded p95 latency threshold");
if (soak.max_rss_kib - soak.initial_rss_kib !== soak.peak_rss_growth_kib) {
  failures.push("streaming soak peak RSS growth is not derived from max RSS minus initial RSS");
}
if (soak.peak_rss_growth_kib > soak.thresholds?.peak_rss_growth_kib) {
  failures.push("streaming soak exceeded peak RSS growth threshold");
}

if (result.journeys?.includes("prior-version-upgrade")) {
  if (
    upgrade.status !== "passed" ||
    upgrade.source_sha !== result.source_sha ||
    upgrade.previous_source_sha === result.source_sha ||
    !["published-release-artifact", "controlled-source-rebuild"].includes(upgrade.previous_build_kind) ||
    upgrade.previous_public_release_verified !== true ||
    !/^[0-9a-f]{40}$/.test(upgrade.previous_tag_object_sha ?? "") ||
    !["tag", "commit"].includes(upgrade.previous_tag_object_type) ||
    !/^https:\/\/github\.com\//.test(upgrade.previous_release_url ?? "") ||
    !upgrade.previous_published_at ||
    !upgrade.continuity_record_id ||
    !upgrade.continuity_marker ||
    upgrade.continuity_record_kind !== "durable-setting" ||
    upgrade.previous_type_create_http_status !== 200 ||
    upgrade.previous_create_http_status !== 200 ||
    upgrade.previous_read_http_status !== 200 ||
    upgrade.candidate_read_http_status !== 200 ||
    upgrade.rollback_read_http_status !== 200 ||
    !/^[0-9a-f]{64}$/.test(upgrade.previous_record_sha256 ?? "") ||
    upgrade.candidate_record_sha256 !== upgrade.previous_record_sha256 ||
    upgrade.rollback_record_sha256 !== upgrade.previous_record_sha256 ||
    !/^[0-9a-f]{64}$/.test(upgrade.pre_upgrade_backup_tree_sha256 ?? "") ||
    upgrade.rollback_restored_tree_sha256 !== upgrade.pre_upgrade_backup_tree_sha256 ||
    upgrade.upgrade_database_url === upgrade.rollback_database_url
  ) {
    failures.push("prior-version upgrade evidence is incomplete or source-mismatched");
  }
}
if (result.journeys?.includes("non-root-container")) {
  const nonRoot = readJson("non-root-container.json");
  if (nonRoot.source_sha !== result.source_sha || !(nonRoot.uid > 0) || !nonRoot.writable_persistence || nonRoot.sigterm_exit_code !== 0) {
    failures.push("non-root container evidence is incomplete or source-mismatched");
  }
}

if (failures.length) {
  console.error(`Candidate certification validation failed:\n- ${failures.join("\n- ")}`);
  process.exit(1);
}
console.log(`Candidate certification evidence passed for ${result.candidate_tag} (${result.source_sha}).`);
