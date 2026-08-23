#!/usr/bin/env node

import {existsSync, readFileSync, readdirSync, statSync} from 'node:fs';
import {basename, dirname, extname, join, relative, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const defaultRoot = resolve(dirname(scriptPath), '..');

const read = (root, path) => readFileSync(join(root, path), 'utf8');

function walk(root, directory) {
  const absolute = join(root, directory);
  if (!existsSync(absolute)) return [];
  const paths = [];
  for (const entry of readdirSync(absolute, {withFileTypes: true})) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) paths.push(...walk(root, relative(root, child)));
    else if (entry.isFile()) paths.push(relative(root, child));
  }
  return paths;
}

function requireText(body, expected, label, failures) {
  if (!body.includes(expected)) failures.push(`${label}: missing ${JSON.stringify(expected)}`);
}

function validateAssetParity(root, failures) {
  const pairs = [
    ['frontend/public/brand/uar-mark-dark.svg', 'website/static/img/brand/uar-mark-dark.svg'],
    ['frontend/public/brand/uar-mark-light.svg', 'website/static/img/brand/uar-mark-light.svg'],
    ['frontend/public/brand/uar-wordmark-dark.svg', 'website/static/img/brand/uar-wordmark-dark.svg'],
    ['frontend/public/brand/uar-wordmark-light.svg', 'website/static/img/brand/uar-wordmark-light.svg'],
    ['frontend/public/brand/uar-favicon-dark.svg', 'website/static/img/brand/uar-favicon.svg'],
  ];

  for (const [source, copy] of pairs) {
    if (!existsSync(join(root, source))) failures.push(`canonical brand asset missing: ${source}`);
    if (!existsSync(join(root, copy))) failures.push(`site brand asset missing: ${copy}`);
    if (existsSync(join(root, source)) && existsSync(join(root, copy)) && read(root, source) !== read(root, copy)) {
      failures.push(`site brand asset differs from canonical source: ${copy}`);
    }
  }
  if (!existsSync(join(root, 'website/static/img/brand/uar-social-card.svg'))) {
    failures.push('site brand asset missing: website/static/img/brand/uar-social-card.svg');
  }
}

