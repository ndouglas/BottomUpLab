// Raw release byte ceilings, independently enforced for both browser modules.
// Usage: node scripts/wasm-budget.mjs [engine-path] [--experiments path]
// Single-package release hooks use --only engine|experiments.
import { statSync } from 'node:fs';

const ceiling = 8_000_000;
const paths = {
  engine: new URL('../src/wasm-pkg/sugarscape_bg.wasm', import.meta.url),
  experiments: new URL('../src/experiments-pkg/experiments_bg.wasm', import.meta.url),
};
let only;
let enginePath;
const args = process.argv.slice(2);
for (let index = 0; index < args.length; index++) {
  const arg = args[index];
  if (arg === '--experiments' && args[index + 1]) paths.experiments = args[++index];
  else if (arg === '--only' && ['engine', 'experiments'].includes(args[index + 1])) only = args[++index];
  else if (!arg.startsWith('-') && !enginePath) { paths.engine = arg; enginePath = true; }
  else { console.error('Usage: wasm-budget.mjs [engine-path] [--experiments path] [--only engine|experiments]'); process.exit(1); }
}
let failed = false;
for (const module of only ? [only] : ['engine', 'experiments']) {
  let size;
  try { size = statSync(paths[module]).size; }
  catch (error) {
    console.error(`${module} bundle ${paths[module]} missing or unreadable: actual bytes unavailable, ceiling ${ceiling} bytes (${error.code})`);
    failed = true;
    continue;
  }
  if (size > ceiling) {
    console.error(`${module} bundle is ${size} bytes, over the ${ceiling} byte budget`);
    failed = true;
  } else console.log(`${module} bundle ${size} bytes, within the ${ceiling} byte budget`);
}
if (failed) process.exit(1);
