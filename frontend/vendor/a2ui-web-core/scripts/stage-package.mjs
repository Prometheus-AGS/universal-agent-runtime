import {access, cp, mkdir, readFile, stat, writeFile} from 'node:fs/promises';
import {extname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const [directory] = process.argv.slice(2);
if (!directory) {
  throw new Error('Pass a new staging directory for the compiled package.');
}

const source = fileURLToPath(new URL('../', import.meta.url));
const destination = resolve(directory);
const manifest = JSON.parse(await readFile(resolve(source, 'package.json'), 'utf8'));
const compiledPath = (path, extension) => path.replace(/\.ts$/, extension);
manifest.main = compiledPath(manifest.main, '.js');
manifest.types = compiledPath(manifest.types, '.d.ts');
manifest.sideEffects = manifest.sideEffects.map(path => compiledPath(path, '.js'));
for (const entry of Object.values(manifest.exports)) {
  entry.default = compiledPath(entry.default, '.js');
  entry.types = compiledPath(entry.types, '.d.ts');
}
delete manifest.scripts;
delete manifest.private;
manifest.files = ['src', 'ORIGIN.md'];

await access(resolve(source, 'dist/index.js'));
await access(resolve(source, 'dist/index.d.ts'));
await mkdir(destination);
await cp(resolve(source, 'dist'), resolve(destination, 'src'), {recursive: true});
await cp(resolve(source, 'src'), resolve(destination, 'src'), {
  recursive: true,
  filter: async entry => (await stat(entry)).isDirectory() || ['.json', '.css', '.svg'].includes(extname(entry)),
});
for (const file of ['LICENSE', 'README.md', 'CHANGELOG.md', 'ORIGIN.md']) {
  await cp(resolve(source, file), resolve(destination, file));
}
await writeFile(resolve(destination, 'package.json'), JSON.stringify(manifest, null, 2) + '\n');
