from pathlib import Path
import os,subprocess,tempfile,shutil,json
source=Path('/Users/gqadonis/.claude/worktrees/production-readiness-476/scripts/verify-rust-test-baseline.sh')
base=Path(tempfile.mkdtemp(prefix='uar476-safeguard-'))
root=base/'repo';(root/'scripts').mkdir(parents=True);(base/'bin').mkdir()
shutil.copy2(source,root/'scripts'/source.name)
(root/'.gitignore').write_text('target/\n');(root/'source.txt').write_text('committed\n')
def git(*args):return subprocess.run(['git',*args],cwd=root,text=True,capture_output=True,check=True)
git('init','-q');git('config','user.name','Synthetic Fixture');git('config','user.email','fixture@example.invalid');git('add','.');git('commit','-qm','fixture')
(base/'bin/cargo').write_text('''#!/usr/bin/env python3
import os,sys
from pathlib import Path
if sys.argv[1:]==['--version']:
 print('cargo synthetic-fixture');sys.exit(0)
with open(os.environ['CALL_LOG'],'a') as f:f.write(' '.join(sys.argv[1:])+'\\n')
control=os.environ.get('CONTROL','pass')
if control=='fail':print('error: controlled compile failure');sys.exit(42)
if control=='warning':print('warning: controlled diagnostic')
if control=='mutate':Path('source.txt').write_text('changed\\n')
print('Finished synthetic compile; no Rust code compiled')
''')
(base/'bin/rustc').write_text('#!/bin/sh\nprintf "rustc synthetic-fixture\\n"\n')
for f in (base/'bin').iterdir():f.chmod(0o755)
env={**os.environ,'PATH':str(base/'bin')+':'+os.environ['PATH'],'CALL_LOG':str(base/'calls'),'TMPDIR':str(base)}
results=[]
def run(name,control,expected,dirty=False):
 git('checkout','--','source.txt')
 if dirty:(root/'source.txt').write_text('dirty\n')
 (base/'calls').write_text('')
 p=subprocess.run(['bash','scripts/verify-rust-test-baseline.sh','--acceptance'],cwd=root,env={**env,'CONTROL':control},text=True,capture_output=True)
 calls=(base/'calls').read_text().splitlines()
 ok=(p.returncode==0 if expected=='pass' else p.returncode!=0)
 if name=='clean_profile_matrix':ok=ok and len(calls)==3 and 'minimal' in calls[0] and 'server-full' in calls[1] and 'server-full,postgres-backend' in calls[2]
 if name=='dirty_source_refusal':ok=ok and not calls
 if name=='compile_failure':ok=ok and p.returncode==42 and len(calls)==1
 if name=='warning_refusal':ok=ok and len(calls)==1 and 'warnings emitted' in p.stderr
 if name=='changed_source_refusal':ok=ok and 'source identity changed or became dirty' in p.stderr
 results.append({'name':name,'passed':ok,'exit_code':p.returncode,'cargo_calls':calls,'stdout':p.stdout,'stderr':p.stderr})
run('clean_profile_matrix','pass','pass')
run('dirty_source_refusal','pass','fail',True)
run('compile_failure','fail','fail')
run('warning_refusal','warning','fail')
run('changed_source_refusal','mutate','fail')
out={'claim':'Synthetic safeguard controls only; fake Cargo/rustc. No Rust compilation or product behavior proved.','results':results}
Path('/tmp/uar476-safeguard-controls.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'synthetic_controls':[{'name':r['name'],'passed':r['passed'],'exit_code':r['exit_code']} for r in results],'receipt':'/tmp/uar476-safeguard-controls.json'}))
shutil.rmtree(base)
raise SystemExit(0 if all(r['passed'] for r in results) else 1)
