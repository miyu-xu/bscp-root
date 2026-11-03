"""Validate a retained r14 route; never create/start/resume/delete an instance."""
from pathlib import Path
import argparse,json,hashlib,subprocess,sys
P=Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--platform',choices=['linux','macos','windows'],required=True);p.add_argument('--plan',action='store_true');a=p.parse_args()
s=json.loads((P/'r14-rollback.json').read_bytes())['platforms'][a.platform]
if a.plan:print(json.dumps({'platform':a.platform,**s,'action':'check','VM':False},indent=2));raise SystemExit(0)
code='SPEC='+repr(s)+'\nPLATFORM='+repr(a.platform)+'\n'+"""
from pathlib import Path
import hashlib,json,subprocess,sys
entry=Path(SPEC['entry']);root=Path(SPEC['state']);profile=root/'profiles'/(PLATFORM+'.json')
assert hashlib.sha256(entry.read_bytes()).hexdigest()==SPEC['entry_sha256'],'entry changed'
assert hashlib.sha256(profile.read_bytes()).hexdigest()==SPEC['profile_sha256'],'profile changed'
cmd=[sys.executable,'-B',str(entry),'--state-root',str(root),'--platform',PLATFORM,'check']
p=subprocess.run(cmd,capture_output=True,text=True,timeout=180)
print(json.dumps({'argv':cmd,'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr,'VM_started':False}))
raise SystemExit(p.returncode)
"""
if s['ssh']:
 cmd=['ssh','-T','-o','BatchMode=yes','-o','StrictHostKeyChecking=yes','-o','ConnectTimeout=15',s['ssh'],'python3 -B -']
else:cmd=[sys.executable,'-B','-']
r=subprocess.run(cmd,input=code,text=True,capture_output=True,timeout=210);sys.stdout.write(r.stdout);sys.stderr.write(r.stderr);raise SystemExit(r.returncode)
