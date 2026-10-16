import { after, test } from 'node:test';
import assert from 'node:assert/strict';
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, truncateSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const root = mkdtempSync(join(tmpdir(), 'wasm-budget-'));
after(() => rmSync(root, { recursive: true, force: true }));
const script = new URL('./wasm-budget.mjs', import.meta.url);
function fixture(name, size) {
  const path = join(root, name);
  writeFileSync(path, '');
  truncateSync(path, size);
  return path;
}
const limit = 8_000_000;
const exact = fixture('exact.wasm', limit);
const over = fixture('over.wasm', limit + 1);
const missing = join(root, 'missing.wasm');
function run(engine = exact, experiments = exact) {
  return spawnSync(process.execPath, [script.pathname, engine, '--experiments', experiments], { encoding: 'utf8' });
}
test('accepts both modules exactly at the raw byte ceiling', () => {
  const result = run();
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /engine bundle 8000000 bytes/);
  assert.match(result.stdout, /experiments bundle 8000000 bytes/);
});
for (const module of ['engine', 'experiments']) {
  test(`rejects ${module} one byte over independently`, () => {
    const result = module === 'engine' ? run(over) : run(exact, over);
    assert.equal(result.status, 1);
    assert.match(result.stderr, new RegExp(`${module} bundle.*8000001 bytes.*8000000`));
  });
  test(`names a missing ${module} artifact and its byte ceiling`, () => {
    const result = module === 'engine' ? run(missing) : run(exact, missing);
    assert.equal(result.status, 1);
    assert.match(result.stderr, new RegExp(`${module} bundle.*missing.*actual bytes unavailable.*8000000`));
  });
}
// Exercise npm's ordinary lifecycle in an isolated miniature web tree. Only the
// compiler is stubbed; the actual postbuild gate sees real sparse file sizes.
for (const module of ['engine', 'experiments']) test(`ordinary wasm:${module} rejects an oversized release`, () => {
  const project = join(root, module);
  mkdirSync(join(project, 'scripts'), { recursive: true });
  mkdirSync(join(project, 'bin'));
  mkdirSync(join(project, 'src', module === 'engine' ? 'wasm-pkg' : 'experiments-pkg'), { recursive: true });
  copyFileSync(new URL('../package.json', import.meta.url), join(project, 'package.json'));
  copyFileSync(script, join(project, 'scripts', 'wasm-budget.mjs'));
  writeFileSync(join(project, 'bin', 'wasm-pack'), '#!/bin/sh\nexit 0\n', { mode: 0o755 });
  const artifact = join(project, 'src', module === 'engine' ? 'wasm-pkg/sugarscape_bg.wasm' : 'experiments-pkg/experiments_bg.wasm');
  writeFileSync(artifact, '');
  truncateSync(artifact, limit + 1);
  const result = spawnSync('npm', ['run', `wasm:${module}`], { cwd: project, encoding: 'utf8', env: { ...process.env, PATH: `${join(project, 'bin')}:${process.env.PATH}` } });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stderr, new RegExp(`${module} bundle.*8000001 bytes.*8000000`));
});
test('legacy positional engine override works for a single-package check', () => {
  const result = spawnSync(process.execPath, [script.pathname, exact, '--only', 'engine'], { encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /engine bundle 8000000 bytes/);
});
test('ordinary production build rejects experiments overflow after an engine success', () => {
  const project = join(root, 'full-build');
  for (const directory of ['scripts', 'bin', 'src/wasm-pkg', 'src/experiments-pkg']) mkdirSync(join(project, directory), { recursive: true });
  copyFileSync(new URL('../package.json', import.meta.url), join(project, 'package.json'));
  copyFileSync(script, join(project, 'scripts/wasm-budget.mjs'));
  writeFileSync(join(project, 'bin/wasm-pack'), '#!/bin/sh\nexit 0\n', { mode: 0o755 });
  for (const [path, size] of [['wasm-pkg/sugarscape_bg.wasm', limit], ['experiments-pkg/experiments_bg.wasm', limit + 1]]) {
    const artifact = join(project, 'src', path);
    writeFileSync(artifact, '');
    truncateSync(artifact, size);
  }
  const result = spawnSync('npm', ['run', 'build'], { cwd: project, encoding: 'utf8', env: { ...process.env, PATH: `${join(project, 'bin')}:${process.env.PATH}` } });
  assert.notEqual(result.status, 0, result.stdout);
  assert.match(result.stdout, /engine bundle 8000000 bytes/);
  assert.match(result.stderr, /experiments bundle.*8000001 bytes.*8000000/);
});
