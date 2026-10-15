import { Worker } from 'node:worker_threads';
import { fileURLToPath } from 'node:url';
import { writeFileSync, readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const moduleUrl=new URL('../../../web/src/experiments-pkg/experiments.js',import.meta.url).href;
const wasmPath=fileURLToPath(new URL('../../../web/src/experiments-pkg/experiments_bg.wasm',import.meta.url));
const input=JSON.stringify({study:'persistent_notes',stratum:{acquisition_cost:1,write_cost:2,note_read_cost:1,manifest_inspection_cost:4,writer_share:3},arm:'Retained',routine:'ReadAdaptive',request:'Right',ids:{writer:'Writer',reader:'Reader'}});
async function cold(op,text){
 const start=performance.now(); let peakRss=process.memoryUsage().rss;
 const sampler=setInterval(()=>{peakRss=Math.max(peakRss,process.memoryUsage().rss);},10);
 const worker=new Worker(`const {parentPort,workerData}=require('node:worker_threads'); const {readFileSync}=require('node:fs'); (async()=>{const wasm=await import(workerData.moduleUrl);wasm.initSync({module:readFileSync(workerData.wasmPath)});const result=workerData.op==='run'?wasm.experiment_run_json(workerData.text):wasm.experiment_validate_json(workerData.text);parentPort.postMessage(result);})();`,{eval:true,workerData:{moduleUrl,wasmPath,op,text}});
 try { const result=await new Promise((resolve,reject)=>{const deadline=setTimeout(()=>{worker.terminate();reject(new Error('actual worker deadline exceeded 60 seconds'));},60_000);worker.once('message',value=>{clearTimeout(deadline);resolve(value);});worker.once('error',error=>{clearTimeout(deadline);reject(error);});}); return {result,evidence:{op,cold_total_ms:performance.now()-start,peak_process_rss_bytes:peakRss,record_bytes:Buffer.byteLength(result),record_sha256:createHash('sha256').update(result).digest('hex')}};
 } finally {clearInterval(sampler);await worker.terminate();}
}
const run=await cold('run',input);const validate=await cold('validate',run.result);
if(run.result!==validate.result) throw new Error('fresh cold validation record mismatch');
const record=JSON.parse(run.result);const bytes=readFileSync(wasmPath);
const evidence={deadline_ms:60_000,full_raw_module_bytes:bytes.length,module_sha256:createHash('sha256').update(bytes).digest('hex'),rules_identity:record.rules_identity,engineering_selected_evaluations:{run:1,validation_reconstruction:1},workers:[run.evidence,validate.evidence]};
writeFileSync(new URL('task-5-cold-worker.json',import.meta.url),JSON.stringify(evidence,null,2)); console.log(JSON.stringify(evidence,null,2));
