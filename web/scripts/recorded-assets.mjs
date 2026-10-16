// Reproducible from tracked inputs and the normal native exporter; no saved outputs required.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../../', import.meta.url));
execFileSync('cargo', ['build', '--release', '-p', 'sugarscape-cli'], { cwd: root, stdio: 'inherit' });
const bytes = execFileSync(`${root}/target/release/sugarscape`, ['experiment-view', 'recorded'], { cwd: root, maxBuffer: 16 * 1024 * 1024, timeout: 60000 });
if (bytes.length > 16 * 1024 * 1024) throw new Error('Recorded asset exceeds 16 MiB');
const value = JSON.parse(bytes);
if (value.kind !== 'recorded_results' || value.version !== 1) throw new Error('Invalid native recorded envelope');
const sha256 = createHash('sha256').update(bytes).digest('hex');
const manifest = { version: 'browser-recorded-asset-v1', path: `recorded-assets/${sha256}.json`, bytes: bytes.length, sha256 };
mkdirSync(`${root}/web/public/recorded-assets`, { recursive: true });
writeFileSync(`${root}/web/public/${manifest.path}`, bytes);
writeFileSync(`${root}/web/src/episodes/recorded-manifest.json`, JSON.stringify(manifest) + '\n');
console.log(JSON.stringify(manifest));
