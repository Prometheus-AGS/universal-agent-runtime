#!/usr/bin/env node

import {cpSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {dirname, resolve} from 'node:path';
import {validateDocumentationBrand} from './validate-documentation-brand.mjs';

const scriptPath = fileURLToPath(import.meta.url);
const sourceRoot = resolve(dirname(scriptPath), '..');

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'uar-doc-brand-'));
  for (const path of ['frontend/public/brand', 'website/docs', 'website/src', 'website/static/img/brand']) {
    mkdirSync(join(root, path), {recursive: true});
    cpSync(join(sourceRoot, path), join(root, path), {recursive: true});
  }
  for (const path of ['website/docusaurus.config.ts', 'website/package.json']) {
    mkdirSync(join(root, dirname(path)), {recursive: true});
    cpSync(join(sourceRoot, path), join(root, path));
  }
  return root;
}

function replace(root, path, from, to) {
  const absolute = join(root, path);
  const body = readFileSync(absolute, 'utf8');
  if (!body.includes(from)) throw new Error(`fixture mutation source missing: ${from}`);
  writeFileSync(absolute, body.replace(from, to));
}

function expectFailure(name, mutate, expected) {
  const root = fixture();
  try {
    mutate(root);
    const result = validateDocumentationBrand({root});
    if (!result.failures.some((failure) => failure.includes(expected))) {
      throw new Error(`${name}: expected ${JSON.stringify(expected)}, observed ${JSON.stringify(result.failures)}`);
    }
    console.log(`PASS negative control: ${name}`);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

expectFailure('hosted search rejected', (root) => {
  replace(root, 'website/docusaurus.config.ts', "themes: [", "algolia: {},\n  themes: [");
}, 'hosted search');

expectFailure('remote font rejected', (root) => {
  replace(root, 'website/docusaurus.config.ts', "title: 'Universal Agent Runtime'", "title: 'Universal Agent Runtime',\n  customFields: {font: 'https://fonts.googleapis.com/css2'}");
}, 'remote font');

expectFailure('stock token rejected', (root) => {
  replace(root, 'website/src/css/custom.css', '--uar-ember: #b93c1c', '--uar-ember: #2e8555');
}, 'stock Docusaurus green token is forbidden');

expectFailure('gradient rejected', (root) => {
  replace(root, 'website/src/pages/index.module.css', '.page {', '.page { background-image: linear-gradient(red, blue);');
}, 'gradients are forbidden');

expectFailure('decorative shadow rejected', (root) => {
  replace(root, 'website/src/pages/index.module.css', '.page {', '.page { box-shadow: 0 2px 8px #000;');
}, 'decorative box shadow is forbidden');

expectFailure('decorative border rejected', (root) => {
  replace(root, 'website/src/pages/index.module.css', '.page {', '.page { border: 1px solid red;');
}, 'decorative border is forbidden');

expectFailure('invisible focus rejected', (root) => {
  replace(root, 'website/src/css/custom.css', 'outline: 3px solid var(--uar-focus)', 'outline: none');
}, 'invisible focus outline is forbidden');

expectFailure('missing reduced motion rejected', (root) => {
  replace(root, 'website/src/pages/index.module.css', '@media (prefers-reduced-motion: reduce)', '@media (prefers-color-scheme: purple)');
}, 'reduced-motion handling is missing');

expectFailure('asset drift rejected', (root) => {
  writeFileSync(join(root, 'website/static/img/brand/uar-mark-dark.svg'), '<svg/>\n');
}, 'differs from canonical source');

expectFailure('missing route rejected', (root) => {
  replace(root, 'website/src/pages/index.tsx', "to: '/docs/intro'", "to: '/docs/does-not-exist'");
}, 'navigation target does not exist');

expectFailure('stock tutorial marker rejected', (root) => {
  writeFileSync(join(root, 'website/src/stock.tsx'), 'export const title = "Dinosaurs are cool";\n');
}, 'stock tutorial marker remains');

const current = validateDocumentationBrand({root: sourceRoot});
if (current.failures.length) throw new Error(`current brand source failed: ${JSON.stringify(current.failures)}`);
console.log('PASS positive control: current UAR documentation brand source');
