// Operator host launcher. Credentials/configuration never enter the Cucumber child.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { randomUUID } from 'node:crypto';
import { mkdir, open, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import { assertOwnedEndpoint } from '../behaviors/support/restart-owned-uar.mjs';
import { prepareEmbeddingHost, restoreEmbeddingHost } from './embedding-config.mjs';

const id = randomUUID();
const output = resolve(`.prometheus/verification/knowledge-regeneration-host/${id}`);
await mkdir(output, { recursive: true });
const runtime = JSON.parse(await readFile('.prometheus/verification/manual-runtime.json', 'utf8'));
const receipt = { scenario: 'KNOWLEDGE-REGENERATION-01', platform: process.platform, started_at: new Date().toISOString(),
  command: ['node', 'node_modules/@cucumber/cucumber/bin/cucumber.js', '--config', 'scripts/behaviors/knowledge-regeneration.cucumber.mjs'],
  child_credential_variables: [], preparation: {}, restoration: {}, output };
const state = {};
let failure;
try {
  assert.equal(process.platform, 'darwin');
  await assertOwnedEndpoint(new URL(runtime.base_url));
  await prepareEmbeddingHost(state, id, receipt.preparation);
  const env = Object.fromEntries(['HOME', 'LANG', 'LC_ALL', 'PATH', 'TMPDIR'].filter(k => process.env[k] !== undefined).map(k => [k, process.env[k]]));
  Object.assign(env, { UAR_BASE_URL: runtime.base_url, UAR_WORKSPACE_ID: runtime.workspace_id,
    UAR_CUCUMBER_REPORT: resolve(output, 'cucumber.json') });
  const log = await open(resolve(output, 'execution.log'), 'wx', 0o600);
  try {
    const child = spawn(process.execPath, receipt.command.slice(1), { env, stdio: ['ignore', log.fd, log.fd] });
    receipt.exit_code = await new Promise((done, reject) => { child.once('error', reject); child.once('close', (code, signal) => { receipt.signal = signal; done(code); }); });
  } finally { await log.close(); }
  console.log(await readFile(resolve(output, 'execution.log'), 'utf8'));
  assert.equal(receipt.exit_code, 0, 'Selected Cucumber scenario failed; retained execution log');
} catch (error) {
  failure = error;
  receipt.failure = error.message;
} finally {
  try { await restoreEmbeddingHost(state, receipt.restoration); }
  catch (error) { receipt.restore_failure = error.message; failure ??= error; }
  receipt.completed_at = new Date().toISOString();
  receipt.status = failure ? 'FAILED' : 'PASSED';
  await writeFile(resolve(output, 'host.json'), JSON.stringify(receipt, null, 2) + '\n', { flag: 'wx', mode: 0o600 });
  console.log(JSON.stringify({ status: receipt.status, output, failure: receipt.failure, restore_failure: receipt.restore_failure }));
}
if (failure) process.exitCode = 1;
