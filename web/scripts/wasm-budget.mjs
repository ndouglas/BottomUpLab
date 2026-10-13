// Fails the web build when the engine bundle grows past its budget. The spec
// measured 6.58 MB after the split; the headroom is for new model kinds, not
// for the provenance cluster, which belongs in the experiments package.
import { statSync } from 'node:fs';

const budget = 8_000_000;
const path = process.argv[2] ?? new URL('../src/wasm-pkg/sugarscape_bg.wasm', import.meta.url);
const size = statSync(path).size;
if (size > budget) {
  console.error(`engine bundle is ${size} bytes, over the ${budget} byte budget; a cluster export has leaked into sugarscape-wasm`);
  process.exit(1);
}
console.log(`engine bundle ${size} bytes, within the ${budget} byte budget`);
