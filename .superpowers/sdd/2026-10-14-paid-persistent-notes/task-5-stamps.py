import hashlib,json,re,struct,subprocess
from pathlib import Path
root=Path.cwd(); out=root/'.superpowers/sdd/2026-10-14-paid-persistent-notes'
def raw(path):
 h=hashlib.sha256(); n=0
 with path.open('rb') as f:
  while chunk:=f.read(65536): h.update(chunk); n+=len(chunk)
 return {'bytes':n,'sha256':h.hexdigest()}
def names(path,label):
 text=path.read_text(); body=re.search(r'pub const '+label+r': &\[&str\] = &\[(.*?)\];',text,re.S).group(1)
 return re.findall(r'"([^"]+)"',body)
def domain_hash(paths,domain):
 h=hashlib.sha256(domain.encode()+b'\0'); total=0
 for name in sorted(paths):
  path=root/name; size=path.stat().st_size; total+=size
  encoded=name.encode();h.update(struct.pack('>Q',len(encoded)));h.update(encoded);h.update(struct.pack('>Q',size))
  with path.open('rb') as f:
   while chunk:=f.read(65536):h.update(chunk)
 return h.hexdigest(),total
prior_stamps=json.loads((out/'task-4-current-stamps.json').read_text()); stamps={}
for stamp,helper,domain in [('f5_source_identity.rs','f5_source_identity.rs','foraging-shortcut-source-v1'),('war1_compiled_inputs.rs','war1_source_identity.rs','war1-compiled-inputs-v1'),('war1_study_stamp.rs','war1_study_source_identity.rs','war1-study-compiled-inputs-v1')]:
 helper=root/'survey/build_support'/helper;dirs=names(helper,'DIRECTORIES');files=names(helper,'FILES');paths=set()
 for directory in dirs:
  for path in (root/directory).rglob('*'):
   if path.is_file():
    rel=path.relative_to(root).as_posix()
    if stamp=='f5_source_identity.rs' and not rel.endswith('.rs'):continue
    if stamp!='f5_source_identity.rs' and any(part in {'target','.git','.superpowers','node_modules','__pycache__','generated','evidence','cache'} for part in path.relative_to(root).parts):continue
    if stamp=='war1_study_stamp.rs' and rel=='crates/sugarscape-core/src/persistent_notes/tests/fixtures/evaluation-reference.json':continue
    paths.add(rel)
 for name in files:
  if (root/name).is_file():paths.add(name)
 if stamp=='war1_study_stamp.rs':
  paths.update((root/'survey/war1_study/fixtures/core-literal-inputs-v1.txt').read_text().splitlines())
  paths.add('survey/war1_study/fixtures/construction-manifest.json')
  for path in (root/'survey/war1_study').rglob('*.py'):paths.add(path.relative_to(root).as_posix())
  assert all((root/name).stat().st_size<=16*1024*1024 for name in paths)
 fresh,total=domain_hash(paths,domain)
 if stamp=='war1_study_stamp.rs':assert total<=64*1024*1024
 candidates=[]
 for path in (root/'survey/target/debug/build').glob('survey-*/out/'+stamp):
  text=path.read_text(); label={'f5_source_identity.rs':'COMPILED_SOURCE_IDENTITY','war1_compiled_inputs.rs':'COMPILED_W1_INPUTS_SHA256','war1_study_stamp.rs':'INPUTS_SHA256'}[stamp];values=re.findall(label+r'[^=]*=\s*"([0-9a-f]{64})"',text)
  if values and values[0]==fresh:candidates.append(path)
 assert candidates, f'no actual current compiled stamp for {stamp}: {fresh}'
 path=max(candidates,key=lambda p:p.stat().st_mtime_ns);old=set(prior_stamps[stamp]['selected_paths']);assert not old-paths
 stamps[stamp]={'stamp_path':path.relative_to(root).as_posix(),'prior_identity':prior_stamps[stamp]['fresh_identity'],'compiled_identity':fresh,'fresh_identity':fresh,'selected_paths':sorted(paths),'selected_bytes':total,'added_paths':sorted(paths-old),'removed_paths':sorted(old-paths)}
(out/'task-5-current-stamps.json').write_text(json.dumps(stamps,indent=2)+'\n')
print('current compiled F5/W1/W1-A stamps match exact fresh inventories; old paths retained; W1-A exact exclusion and caps unchanged')
