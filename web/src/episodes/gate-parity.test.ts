import { fileURLToPath } from 'node:url';
import { readFileSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { expect, it } from 'vitest';
import { initSync, experiment_run_json, experiment_validate_json } from '../experiments-pkg/experiments.js';
import type { EpisodeRecord } from './types';
initSync({ module: readFileSync(new URL('../experiments-pkg/experiments_bg.wasm', import.meta.url)) });
const cwd = fileURLToPath(new URL('../../../', import.meta.url));
function boundedRequest<T>(operation: () => T): T {
  const started = performance.now();
  const result = operation();
  expect(performance.now() - started).toBeLessThan(60_000);
  return result;
}
for (const [name, holding_cost, holder_share, distance, holder, retriever] of [
 ['supplied near',1,3,0,540,60], ['supplied far',2,6,1,423,30], ['non-equilibrium delayed deadline',1,3,0,540,15], ['idle deadline',1,0,0,0,0],
] as const) {
  it(`${name}: current actual WASM reproduces complete native record`, () => {
    const scratch = mkdtempSync(`${tmpdir()}/gate-parity-`);
    try {
      const input = JSON.stringify({study:'gate_assistance',stratum:{holding_cost,holder_share,distance},profile:{holder,retriever},ids:{holder:'Agent-A',retriever:'Agent-B'}});
      const path = `${scratch}/input.json`;writeFileSync(path,input);
      const options = {cwd,encoding:'utf8' as const,timeout:60_000};
      const record = boundedRequest(() => JSON.parse(execFileSync(`${cwd}/target/release/sugarscape`, ['experiment-view','run','--input',path], options))) as EpisodeRecord;
      const actual = boundedRequest(() => JSON.parse(experiment_run_json(input)));
      expect(actual).toEqual(record);
      expect(boundedRequest(() => JSON.parse(experiment_validate_json(JSON.stringify(actual))))).toEqual(record);
    } finally { rmSync(scratch,{recursive:true,force:true}); }
  // Three separate requests each retain the application's 60-second limit.
  }, 180_000);
}
it('duplicate raw JSON keys reject before a map can erase them', () => {
  expect(() => experiment_run_json('{"study":"gate_assistance","study":"gate_assistance","stratum":{"holding_cost":1,"holder_share":3,"distance":0},"selector":"SuppliedHelp","ids":{"holder":"Agent-A","retriever":"Agent-B"}}')).toThrow();
}, 30_000);
