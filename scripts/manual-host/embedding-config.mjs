import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { restartOwnedUar } from '../behaviors/support/restart-owned-uar.mjs';

const sha = bytes => createHash('sha256').update(bytes).digest('hex');

// Host provisioning only: provider credentials never enter HTTP test requests or evidence.
export async function prepareEmbeddingHost(state, id, record) {
  const launch = JSON.parse(await readFile('target/prompt-macos-launch.json', 'utf8'));
  const original = await readFile(launch.config);
  assert.equal(sha(original), launch.config_sha256);
  let key;
  try {
    key = execFileSync('python3', ['-c', `
import pathlib, re, shlex
lines = pathlib.Path.home().joinpath('.bash_profile').read_text().splitlines()
values = []
for line in lines:
    if re.match(r'^\\s*(?:export\\s+)?OPENAI_API_KEY=', line):
        parts = shlex.split(line, comments=True)
        item = next(p for p in parts if p.startswith('OPENAI_API_KEY='))
        value = item.split('=', 1)[1]
        assert value and '$' not in value and chr(96) not in value
        values.append(value)
assert values
print(values[-1], end='')
`], { encoding: 'utf8', timeout: 10_000, stdio: ['ignore', 'pipe', 'pipe'] });
  } catch { throw new Error('Cannot read the configured literal OpenAI provider key'); }
  const config = JSON.parse(original);
  assert.ok(config.llm?.embedding);
  config.llm.embedding.base_url = 'https://api.openai.com/v1';
  config.llm.embedding.api_key = key;
  key = undefined;
  const next = Buffer.from(JSON.stringify(config, null, 2) + '\n');
  Object.assign(state, { path: launch.config, original, originalSha: sha(original), preparedSha: sha(next),
    backup: join(dirname(launch.config), `embedding-regeneration-${id}.original.private.json`) });
  await writeFile(state.backup, original, { flag: 'wx', mode: 0o600 });
  state.changed = true;
  await writeFile(state.path, next, { mode: 0o600 });
  await restartOwnedUar(record, undefined, state.preparedSha);
}

export async function restoreEmbeddingHost(state, record) {
  if (!state.changed) return;
  assert.equal(sha(await readFile(state.path)), state.preparedSha, 'Do not overwrite an outside host configuration edit');
  await writeFile(state.path, state.original, { mode: 0o600 });
  await restartOwnedUar(record, undefined, state.originalSha, true);
  assert.equal(sha(await readFile(state.path)), state.originalSha, true);
  state.changed = false;
}
