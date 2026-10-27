import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { createHash, randomUUID } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { open, readFile, writeFile } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';

async function digest(path) {
  const hash = createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}

function alive(pid) {
  try { process.kill(pid, 0); return true; }
  catch (error) { if (error.code === 'ESRCH') return false; throw error; }
}

// Native NUL-delimited layout used by Apple's ps: adv_cmds/ps/print.c getproclline.
// Values stay in memory; never expose the captured buffer in an error or receipt.
export function processEnvironment(pid, expectedArgc = 3) {
  let raw;
  try {
    raw = execFileSync('python3', ['-c', `
import ctypes, os, sys
lib = ctypes.CDLL(None, use_errno=True)
lib.sysctl.argtypes = [ctypes.POINTER(ctypes.c_int), ctypes.c_uint, ctypes.c_void_p, ctypes.POINTER(ctypes.c_size_t), ctypes.c_void_p, ctypes.c_size_t]
lib.sysctl.restype = ctypes.c_int
size = ctypes.c_size_t(os.sysconf('SC_ARG_MAX'))
buf = ctypes.create_string_buffer(size.value)
mib = (ctypes.c_int * 3)(1, 49, int(sys.argv[1]))
if lib.sysctl(mib, 3, buf, ctypes.byref(size), None, 0) != 0:
    raise OSError(ctypes.get_errno(), 'Cannot read owned process environment')
sys.stdout.buffer.write(buf.raw[:size.value])
`, String(pid)], { timeout: 10_000, maxBuffer: 4 * 1024 * 1024, stdio: ['ignore', 'pipe', 'pipe'] });
  } catch { throw new Error('Cannot capture the owned process environment; runtime left untouched'); }
  assert.ok(raw.length > 4, 'Missing process argument bytes');
  const argc = raw.readInt32LE(0);
  assert.equal(argc, expectedArgc, 'Unexpected owned process argument count');
  let cursor = 4;
  const next = () => {
    const end = raw.indexOf(0, cursor);
    assert.ok(end >= cursor, 'Truncated process argument bytes');
    const value = new TextDecoder('utf-8', { fatal: true }).decode(raw.subarray(cursor, end));
    cursor = end + 1;
    return value;
  };
  next(); // executable path, then NUL padding before argv
  while (cursor < raw.length && raw[cursor] === 0) cursor++;
  for (let i = 0; i < argc; i++) next();
  const env = Object.create(null);
  while (cursor < raw.length && raw[cursor] !== 0) {
    const binding = next();
    const equals = binding.indexOf('=');
    assert.ok(equals > 0, 'Malformed process environment binding');
    const key = binding.slice(0, equals);
    assert.ok(!Object.hasOwn(env, key), 'Duplicate process environment binding');
    env[key] = binding.slice(equals + 1);
  }
  raw.fill(0);
  return env;
}

export async function assertOwnedEndpoint(base) {
  const runtime = JSON.parse(await readFile(resolve('.prometheus/verification/manual-runtime.json'), 'utf8'));
  const launch = JSON.parse(await readFile(resolve('target/prompt-macos-launch.json'), 'utf8'));
  const proxy = JSON.parse(await readFile(resolve('target/manual-identity-proxy/launch.json'), 'utf8'));
  assert.equal(base.href, new URL(runtime.base_url).href);
  assert.equal(base.href, new URL(proxy.base_url).href);
  assert.equal(new URL(proxy.upstream).href, new URL(launch.base_url).href);
  assert.equal(new URL(runtime.runtime_url).href, new URL(launch.base_url).href);
  assert.equal(runtime.runtime_pid, launch.pid);
  assert.equal(runtime.external_identity_pid, proxy.pid);
  const expected = `${proxy.binary} --config ${proxy.config} --upstream ${proxy.upstream} --listen ${new URL(proxy.base_url).host}`;
  assert.equal(execFileSync('ps', ['-p', String(proxy.pid), '-o', 'command='], { encoding: 'utf8', timeout: 10_000 }).trim(), expected);
  for (const [pid, url] of [[proxy.pid, proxy.base_url], [launch.pid, launch.base_url]]) {
    const listener = execFileSync('lsof', ['-nP', '-a', '-p', String(pid), '-iTCP:' + new URL(url).port, '-sTCP:LISTEN', '-Fn'], { encoding: 'utf8', timeout: 10_000 });
    assert.ok(listener.split('\n').includes(`n${new URL(url).host}`), 'Recorded process must own the selected listener');
  }
  return { client_url: base.href, upstream_url: proxy.upstream, proxy_pid: proxy.pid, runtime_pid: launch.pid };
}

// Private, process-local recovery context for a stopped runtime owned by this invocation.
let recoveryContext;

