import { createHash } from 'node:crypto'
import { execFileSync, spawnSync } from 'node:child_process'
import { closeSync, copyFileSync, cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, openSync, readFileSync, readdirSync, realpathSync, rmSync, writeFileSync } from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = realpathSync(path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..'))
const [source, outputArgument, ...extra] = process.argv.slice(2)
if (extra.length || !/^[a-f0-9]{40}$/.test(source ?? '') || !outputArgument || !path.isAbsolute(outputArgument)) {
  throw new Error('usage: node scripts/build-boss-candidate.mjs <exact-40-hex-source> <new-absolute-output-directory>')
}
if (process.env.CI || process.platform !== 'darwin' || process.arch !== 'arm64') {
  throw new Error('Boss candidates require a local native Apple Silicon host; release CI is not supported')
}
const outputParent = realpathSync(path.dirname(outputArgument))
const output = path.join(outputParent, path.basename(outputArgument))
if (output === root || output.startsWith(`${root}${path.sep}`)) {
  throw new Error('Retain the immutable candidate outside the source checkout')
}
if (existsSync(output)) throw new Error('Candidate output already exists; use a new retained directory')
const git = (directory, ...args) => execFileSync('git', ['-C', directory, ...args], { encoding: 'utf8' }).trim()
function sourceIdentity() {
  if (realpathSync(git(root, 'rev-parse', '--show-toplevel')) !== root ||
      git(root, 'rev-parse', `${source}^{commit}`) !== source) {
    throw new Error('Candidate source must resolve to the exact committed Git object')
  }
  return git(root, 'rev-parse', `${source}^{tree}`)
}
const sourceTree = sourceIdentity()
const committedProducer = execFileSync('git', ['-C', root, 'show', `${source}:scripts/build-boss-candidate.mjs`])
if (!committedProducer.equals(readFileSync(fileURLToPath(import.meta.url)))) {
  throw new Error('Run the candidate producer matching the selected immutable source commit')
}
const scratch = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'uar-boss-candidate-')))
const snapshot = path.join(scratch, 'source')
const targetDirectory = path.join(scratch, 'target')
const platform = 'darwin-arm64'
const target = 'aarch64-apple-darwin'
const features = ['server-full']
const packageName = `uar-sidecar-${platform}`
const packageRoot = path.join(output, packageName)
const command = ['cargo', 'build', '--release', '--locked', '--no-default-features', '--features', 'server-full',
  '--bin', 'uar-sidecar', '--target', target]
const digest = (filename) => createHash('sha256').update(readFileSync(filename)).digest('hex')
let archiveNumber = 0

function exportSnapshot(repository, revision, destination) {
  mkdirSync(destination, { recursive: true })
  const archive = path.join(scratch, `source-${archiveNumber++}.tar`)
  const descriptor = openSync(archive, 'wx')
  try {
    const result = spawnSync('git', ['-C', repository, 'archive', '--format=tar', revision], {
      stdio: ['ignore', descriptor, 'inherit']
    })
    if (result.error) throw result.error
    if (result.status !== 0) throw new Error(`Source snapshot failed: ${revision}`)
  } finally {
    closeSync(descriptor)
  }
  execFileSync('tar', ['xf', archive, '-C', destination], { stdio: 'inherit' })
  rmSync(archive)
  const entries = execFileSync('git', ['-C', repository, 'ls-tree', '-rz', revision], {
    encoding: 'utf8', maxBuffer: 64 * 1024 * 1024
  }).split('\0')
  for (const entry of entries) {
    const match = entry.match(/^160000 commit ([a-f0-9]{40})\t(.+)$/s)
    if (!match) continue
    const [, submoduleRevision, relative] = match
    if (path.posix.isAbsolute(relative) || relative.split('/').some((part) => part === '..')) {
      throw new Error('Unsafe submodule snapshot path')
    }
    const submodule = path.join(repository, ...relative.split('/'))
    if (realpathSync(git(submodule, 'rev-parse', '--show-toplevel')) !== realpathSync(submodule) ||
        git(submodule, 'rev-parse', `${submoduleRevision}^{commit}`) !== submoduleRevision) {
      throw new Error(`Committed submodule object must be available: ${relative}`)
    }
    exportSnapshot(submodule, submoduleRevision, path.join(destination, ...relative.split('/')))
  }
}

function filesIn(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(directory, entry.name)
    const stat = lstatSync(filename)
    if (stat.isDirectory()) return filesIn(filename)
    if (!stat.isFile() || stat.size === 0) throw new Error(`Candidate contains a non-regular or empty file: ${filename}`)
    return [filename]
  })
}

