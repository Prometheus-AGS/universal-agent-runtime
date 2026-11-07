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
// SHA-256 of compact JSON [relative path, Git blob OID, link target] rows
// sorted by path. These pinned metadata links contribute their own bytes only.
const reviewedDanglingGroups = new Map([
  ['crates/prometheus-skill-system', {
    head: 'f4bef4556a83ac22bf204bf69a01ddf0f9e007f2',
    count: 2, digest: '0252e8cef71026040678de563c076e2f3efcac4390e33e3fecb070b063244908',
  }],
  ['models.dev', {
    head: '4a0154da2410bca18e97be812fecb9e8f85466e8',
    count: 32, digest: '163311813b9ba14794c114959943c10b7752ada4790073835194e0d85ad83608',
  }],
]);

function linkDestination(directory, path, linkTarget) {
  const parts = linkTarget.split('/');
  let leadingParents = 0;
  while (parts[leadingParents] === '..') leadingParents++;
  const parent = dirname(path), parentPath = relative(directory, parent);
  const parentDepth = parentPath ? parentPath.split('/').length : 0;
  assert.ok(leadingParents <= parentDepth, 'Symlink target leaves its owning boundary');
  assert.ok(!isAbsolute(linkTarget) && leadingParents < parts.length
    && parts.slice(leadingParents).every(part => part && part !== '.' && part !== '..'), 'Ambiguous symlink target');
  const destination = resolve(parent, linkTarget);
  assert.ok(destination.startsWith(`${directory}/`), 'Unowned/external symlink source is unsupported');
  return destination;
}

function inspectLinkDestination(destination) {
  const parts = relative(root, destination).split('/');
  let current = root;
  for (const [index, part] of parts.entries()) {
    current = resolve(current, part);
    let metadata;
    try {
      metadata = lstatSync(current);
    } catch (error) {
      if (error.code === 'ENOENT') return 'missing';
      throw error;
    }
    if (index < parts.length - 1) {
      assert.ok(metadata.isDirectory(), 'Symlink target passes through a non-directory or symlink');
    } else {
      if (metadata.isSymbolicLink()) return 'symlink';
      if (metadata.isDirectory()) return 'directory';
      assert.ok(metadata.isFile(), 'Unsupported symlink target type');
      return 'regular';
    }
  }
  assert.fail('Empty symlink destination');
}

function capture(directory, prefix = '') {
  assert.equal(git(directory, 'rev-parse', '--show-toplevel'), directory, 'Missing recursive repository checkout');
  assert.equal(git(directory, 'status', '--porcelain', '--untracked-files=all'), '', 'Clean committed root/submodules required');
  const gitDirectory = git(directory, 'rev-parse', '--git-dir');
  assert.ok(!existsSync(resolve(directory, gitDirectory, 'MERGE_HEAD')), 'Pending merge is not a committed release');
  const head = git(directory, 'rev-parse', 'HEAD'), tree = git(directory, 'rev-parse', 'HEAD^{tree}');
  const entries = git(directory, 'ls-tree', '-rz', 'HEAD').split('\0').filter(Boolean);
  const files = [], submodules = [];
  const reviewedGroup = reviewedDanglingGroups.get(prefix);
  const dangling = [];
  for (const entry of entries) {
    assert.ok(++entryCount <= limits.maxEntries, 'Owner source entry budget exhausted');
    const split = entry.indexOf('\t'); assert.ok(split > 0);
    const [mode, type, oid] = entry.slice(0, split).split(' '), name = entry.slice(split + 1);
    assert.ok(name && /^[\x20-\x7e]+$/.test(name) && !isAbsolute(name) && !name.split('/').includes('..'), 'Unsupported source path encoding');
    const path = resolve(directory, name), scoped = prefix ? `${prefix}/${name}` : name;
    if (mode === '160000' && type === 'commit') {
      assert.ok(lstatSync(path).isDirectory());
      assert.equal(realpathSync.native(path), path, 'Recursive checkout path must be canonical');
      const nested = capture(path, scoped);
      assert.equal(nested.head, oid, 'Gitlink and recursive checkout differ');
      files.push(...nested.files); submodules.push({ path: scoped, gitlink: oid, head_tree: nested.tree }, ...nested.submodules); continue;
    }
    const metadata = lstatSync(path);
    if (mode === '120000' && type === 'blob') {
      assert.ok(metadata.isSymbolicLink()); const link = readlinkSync(path, { encoding: 'buffer' });
      assert.equal(blobOid(link), oid, 'Symlink bytes differ from committed blob');
      assert.ok(link.length <= limits.maxFileBytes); totalBytes += link.length; assert.ok(totalBytes <= limits.maxTotalBytes);
      const linkTarget = link.toString('utf8');
      assert.ok(Buffer.from(linkTarget).equals(link), 'Non-UTF8 symlink target is unsupported');
      assert.equal(realpathSync.native(dirname(path)), dirname(path), 'Symlink source parent must be canonical');
      const destination = linkDestination(root, path, linkTarget);
      const targetKind = inspectLinkDestination(destination);
      let resolution = 'resolved';
      if (targetKind === 'missing') {
        assert.equal(head, reviewedGroup?.head, 'Dangling symlink outside a reviewed pinned checkout');
        linkDestination(directory, path, linkTarget);
        dangling.push([name, oid, linkTarget]);
        resolution = 'reviewed_dangling';
      }
      files.push({ path: scoped, absolute_path: path, kind: 'symlink', mode, git_oid: oid,
        target: linkTarget, sha256: sha(link), resolution, target_kind: targetKind });
    } else {
      assert.ok(type === 'blob' && ['100644', '100755'].includes(mode) && metadata.isFile(), 'Unsupported/special source input');
      assert.ok(metadata.size <= limits.maxFileBytes);
      const bytes = readFileSync(path); assert.equal(blobOid(bytes), oid, 'Source bytes differ from committed blob'); totalBytes += bytes.length; assert.ok(totalBytes <= limits.maxTotalBytes);
      files.push({ path: scoped, absolute_path: path, kind: 'regular', mode, git_oid: oid, bytes: bytes.length, sha256: sha(bytes) });
    }
  }
  if (reviewedGroup?.head === head) {
    dangling.sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0);
    assert.equal(dangling.length, reviewedGroup.count, 'Reviewed dangling symlink count changed');
    assert.equal(sha(JSON.stringify(dangling)), reviewedGroup.digest, 'Reviewed dangling symlink identities changed');
  }
  assert.equal(git(directory, 'status', '--porcelain', '--untracked-files=all'), '');
  assert.equal(git(directory, 'rev-parse', 'HEAD'), head);
  return { head, tree, files: files.sort((a, b) => a.path < b.path ? -1 : 1), submodules: submodules.sort((a, b) => a.path < b.path ? -1 : 1) };
}