function validateTheme(root, failures) {
  const config = read(root, 'website/docusaurus.config.ts');
  const css = read(root, 'website/src/css/custom.css');
  const homeCss = read(root, 'website/src/pages/index.module.css');
  const packageJson = JSON.parse(read(root, 'website/package.json'));

  const exactDependencies = {
    '@easyops-cn/docusaurus-search-local': '0.55.3',
    '@fontsource-variable/geist': '5.3.0',
    '@fontsource/space-grotesk': '5.3.0',
    '@fontsource/jetbrains-mono': '5.3.0',
  };
  for (const [name, version] of Object.entries(exactDependencies)) {
    if (packageJson.dependencies?.[name] !== version) failures.push(`website dependency must be pinned exactly: ${name}@${version}`);
  }

  for (const required of [
    "'@easyops-cn/docusaurus-search-local'",
    "hashed: 'filename'",
    "language: ['en']",
    'indexDocs: true',
    'indexBlog: false',
    'indexPages: true',
    "docsRouteBasePath: '/docs'",
    "defaultMode: 'dark'",
    "favicon: 'img/brand/uar-favicon.svg'",
    "image: 'img/brand/uar-social-card.svg'",
    "theme: {light: 'neutral', dark: 'dark'}",
  ]) requireText(config, required, 'website/docusaurus.config.ts', failures);

  for (const [pattern, label] of [
    [/algolia|docsearch/iu, 'hosted search'],
    [/ask[ -]?ai/iu, 'Ask AI'],
    [/google-analytics|gtag\(|segment|posthog|plausible/iu, 'hosted analytics'],
    [/fonts\.(?:googleapis|gstatic)\.com/iu, 'remote font'],
    [/docusaurus-social-card|favicon\.ico|Dinosaurs are cool/iu, 'stock metadata'],
  ]) if (pattern.test(config)) failures.push(`website/docusaurus.config.ts: forbidden ${label} configuration`);

  for (const required of [
    "@import '@fontsource-variable/geist'",
    "@import '@fontsource/space-grotesk/600.css'",
    "@import '@fontsource/jetbrains-mono/400.css'",
    '--uar-canvas: #f7f7f8',
    '--uar-canvas: #0b0f14',
    '--uar-ember: #ff6a3d',
    '--uar-cyan: #00c2dc',
    ':focus-visible',
    'outline: 3px solid var(--uar-focus)',
  ]) requireText(css, required, 'website/src/css/custom.css', failures);

  for (const [path, body] of [
    ['website/src/css/custom.css', css],
    ['website/src/pages/index.module.css', homeCss],
  ]) {
    if (/#(?:2e8555|25c2a0|3cad6e|205d3b)\b/iu.test(body)) failures.push(`${path}: stock Docusaurus green token is forbidden`);
    if (/(?:linear|radial|conic)-gradient\s*\(/iu.test(body)) failures.push(`${path}: gradients are forbidden`);
    for (const match of body.matchAll(/(?:^|[;{])\s*box-shadow\s*:\s*([^;}]+)/gimu)) {
      if (match[1].trim() !== 'none' && match[1].trim() !== 'none !important') failures.push(`${path}: decorative box shadow is forbidden`);
    }
    for (const match of body.matchAll(/(?:^|[;{])\s*border\s*:\s*([^;}]+)/gimu)) {
      if (!/^(?:0|none|transparent)(?:\s*!important)?$/iu.test(match[1].trim())) failures.push(`${path}: decorative border is forbidden`);
    }
    if (/outline\s*:\s*(?:0|none)/iu.test(body)) failures.push(`${path}: invisible focus outline is forbidden`);
    if (!/@media\s*\(prefers-reduced-motion:\s*reduce\)/iu.test(body)) failures.push(`${path}: reduced-motion handling is missing`);
  }
}

function documentExists(root, route) {
  const id = route.replace(/^\/docs\/?/, '').replace(/\/$/, '');
  if (!id) return true;
  return [
    `website/docs/${id}.md`,
    `website/docs/${id}.mdx`,
    `website/docs/${id}/index.md`,
    `website/docs/${id}/index.mdx`,
    `website/docs/${id}/intro.md`,
    `website/docs/${id}/intro.mdx`,
  ].some((path) => existsSync(join(root, path)));
}

function validateHomepageAndNavigation(root, failures) {
  const homepage = read(root, 'website/src/pages/index.tsx');
  const config = read(root, 'website/docusaurus.config.ts');

  for (const required of [
    'const readerPaths =',
    'const surfaceSteps =',
    'const protocols =',
    'export default function Home()',
    'as="h1"',
    'as="h2"',
    'as="h3"',
    'aria-labelledby=',
    'width="520" height="96"',
    'Capability inversion is the safety boundary.',
    'Profiles are separate contracts.',
  ]) requireText(homepage, required, 'website/src/pages/index.tsx', failures);

  if (/\buse(?:State|Effect|Memo|Callback|Reducer)\b/u.test(homepage)) failures.push('website/src/pages/index.tsx: homepage must remain static');
  if (homepage.indexOf('const readerPaths =') > homepage.indexOf('export default function Home()')) {
    failures.push('website/src/pages/index.tsx: static reader paths must remain module-level');
  }

  const internalRoutes = new Set();
  for (const body of [homepage, config]) {
    for (const match of body.matchAll(/(?:to\s*[=:]\s*|to=)["'](\/docs[^"']*)["']/gu)) internalRoutes.add(match[1]);
  }
  for (const route of internalRoutes) {
    if (!documentExists(root, route)) failures.push(`navigation target does not exist: ${route}`);
  }
}

function validateNoStockContent(root, failures) {
  const stockFiles = new Set([
    'docusaurus-social-card.jpg',
    'docusaurus.png',
    'favicon.ico',
    'logo.svg',
    'undraw_docusaurus_mountain.svg',
    'undraw_docusaurus_react.svg',
    'undraw_docusaurus_tree.svg',
  ]);
  for (const path of walk(root, 'website/static/img')) {
    if (stockFiles.has(basename(path))) failures.push(`stock site asset remains: ${path}`);
  }
  if (walk(root, 'website/src/components/HomepageFeatures').length > 0) failures.push('stock HomepageFeatures component remains');

  for (const path of walk(root, 'website/src')) {
    if (!['.ts', '.tsx', '.css'].includes(extname(path))) continue;
    const body = read(root, path);
    if (/undraw_|HomepageFeatures|Dinosaurs are cool|Docusaurus Tutorial/iu.test(body)) failures.push(`stock tutorial marker remains: ${path}`);
  }
}

export function validateDocumentationBrand({root = defaultRoot} = {}) {
  const resolvedRoot = resolve(root);
  const failures = [];
  const requiredPaths = [
    'website/docusaurus.config.ts',
    'website/package.json',
    'website/src/css/custom.css',
    'website/src/pages/index.tsx',
    'website/src/pages/index.module.css',
  ];
  for (const path of requiredPaths) if (!existsSync(join(resolvedRoot, path)) || !statSync(join(resolvedRoot, path)).isFile()) failures.push(`required branding source missing: ${path}`);
  if (failures.length) return {failures};

  validateAssetParity(resolvedRoot, failures);
  validateTheme(resolvedRoot, failures);
  validateHomepageAndNavigation(resolvedRoot, failures);
  validateNoStockContent(resolvedRoot, failures);
  return {failures};
}

if (process.argv[1] && resolve(process.argv[1]) === scriptPath) {
  const result = validateDocumentationBrand({root: process.argv[2] ?? defaultRoot});
  if (result.failures.length) {
    console.error('Documentation brand validation failed:');
    for (const failure of result.failures) console.error(`- ${failure}`);
    process.exit(1);
  }
  console.log('Documentation brand validation passed.');
}
