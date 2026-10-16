import { beforeEach, expect, it, vi } from 'vitest';
const mocks = vi.hoisted(() => ({ init:vi.fn(), load:vi.fn(), catalog:vi.fn(()=> '[]'), run:vi.fn(()=> '{}'), validate:vi.fn(()=> '{}') }));
vi.mock('../experiments-pkg/experiments.js',()=>({default:mocks.init,experiment_catalog_json:mocks.catalog,experiment_run_json:mocks.run,experiment_validate_json:mocks.validate}));
vi.mock('./recorded-assets',()=>({loadRecordedAsset:mocks.load}));
vi.mock('./recorded-manifest.json',()=>({default:{}}));
beforeEach(()=>{vi.resetModules();vi.clearAllMocks();});
for(const op of ['catalog','run','validate','recorded']) it(`${op} isolates recorded loading from WASM initialization`,async()=>{
 let listener: (event: unknown)=>Promise<void> = async()=>{};
 vi.stubGlobal('addEventListener',(_type:string,fn:typeof listener)=>{listener=fn;});
 const post = vi.fn();vi.stubGlobal('postMessage',post);
 mocks.load.mockResolvedValue({kind:'recorded_results',version:1,studies:{}});
 await import('./worker');
 await listener({data:{id:1,op,text:'{}'}});
 expect(mocks.load).toHaveBeenCalledTimes(op==='recorded'?1:0);
 expect(mocks.init).toHaveBeenCalledTimes(op==='recorded'?0:1);
 expect(post).toHaveBeenCalledTimes(1);
 vi.unstubAllGlobals();
});
it('reports a failed recorded fetch without initializing WASM',async()=>{
 let listener: (event: unknown)=>Promise<void> = async()=>{};
 vi.stubGlobal('addEventListener',(_type:string,fn:typeof listener)=>{listener=fn;});
 const post=vi.fn();vi.stubGlobal('postMessage',post);
 mocks.load.mockRejectedValue(new Error('Recorded asset unavailable (404)'));
 await import('./worker');await listener({data:{id:2,op:'recorded'}});
 expect(post.mock.calls[0][0].errors[0].message).toContain('404');
 expect(mocks.init).not.toHaveBeenCalled();
 vi.unstubAllGlobals();
});
