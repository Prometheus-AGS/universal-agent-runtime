import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const [harness, ...forwarded] = process.argv.slice(2);
const binaries = { codex: 'codex', claude: 'claude', kimi: 'kimi', opencode: 'opencode', minimax: 'mcode' };

if (!harness || harness === '--help' || harness === '-h') {
  console.log('Usage: node .agent-team/uar-core/launch.mjs <codex|claude|kimi|opencode|minimax> [native CLI arguments]');
  console.log('Runs in the UAR project. Kimi defaults to kimi-code/k3; MiniMax uses the project .minimax data directory.');
} else if (!Object.hasOwn(binaries, harness)) {
  console.error(`Unknown harness: ${harness}`);
  process.exitCode = 2;
} else {
  const args = [...forwarded];
  if (harness === 'codex') args.unshift('-C', root);
  if (harness === 'kimi' && !args.some((arg) => arg === '--model' || arg.startsWith('--model=') || arg === '-m' || /^-m[^-]/u.test(arg))) {
    args.unshift('--model', 'kimi-code/k3');
  }
  const env = harness === 'minimax'
    ? { ...process.env, MINIMAX_DATA_DIR: fileURLToPath(new URL('../../.minimax/', import.meta.url)) }
    : process.env;
  const child = spawn(binaries[harness], args, { cwd: root, env, stdio: 'inherit', shell: false });
  child.on('error', (error) => {
    console.error(error.code === 'ENOENT'
      ? `${binaries[harness]} is not installed on PATH. This launcher does not install CLI tools.`
      : `Could not start ${binaries[harness]} (${error.code ?? 'launch error'}).`);
    process.exitCode = 1;
  });
  child.on('exit', (code) => { process.exitCode = code ?? 1; });
}
