#!/usr/bin/env node
import { execFileSync } from 'node:child_process';
import { existsSync } from 'node:fs';

const configPath = '../.vale.ini';
const targets = ['docs/', '../docs/adr', '../README.md'];

function hasVale() {
  try {
    execFileSync('vale', ['--version'], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

if (!hasVale()) {
  console.warn(' Vale CLI is not installed. Skipping prose lint.');
  console.warn(' Install Vale from https://vale.sh or run:');
  console.warn('   brew install vale');
  console.warn(' To enable docs:lint in CI, the workflow installs Vale via the official installer.');
  process.exit(0);
}

try {
  execFileSync('vale', ['--config', configPath, ...targets], { stdio: 'inherit' });
} catch (error) {
  process.exit(error.status ?? 1);
}
