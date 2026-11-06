// Bounded metadata capture only; not release authority or runtime acceptance.
// Future argv: node <this-file> ABSOLUTE_REPOSITORY ABSOLUTE_CREATE_ONLY_OUTPUT ABSOLUTE_LIMITS_JSON.
// Do not execute without the operator's named whole-phase command permission.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, lstatSync, readFileSync, readlinkSync, realpathSync, writeFileSync } from 'node:fs';
import { isAbsolute, relative, resolve, dirname } from 'node:path';
const [repository, output, limitsPath] = process.argv.slice(2);
assert.ok(limitsPath && isAbsolute(limitsPath));
const limits = JSON.parse(readFileSync(limitsPath, 'utf8'));
for (const key of ['maxFileBytes', 'maxTotalBytes', 'maxEntries', 'gitOutputBytes', 'gitTimeoutMs']) assert.ok(Number.isSafeInteger(limits[key]) && limits[key] > 0);
let totalBytes = 0, entryCount = 0;
assert.ok(repository && output && isAbsolute(repository) && isAbsolute(output));
const root = realpathSync(repository), target = resolve(output);
assert.ok(/^[\x20-\x7e]+$/.test(root), 'Non-ASCII source roots are unsupported by the existing build preimage encoding');
const rel = relative(root, target);
assert.ok(rel.startsWith(`target/`) || rel.startsWith('../'), 'Output must be ignored target/ or outside source root');
assert.equal(realpathSync(dirname(target)), dirname(target), 'Output parent must already be canonical and owned');
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const blobOid = bytes => createHash('sha1').update(Buffer.from(`blob ${bytes.length}\0`)).update(bytes).digest('hex');
const canonical = value => JSON.stringify(value, (_key, item) => item && typeof item === 'object' && !Array.isArray(item)
  ? Object.fromEntries(Object.entries(item).sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0)) : item);
const git = (cwd, ...args) => execFileSync('git', ['-C', cwd, ...args], { encoding: 'utf8', maxBuffer: limits.gitOutputBytes, timeout: limits.gitTimeoutMs }).trimEnd();
function capture(directory, prefix = '') {
  assert.equal(git(directory, 'rev-parse', '--show-toplevel'), directory, 'Missing recursive repository checkout');
  assert.equal(git(directory, 'status', '--porcelain', '--untracked-files=all'), '', 'Clean committed root/submodules required');
  const gitDirectory = git(directory, 'rev-parse', '--git-dir');
  assert.ok(!existsSync(resolve(directory, gitDirectory, 'MERGE_HEAD')), 'Pending merge is not a committed release');
  const head = git(directory, 'rev-parse', 'HEAD'), tree = git(directory, 'rev-parse', 'HEAD^{tree}');
  const entries = git(directory, 'ls-tree', '-rz', 'HEAD').split('\0').filter(Boolean);
  const files = [], submodules = [];
  for (const entry of entries) {
    assert.ok(++entryCount <= limits.maxEntries, 'Owner source entry budget exhausted');
    const split = entry.indexOf('\t'); assert.ok(split > 0);
    const [mode, type, oid] = entry.slice(0, split).split(' '), name = entry.slice(split + 1);
    assert.ok(name && /^[\x20-\x7e]+$/.test(name) && !isAbsolute(name) && !name.split('/').includes('..'), 'Unsupported source path encoding');
    const path = resolve(directory, name), scoped = prefix ? `${prefix}/${name}` : name;
    if (mode === '160000' && type === 'commit') {
      assert.ok(lstatSync(path).isDirectory()); const nested = capture(realpathSync(path), scoped);
      assert.equal(nested.head, oid, 'Gitlink and recursive checkout differ');
      files.push(...nested.files); submodules.push({ path: scoped, gitlink: oid, head_tree: nested.tree }, ...nested.submodules); continue;
    }
    const metadata = lstatSync(path);
    if (mode === '120000' && type === 'blob') {
      assert.ok(metadata.isSymbolicLink()); const link = readlinkSync(path, { encoding: 'buffer' });
      assert.equal(blobOid(link), oid, 'Symlink bytes differ from committed blob');
      assert.ok(link.length <= limits.maxFileBytes); totalBytes += link.length; assert.ok(totalBytes <= limits.maxTotalBytes);
      assert.ok(Buffer.from(link.toString('utf8')).equals(link), 'Non-UTF8 symlink target is unsupported');
      const destination = resolve(dirname(path), link.toString('utf8'));
      assert.ok(destination.startsWith(`${root}/`) && existsSync(destination) && realpathSync(destination).startsWith(`${root}/`), 'Unowned/external/dangling symlink source is unsupported');
      files.push({ path: scoped, absolute_path: path, kind: 'symlink', mode, git_oid: oid, target: link.toString('utf8'), sha256: sha(link) });
    } else {
      assert.ok(type === 'blob' && ['100644', '100755'].includes(mode) && metadata.isFile(), 'Unsupported/special source input');
      assert.ok(metadata.size <= limits.maxFileBytes);
      const bytes = readFileSync(path); assert.equal(blobOid(bytes), oid, 'Source bytes differ from committed blob'); totalBytes += bytes.length; assert.ok(totalBytes <= limits.maxTotalBytes);
      files.push({ path: scoped, absolute_path: path, kind: 'regular', mode, git_oid: oid, bytes: bytes.length, sha256: sha(bytes) });
    }
  }
  assert.equal(git(directory, 'status', '--porcelain', '--untracked-files=all'), '');
  assert.equal(git(directory, 'rev-parse', 'HEAD'), head);
  return { head, tree, files: files.sort((a, b) => a.path < b.path ? -1 : 1), submodules: submodules.sort((a, b) => a.path < b.path ? -1 : 1) };
}
const first = capture(root);
// A second byte capture detects concurrent source changes even when a later Git
// status would be clean again. Both captures count against the explicit budget.
const second = capture(root);
assert.equal(canonical(second), canonical(first), 'Source changed during inventory capture');
if (rel.startsWith('target/')) assert.ok(/^\/?target\/?$/m.test(readFileSync(resolve(root, '.gitignore'), 'utf8')), 'Selected target output is not declared ignored');
const identity = { head: first.head, head_tree: first.tree, merge_head: null, working_index_status: '',
  submodules: first.submodules, files: first.files };
const regular = first.files.filter(row => row.kind === 'regular').map(row => ({ path: row.absolute_path, sha256: `sha256:${row.sha256}` }))
  .sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
const inventory = { schema: 'uar.production-readiness.source-inventory/v1', classification: 'SOURCE_IDENTITY_ONLY_NOT_VALIDATION',
  root, created_at: new Date().toISOString(), identity, source_identity_sha256: sha(canonical(identity)),
  source_identity: `sha256:${sha(canonical(regular))}`,
  files: first.files.map(row => ({ path: row.absolute_path, kind: row.kind, sha256: `sha256:${row.sha256}`, ...(row.target ? { target: row.target } : {}) })),
  limits, source_entry_count: entryCount / 2, captured_bytes_both_passes: totalBytes,
  release_authority: 'existing release provenance/checklist only; this is byte inventory, not build/runtime acceptance' };
writeFileSync(target, `${JSON.stringify(inventory, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
assert.equal(git(root, 'status', '--porcelain', '--untracked-files=all'), '', 'Inventory output dirtied committed source');
console.log(JSON.stringify({ inventory: target, sha256: sha(readFileSync(target)), sourceCommit: first.head,
  sourceInventoryDigest: inventory.source_identity_sha256, status: 'INVENTORIED_NOT_QUALIFIED' }));