// Host lifecycle adapter: no HTTP authentication, credential issuance or store edits.
export async function restartOwnedUar(record, replacementSha256, replacementConfigSha256, recoverStopped = false) {
  assert.equal(process.platform, 'darwin', 'This lifecycle adapter owns only the recorded Mac runtime');
  const launchPath = resolve('target/prompt-macos-launch.json');
  const runtimePath = resolve('.prometheus/verification/manual-runtime.json');
  const launch = JSON.parse(await readFile(launchPath, 'utf8'));
  const runtime = JSON.parse(await readFile(runtimePath, 'utf8'));
  assert.equal(launch.pid, runtime.runtime_pid);
  assert.ok(Number.isSafeInteger(launch.pid) && launch.pid > 1);
  const expected = `${launch.binary} --config ${launch.config}`;
  let cwd, env;
  if (recoverStopped && !alive(launch.pid)) {
    assert.ok(recoveryContext, 'No verified process-local context for recovery');
    assert.equal(recoveryContext.pid, launch.pid);
    assert.equal(recoveryContext.binary, launch.binary);
    assert.equal(recoveryContext.config, launch.config);
    ({ cwd, env } = recoveryContext);
    record.recovered_stopped_owned_runtime = true;
  } else {
    const actual = execFileSync('ps', ['-p', String(launch.pid), '-o', 'command='], { encoding: 'utf8', timeout: 10_000 }).trim();
    assert.equal(actual, expected, 'Refuse to signal a process that does not match ownership metadata');
    const cwdLines = execFileSync('lsof', ['-a', '-p', String(launch.pid), '-d', 'cwd', '-Fn'], { encoding: 'utf8', timeout: 10_000 }).split('\n');
    cwd = cwdLines.find(line => line.startsWith('n'))?.slice(1);
    env = processEnvironment(launch.pid);
  }
  assert.equal(cwd, dirname(launch.config), 'Preserve the fixture working directory');
  const binarySha = await digest(launch.binary);
  const configSha = await digest(launch.config);
  if (replacementSha256 !== undefined) assert.match(replacementSha256, /^[a-f0-9]{64}$/);
  assert.equal(binarySha, replacementSha256 ?? launch.binary_sha256);
  if (replacementConfigSha256 !== undefined) assert.match(replacementConfigSha256, /^[a-f0-9]{64}$/);
  assert.equal(configSha, replacementConfigSha256 ?? launch.config_sha256);
  // Carry the existing host environment opaquely in memory; never log or persist values.
  assert.equal(env.HOME, process.env.HOME);
  assert.ok(env.PATH);
  assert.equal(env.UAR_STATIC_DIR, launch.static_dir);
  recoveryContext = { pid: launch.pid, binary: launch.binary, config: launch.config, cwd, env };
  const logPath = resolve(dirname(launch.config), `server-conversation-restart-${randomUUID()}.log`);
  const log = await open(logPath, 'wx', 0o600);
  Object.assign(record, { previous_pid: launch.pid, binary_sha256: binarySha, config_sha256: configSha,
    working_directory: cwd, config_path: launch.config, log: logPath, stop_signal: 'SIGTERM', store_preserved: true });
  if (replacementSha256) record.replaced_binary_sha256 = launch.binary_sha256;
  if (replacementConfigSha256) record.replaced_config_sha256 = launch.config_sha256;
  let child;
  try {
    if (!record.recovered_stopped_owned_runtime) process.kill(launch.pid, 'SIGTERM');
    const stopDeadline = Date.now() + 45_000;
    while (alive(launch.pid) && Date.now() < stopDeadline) await delay(250);
    assert.equal(alive(launch.pid), false, 'Owned runtime must stop gracefully; never force-kill');
    record.previous_process_exited = true;
    child = spawn(launch.binary, ['--config', launch.config], { cwd, env, detached: true, stdio: ['ignore', log.fd, log.fd] });
    await new Promise((resolveSpawn, reject) => { child.once('spawn', resolveSpawn); child.once('error', reject); });
    child.unref();
    record.new_pid = child.pid;
    assert.notEqual(child.pid, launch.pid);
    launch.pid = child.pid;
    recoveryContext.pid = child.pid;
    launch.log = logPath;
    launch.status = 'starting';
    launch.binary_sha256 = binarySha;
    launch.config_sha256 = configSha;
    runtime.binary_sha256 = binarySha;
    runtime.runtime_pid = child.pid;
    await writeFile(launchPath, JSON.stringify(launch, null, 2) + '\n');
    await writeFile(runtimePath, JSON.stringify(runtime, null, 2) + '\n');
    const startDeadline = Date.now() + 90_000;
    while (Date.now() < startDeadline) {
      assert.ok(alive(child.pid), `Owned runtime exited; inspect ${logPath}`);
      const started = (await readFile(logPath, 'utf8')).split('\n').some(line => {
        try { return JSON.parse(line).fields?.message === 'Server started'; } catch { return false; }
      });
      if (started) {
        assert.equal(execFileSync('ps', ['-p', String(child.pid), '-o', 'command='], { encoding: 'utf8', timeout: 10_000 }).trim(), expected);
        assert.equal(await digest(launch.binary), binarySha);
        assert.equal(await digest(launch.config), configSha);
        launch.status = 'running';
        await writeFile(launchPath, JSON.stringify(launch, null, 2) + '\n');
        record.new_process_ready = true;
        return;
      }
      await delay(500);
    }
    throw new Error(`Owned runtime startup deadline exceeded; inspect ${logPath}`);
  } finally { await log.close(); }
}
