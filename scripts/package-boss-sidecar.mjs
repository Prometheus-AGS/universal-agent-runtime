import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const [platform, target] = process.argv.slice(2)

const targets = {
  'win32-x64': 'x86_64-pc-windows-msvc',
  'win32-arm64': 'aarch64-pc-windows-msvc',
  'darwin-arm64': 'aarch64-apple-darwin',
  'darwin-x64': 'x86_64-apple-darwin'
}
if (!Object.hasOwn(targets, platform ?? '') || target !== targets[platform]) {
  throw new Error('usage: package-boss-sidecar.mjs <platform> <matching-rust-target>; platforms: win32-x64, win32-arm64, darwin-arm64, darwin-x64')
}

const executable = `uar-sidecar${platform.startsWith('win32-') ? '.exe' : ''}`
const releaseDir = path.join(root, 'target', target, 'release')
const binary = path.join(releaseDir, executable)
if (!existsSync(binary)) throw new Error(`Built sidecar is missing: ${binary}`)

const versionMatch = readFileSync(path.join(root, 'Cargo.toml'), 'utf8').match(
  /\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/
)
if (!versionMatch) throw new Error('Cargo package version is missing')
const releaseTag = process.env.GITHUB_REF_TYPE === 'tag' ? process.env.GITHUB_REF_NAME : undefined
if (releaseTag && !releaseTag.startsWith(`boss-sidecar-${platform}-v${versionMatch[1]}-`)) {
  throw new Error(`Sidecar release tag does not match ${platform} v${versionMatch[1]}: ${releaseTag}`)
}

const output = path.join(root, 'dist', 'boss-sidecar')
const packageName = `uar-sidecar-${platform}`
const packageRoot = path.join(output, packageName)
rmSync(packageRoot, { recursive: true, force: true })
mkdirSync(packageRoot, { recursive: true })
cpSync(binary, path.join(packageRoot, executable))
cpSync(path.join(root, 'src', 'uar', 'runtime', 'matching', 'models'), path.join(packageRoot, 'uar-models'), {
  recursive: true
})
mkdirSync(path.join(packageRoot, 'policies'))
for (const policy of ['default.cedar', 'skill-mutation.cedar', 'tool-approval.cedar']) {
  cpSync(path.join(root, 'policies', policy), path.join(packageRoot, 'policies', policy))
}
cpSync(path.join(root, 'LICENSE'), path.join(packageRoot, 'LICENSE'))

function visit(directory, depth = 0) {
  if (depth > 3) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const filename = path.join(directory, entry.name)
    return entry.isDirectory() ? visit(filename, depth + 1) : [filename]
  })
}

const runtimeLibraries = visit(releaseDir).filter((filename) => /\.(?:dll|dylib|so(?:\.\d+)*)$/i.test(filename))
const libraryHashes = new Map()
for (const filename of runtimeLibraries) {
  const name = path.basename(filename)
  const digest = createHash('sha256').update(readFileSync(filename)).digest('hex')
  const previous = libraryHashes.get(name)
  if (previous && previous !== digest) throw new Error(`Conflicting runtime libraries named ${name}`)
  if (!previous) cpSync(filename, path.join(packageRoot, name))
  libraryHashes.set(name, digest)
}

const files = visit(packageRoot)
  .map((filename) => {
    const relative = path.relative(packageRoot, filename).split(path.sep).join('/')
    return {
      path: relative,
      size: statSync(filename).size,
      sha256: createHash('sha256').update(readFileSync(filename)).digest('hex')
    }
  })
  .sort((left, right) => left.path.localeCompare(right.path))

const source = process.env.GITHUB_SHA || execFileSync('git', ['rev-parse', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim()
const payload = {
  schema: 1,
  name: 'uar-sidecar',
  version: versionMatch[1],
  platform,
  source,
  features: ['server-full'],
  files
}
writeFileSync(path.join(packageRoot, 'payload-manifest.json'), `${JSON.stringify(payload, null, 2)}\n`)

mkdirSync(output, { recursive: true })
const asset = `${packageName}.tar.gz`
const archive = path.join(output, asset)
rmSync(archive, { force: true })
execFileSync('tar', ['czf', archive, '-C', output, packageName], { stdio: 'inherit' })

const packagedFiles = [...files.map((file) => file.path), 'payload-manifest.json']
const record = {
  name: 'uar-sidecar',
  version: versionMatch[1],
  platform,
  source,
  ...(releaseTag ? { releaseTag } : {}),
  features: ['server-full'],
  asset,
  sha256: createHash('sha256').update(readFileSync(archive)).digest('hex'),
  archive: 'tar.gz',
  binaries: packagedFiles
}
writeFileSync(path.join(output, `${packageName}.json`), `${JSON.stringify(record, null, 2)}\n`)

console.log(`Packaged ${asset}: ${packagedFiles.length} files from ${source}`)