function captureSource() {
  const source = capture(root);
  const tracked = new Map(source.files.map(row => [row.absolute_path, row]));
  const directories = new Set([root]);
  const trackedParents = [...source.files.map(row => dirname(row.absolute_path)),
    ...source.submodules.map(row => `${root}/${row.path}`)];
  for (let parent of trackedParents) {
    while (!directories.has(parent)) {
      directories.add(parent);
      parent = dirname(parent);
    }
  }
  // A leaf may be another symlink, but every hop and the final target must
  // belong to the already captured recursive Git tree.
  for (const row of source.files) {
    if (row.kind !== 'symlink' || row.resolution !== 'resolved') continue;
    const seen = new Set();
    let link = row;
    while (true) {
      assert.ok(!seen.has(link.absolute_path), 'Symlink cycle');
      seen.add(link.absolute_path);
      const destination = resolve(dirname(link.absolute_path), link.target);
      if (link.target_kind === 'symlink') {
        link = tracked.get(destination);
        assert.ok(link?.kind === 'symlink' && link.resolution === 'resolved', 'Symlink hop must be a tracked resolved link');
        continue;
      }
      if (link.target_kind === 'directory') {
        assert.ok(directories.has(destination), 'Symlink directory target must belong to the tracked tree');
      } else {
        assert.equal(tracked.get(destination)?.kind, 'regular', 'Symlink file target must be tracked');
      }
      assert.equal(realpathSync.native(row.absolute_path), destination, 'Recorded and actual symlink targets disagree');
      row.canonical_target = relative(root, destination);
      break;
    }
  }
  return source;
}

const first = captureSource();
// A second byte capture detects concurrent source changes even when a later Git
// status would be clean again. Both captures count against the explicit budget.
const second = captureSource();
assert.equal(canonical(second), canonical(first), 'Source changed during inventory capture');
if (rel.startsWith('target/')) assert.ok(/^\/?target\/?$/m.test(readFileSync(resolve(root, '.gitignore'), 'utf8')), 'Selected target output is not declared ignored');
const identity = { head: first.head, head_tree: first.tree, merge_head: null, working_index_status: '',
  submodules: first.submodules, files: first.files };
const regular = first.files.filter(row => row.kind === 'regular').map(row => ({ path: row.absolute_path, sha256: `sha256:${row.sha256}` }))
  .sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
const inventory = { schema: 'uar.production-readiness.source-inventory/v1', classification: 'SOURCE_IDENTITY_ONLY_NOT_VALIDATION',
  root, created_at: new Date().toISOString(), identity, source_identity_sha256: sha(canonical(identity)),
  source_identity: `sha256:${sha(canonical(regular))}`,
  files: first.files.map(row => ({ path: row.absolute_path, kind: row.kind, sha256: `sha256:${row.sha256}`,
    ...(row.kind === 'symlink' ? { target: row.target, resolution: row.resolution, canonical_target: row.canonical_target } : {}) })),
  limits, source_entry_count: entryCount / 2, captured_bytes_both_passes: totalBytes,
  release_authority: 'existing release provenance/checklist only; this is byte inventory, not build/runtime acceptance' };
writeFileSync(target, `${JSON.stringify(inventory, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
assert.equal(git(root, 'status', '--porcelain', '--untracked-files=all'), '', 'Inventory output dirtied committed source');
console.log(JSON.stringify({ inventory: target, sha256: sha(readFileSync(target)), sourceCommit: first.head,
  sourceInventoryDigest: inventory.source_identity_sha256, status: 'INVENTORIED_NOT_QUALIFIED' }));