function runtimeLibraries(directory, releaseDirectory, depth = 0) {
  if (depth > 3) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(directory, entry.name)
    if (entry.isDirectory()) return runtimeLibraries(filename, releaseDirectory, depth + 1)
    if (!/\.dylib$/i.test(filename)) return []
    if (!realpathSync(filename).startsWith(`${releaseDirectory}${path.sep}`)) {
      throw new Error(`Runtime library escapes the fresh build directory: ${filename}`)
    }
    return [filename]
  })
}

try {
  exportSnapshot(root, source, snapshot)
  if (sourceIdentity() !== sourceTree) throw new Error('Committed source identity changed while creating the snapshot')
  const version = readFileSync(path.join(snapshot, 'Cargo.toml'), 'utf8')
    .match(/\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/)?.[1]
  if (!version) throw new Error('Candidate Cargo package version is missing')
  mkdirSync(output)
  mkdirSync(packageRoot)
  const log = path.join(packageRoot, 'uar-build.log')
  const descriptor = openSync(log, 'wx')
  try {
    writeFileSync(descriptor, `source ${source}\ntree ${sourceTree}\ncommand ${JSON.stringify(command)}\n`)
    const result = spawnSync(command[0], command.slice(1), {
      cwd: snapshot,
      env: { ...process.env, CARGO_TARGET_DIR: targetDirectory },
      stdio: ['ignore', descriptor, descriptor]
    })
    if (result.error) throw result.error
    if (result.status !== 0) throw new Error(`Candidate build failed; retained log: ${log}`)
  } finally {
    closeSync(descriptor)
  }
  if (sourceIdentity() !== sourceTree) throw new Error('Committed source identity changed during candidate build')
  const releaseDirectory = path.join(targetDirectory, target, 'release')
  const executable = path.join(releaseDirectory, 'uar-sidecar')
  if (!lstatSync(executable).isFile()) throw new Error('Fresh candidate executable is missing')
  copyFileSync(executable, path.join(packageRoot, 'uar-sidecar'))
  cpSync(path.join(snapshot, 'src', 'uar', 'runtime', 'matching', 'models'), path.join(packageRoot, 'uar-models'), { recursive: true })
  mkdirSync(path.join(packageRoot, 'policies'))
  for (const policy of ['default.cedar', 'skill-mutation.cedar', 'tool-approval.cedar']) {
    copyFileSync(path.join(snapshot, 'policies', policy), path.join(packageRoot, 'policies', policy))
  }
  copyFileSync(path.join(snapshot, 'LICENSE'), path.join(packageRoot, 'LICENSE'))
  const libraries = new Map()
  for (const filename of runtimeLibraries(releaseDirectory, releaseDirectory)) {
    const name = path.basename(filename)
    const hash = digest(filename)
    if (libraries.has(name) && libraries.get(name) !== hash) throw new Error(`Conflicting runtime library: ${name}`)
    if (!libraries.has(name)) copyFileSync(filename, path.join(packageRoot, name))
    libraries.set(name, hash)
  }
  const provenance = {
    schema: 1, producer: 'scripts/build-boss-candidate.mjs', sourceMode: 'immutable-git-objects', source, sourceTree, target, features,
    lifecycleControl: { transport: 'inherited-stdin', version: 1, responsePrefix: 'LIFECYCLE:' },
    build: {
      command, isolatedTarget: true, snapshot: 'git-archive-with-submodules',
      log: { path: 'uar-build.log', sha256: digest(log) },
      executable: { path: 'uar-sidecar', sha256: digest(path.join(packageRoot, 'uar-sidecar')) }
    }
  }
  const files = filesIn(packageRoot).map((filename) => ({
    path: path.relative(packageRoot, filename).split(path.sep).join('/'),
    size: lstatSync(filename).size, sha256: digest(filename)
  })).sort((left, right) => left.path.localeCompare(right.path))
  const identity = { name: 'uar-sidecar', version, platform, source, features, provenance }
  writeFileSync(path.join(packageRoot, 'payload-manifest.json'), `${JSON.stringify({ schema: 1, ...identity, files }, null, 2)}\n`, { flag: 'wx' })
  const asset = `${packageName}.tar.gz`
  const archive = path.join(output, asset)
  execFileSync('tar', ['czf', archive, '-C', output, packageName], { stdio: 'inherit' })
  if (sourceIdentity() !== sourceTree) throw new Error('Committed source identity changed before candidate record publication')
  const record = { ...identity, asset, sha256: digest(archive), archive: 'tar.gz',
    binaries: [...files.map((entry) => entry.path), 'payload-manifest.json'] }
  writeFileSync(path.join(output, `${packageName}.json`), `${JSON.stringify(record, null, 2)}\n`, { flag: 'wx' })
  process.stdout.write(`Retained Boss candidate: ${path.join(output, `${packageName}.json`)}\n`)
} finally {
  rmSync(scratch, { recursive: true, force: true })
}
