from pathlib import Path
import os,subprocess,tempfile,shutil,json
src=Path('/Users/gqadonis/.claude/worktrees/production-readiness-476')
b=Path(tempfile.mkdtemp(prefix='uar476-hook-'));r=b/'repo';(r/'scripts').mkdir(parents=True);(b/'bin').mkdir()
for s in ['scripts/verify-rust-test-baseline.sh','lefthook.yml']:shutil.copy2(src/s,r/s)
(r/'.gitignore').write_text('target/\n')
subprocess.run(['git','init','-q'],cwd=r,check=True)
for tool in ['cargo','rustc']:
 f=b/'bin'/tool
 f.write_text('#!/bin/sh\nif [ "$1" = "test" ]; then printf "%s\\n" "$*" >> "$CALL_LOG"; fi\nprintf "synthetic '+tool+'\\n"\n');f.chmod(0o755)
env={**os.environ,'PATH':str(b/'bin')+':/usr/local/bin:'+os.environ['PATH'],'CALL_LOG':str(b/'calls')}
lefthook='/Users/gqadonis/Projects/prometheus/universal-agent-runtime/node_modules/.bin/lefthook'
rows=[]
for name,expected in [('src/example.rs',3),('Cargo.toml',3),('crates/example/Cargo.lock',3),('rust-toolchain.toml',3),('.cargo/config.toml',3),('scripts/verify-rust-test-baseline.sh',3),('README.md',0)]:
 f=r/name;f.parent.mkdir(parents=True,exist_ok=True)
 if not f.exists():f.write_text('synthetic fixture\n')
 (b/'calls').write_text('')
 p=subprocess.run([lefthook,'run','pre-commit','--command','rust-test-baseline','--file',name,'--no-auto-install','--no-tty'],cwd=r,env=env,capture_output=True,text=True)
 calls=(b/'calls').read_text().splitlines()
 rows.append({'file':name,'exit_code':p.returncode,'compile_calls':len(calls),'passed':p.returncode==0 and len(calls)==expected,'stdout':p.stdout,'stderr':p.stderr})
Path('/tmp/uar476-hook-controls.json').write_text(json.dumps({'claim':'Installed Lefthook selector controls in synthetic Git repo, fake Cargo/rustc; no Rust compilation', 'results':rows},indent=2)+'\n')
print(json.dumps({'hook_controls':[{k:v for k,v in row.items() if k not in ['stdout','stderr']} for row in rows],'receipt':'/tmp/uar476-hook-controls.json'}))
shutil.rmtree(b)
raise SystemExit(0 if all(x['passed'] for x in rows) else 1)
