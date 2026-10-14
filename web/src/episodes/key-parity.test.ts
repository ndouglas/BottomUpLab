import { readFileSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { beforeAll, afterAll, describe, expect, it } from 'vitest';
import { initSync, experiment_run_json } from '../experiments-pkg/experiments.js';
import type { EpisodeRecord } from './types';
initSync({module:readFileSync(new URL('../experiments-pkg/experiments_bg.wasm',import.meta.url))});
const cwd=fileURLToPath(new URL('../../../',import.meta.url));
describe.each(['SuppliedHandoff','InitiallyEquipped'])('%s selected native/WASM', selector=>{
 const input=JSON.stringify({study:'key_delegation',stratum:{transfer_enabled:true,transfer_cost:1,giver_share:3,distance:0},ids:{giver:'007 ❄',receiver:'receiver "quoted"'},selector});
 let native:EpisodeRecord,wasm:EpisodeRecord;
 const dir=mkdtempSync(`${tmpdir()}/key-web-parity-`);
 beforeAll(()=>{writeFileSync(`${dir}/input.json`,input);const options={cwd,encoding:'utf8' as const,timeout:60_000,maxBuffer:16*1024*1024};native=JSON.parse(execFileSync(`${cwd}/target/release/sugarscape`,['experiment-view','run','--input',`${dir}/input.json`],options));},60_000);
 // Full-record parity setup has a 120-second engineering allowance; the worker contract remains 60 seconds.
 beforeAll(()=>{wasm=JSON.parse(experiment_run_json(input));},120_000);
 afterAll(()=>rmSync(dir,{recursive:true,force:true}));
 it('preserves every checkpoint, receipt, payload, identity and opaque identifier',()=>expect(wasm).toEqual(native));
});
