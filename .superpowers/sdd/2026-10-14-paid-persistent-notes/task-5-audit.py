import hashlib,json,re,struct,subprocess
from pathlib import Path
root=Path.cwd();out=root/'.superpowers/sdd/2026-10-14-paid-persistent-notes';base='08d5562f4578b572549d8a4ba99000f2124610ba'
def raw(path):
 h=hashlib.sha256();n=0
 with Path(path).open('rb') as f:
  while chunk:=f.read(65536):h.update(chunk);n+=len(chunk)
 return {'bytes':n,'sha256':h.hexdigest()}
def receipt(path):return {'path':str(path),**raw(path)}
frozen=json.loads((out/'baseline-preservation.json').read_text());mismatch=[]
for old in frozen['receipts']:
 now=raw(old['path'])
 if any(now[k]!=old[k] for k in ('bytes','sha256')):mismatch.append({'baseline':old,'current':now})
assert [Path(x['baseline']['path']).relative_to(root).as_posix() for x in mismatch]==['crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json']
prior=json.loads((out/'task-4-final-receipts.json').read_text())
for category in ('source','reference'):
 for old in prior[category]['receipts']:assert raw(old['path'])=={k:old[k] for k in ('bytes','sha256')},old['path']
assert raw(prior['report']['path'])=={k:prior['report'][k] for k in ('bytes','sha256')}
changes=[];new=[]
for path in subprocess.check_output(['git','diff','--name-only',base],text=True).splitlines():
 if path.startswith('.superpowers/'):continue
 prior_file=subprocess.run(['git','show',base+':'+path],capture_output=True)
 if prior_file.returncode:new.append(receipt(path));continue
 old=prior_file.stdout;changes.append({'path':path,'base':{'bytes':len(old),'sha256':hashlib.sha256(old).hexdigest()},'current':raw(path)})
new.extend(receipt(path) for path in subprocess.check_output(['git','ls-files','--others','--exclude-standard'],text=True).splitlines())
def adapter(path):
 text=Path(path).read_text(); names=re.findall(r'source!\("([^"]+)"\)',text)
 receipts=[receipt('crates/sugarscape-core/src/'+name) for name in names]
 return {'ordered_receipts':receipts,'sha256':hashlib.sha256(json.dumps(receipts,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()}
identities={name:adapter(f'crates/sugarscape-core/src/browser_experiments/{name}/mod.rs') for name in ('notes','key')}
metadata=json.loads(Path('crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json').read_text())
for binding in metadata['bindings']:
 for r in binding['sources']:assert receipt(r['path'])==r
artifacts=[receipt(p) for p in ['target/release/sugarscape','web/src/experiments-pkg/experiments_bg.wasm','web/src/experiments-pkg/experiments.js','crates/sugarscape-experiments-wasm/Cargo.toml','crates/sugarscape-experiments-wasm/Cargo.lock']]
value={'baseline':base,'frozen_receipts_count':len(frozen['receipts']),'baseline_mismatches':mismatch,'unchanged_native_source':prior['source'],'unchanged_native_reference':prior['reference'],'unchanged_task4_retained_report':prior['report'],'existing_deltas':changes,'new_files':new,'adapters':identities,'delivery_artifacts':artifacts,'historical_metadata_source_receipts':'exact actual raw files; frozen legacy records must independently reconstruct before compatibility is claimed'}
(out/'task-5-receipts.json').write_text(json.dumps(value,indent=2)+'\n')
print('preserved',len(frozen['receipts']),'frozen receipts; only declared compatibility manifest differs; native source/reference unchanged; actual adapter and delivery receipts recorded')
