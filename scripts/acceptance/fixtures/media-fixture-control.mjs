// Host lifecycle adapter for the externally provisioned #386 fixture.
// This program does not resolve credentials or access Docker configuration values.
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { isAbsolute } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

function setting(name) {
  const value = process.env[name];
  assert.ok(value, 'Missing media fixture setting ' + name);
  return value;
}

function run(executable, args, input) {
  try {
    return execFileSync(executable, args, {
      encoding: 'utf8', input, stdio: ['pipe', 'pipe', 'pipe'],
      timeout: 60_000, maxBuffer: 8 * 1024 * 1024,
    }).trim();
  } catch {
    throw new Error('Media fixture command failed: ' + executable + ' ' + args[0]);
  }
}

function docker(...args) {
  return run('docker', args);
}

const operation = process.argv[2];
const container = setting('UAR_MEDIA_SERVICE_CONTAINER');
assert.match(container, /^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/);
const target = docker('inspect', '--type', 'container', '--format',
  '{{.Id}} {{index .Config.Labels "uar.media-fixture"}}', container).split(' ');
assert.match(target[0], /^[0-9a-f]{64}$/);
assert.equal(target[1], '386', 'Only a dedicated media fixture container may be controlled');
const id = target[0];
const pid = () => Number(docker('inspect', '--format', '{{.State.Pid}}', id));

async function ready() {
  const url = new URL('/readyz', setting('UAR_BASE_URL'));
  assert.ok(['http:', 'https:'].includes(url.protocol) && !url.username && !url.password);
  const deadline = Date.now() + 40_000;
  while (Date.now() < deadline) {
    const response = await fetch(url, {
      redirect: 'error', signal: AbortSignal.timeout(2_000),
    }).catch(() => null);
    if (response?.ok) return;
    await delay(250);
  }
  throw new Error('Media fixture did not become ready');
}

function store(query) {
  const path = setting('UAR_MEDIA_STORE_PATH');
  assert.ok(isAbsolute(path), 'Use the fixture container bind mount store path');
  const output = run(setting('UAR_MEDIA_STORE_RUNNER'), [
    'surrealkv://' + path, setting('UAR_MEDIA_STORE_NAMESPACE'), setting('UAR_MEDIA_STORE_DATABASE'),
  ], query);
  return JSON.parse(output);
}

switch (operation) {
  case 'seed-legacy-catalog': {
    // Run during external provisioning, with a fresh and stopped fixture store.
    assert.equal(pid(), 0, 'Stop the disposable fixture before seeding its store');
    const legacy = JSON.parse(readFileSync(new URL('./media-legacy-catalog.json', import.meta.url), 'utf8'));
    const created = store('CREATE uar_collaboration_state:catalog CONTENT ' + JSON.stringify(legacy) + ';');
    assert.ok(Array.isArray(created) && created.length === 1, 'Expected one seeded catalog');
    console.log(JSON.stringify(created[0]));
    break;
  }
  case 'restart': {
    const before = pid();
    assert.ok(before > 1, 'Media fixture must be running');
    docker('restart', id);
    await ready();
    console.log(JSON.stringify({ before, after: pid() }));
    break;
  }
  case 'legacy-catalog': {
    const legacy = JSON.parse(readFileSync(setting('UAR_MEDIA_LEGACY_RECEIPT'), 'utf8'));
    console.log(JSON.stringify(legacy));
    break;
  }
  case 'snapshot': {
    assert.ok(pid() > 1, 'Media fixture must be running');
    docker('stop', id);
    let catalog;
    try {
      const selected = store('SELECT * FROM uar_collaboration_state:catalog;');
      assert.ok(Array.isArray(selected) && selected.length === 1, 'Expected one fixture catalog');
      catalog = selected[0];
    } finally {
      docker('start', id);
      await ready();
    }
    const logs = spawnSync('docker', ['logs', id], {
      encoding: 'utf8', timeout: 15_000, maxBuffer: 8 * 1024 * 1024,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    assert.ok(!logs.error && logs.status === 0, 'Unable to read fixture diagnostics');
    console.log(JSON.stringify({ catalog, diagnostics: logs.stdout + '\n' + logs.stderr }));
    break;
  }
  default:
    throw new Error('Unknown media fixture lifecycle operation');
}
