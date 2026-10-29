"""Non-VM actual Darwin token/kqueue+production observation owner exercise."""
import pathlib,sys,os,subprocess,select,time,json,hashlib,copy
ROOT=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'t'))
from mac_audit import AuditOps
from audit_owner import BoundInstance
from harmless import close_owned,retain
CHILD='''import time,signal,json
start=time.monotonic();seen=[]
def term(s,f):seen.append(time.monotonic())
signal.signal(signal.SIGTERM,term)
print('READY',flush=True)
while time.monotonic()-start<20:
 if seen and time.monotonic()-seen[0]>=8:break
 time.sleep(.01)
print(json.dumps({'signal_count':len(seen),'elapsed':time.monotonic()-start}),flush=True)
'''
def main():
 out=ROOT/'delayed-observe';out.mkdir(exist_ok=False)
 pins={n:hashlib.sha256((ROOT/'t'/n).read_bytes()).hexdigest() for n in ('mac_audit.py','audit_owner.py')}
 manifest=json.loads((ROOT/'transfer-manifest.json').read_text())
 if any(pins[n]!=manifest['t/'+n]['sha256'] for n in pins):raise RuntimeError('production source changed')
 children=[];handles=[];q=AuditOps();expected={};eof_by_pid={};result={'pass':False,'pins':pins,'non_vm':True,'storage_probe':'SENTINEL_NO_BUSINESS_DISK'};start=time.monotonic()
 try:
  own_exe=q.identity(os.getpid())['exe']
  for role in ('virtmgr','guest'):
   p=subprocess.Popen([sys.executable,'-u','-c',CHILD],stdout=subprocess.PIPE,stderr=subprocess.PIPE);children.append(p)
   if not select.select([p.stdout],[],[],3)[0] or p.stdout.readline()!=b'READY\n':raise RuntimeError('READY missing')
   ident=q.identity(p.pid)
   if ident['exe']!=own_exe:raise RuntimeError('wrong child executable')
   expected[p.pid]={**ident,'start':'original-own-Popen','sid':os.getsid(p.pid),'cmd':CHILD,'role':role}
  ops=AuditOps(expected)
  # Direct unreaped Popen children cannot be confused with unrelated reused PID.
  # This substitutes production session discovery, not token/kqueue operations.
  def inventory():return {p.pid:copy.deepcopy(expected[p.pid]) for p in children if p.poll() is None}
  def logs():
   if any(p.poll() is None for p in children):return False
   if result.get('reader_errors'):raise RuntimeError('original reader failure remains unproven')
   for p in children:
    if p.pid in eof_by_pid:continue
    try:
     o,e=p.communicate(timeout=1);eof_by_pid[p.pid]=dict(pid=p.pid,exit=p.returncode,stdout=o.decode(),stderr=e.decode(),eof=True)
    except BaseException as exc:
     result.setdefault('reader_errors',[]).append(repr(exc));raise
   return set(eof_by_pid)=={p.pid for p in children}
  owner=BoundInstance(expected,inventory,os.getpid(),ops=ops);owner.bind()
  before=time.monotonic();initial=owner.terminate_once(authorized=True,logs_released=logs,storage_snapshot=lambda:{'non_vm_storage':True})
  result.update(initial=initial,initial_seconds=time.monotonic()-before)
  deadline=time.monotonic()+30;observations=[];final=initial
  for i in range(30):
   if time.monotonic()>=deadline or final['resource_state']=='REAPED' or final.get('identity_observation_blocked'):break
   time.sleep(min(1,max(0,deadline-time.monotonic())))
   final=owner.observe_once(logs_released=logs,storage_snapshot=lambda:{'non_vm_storage':True})
   observations.append(dict(elapsed=time.monotonic()-before,state=final['resource_state']))
  child_results=list(eof_by_pid.values())
  result.update(final=final,observations=observations,children=child_results,handles=ops.evidence())
  result['pass']=(initial['resource_state']=='UNKNOWN' and initial.get('observation_expired') is True and len(observations)>0 and final['resource_state']=='REAPED' and len(final['signals'])==2 and all(c['exit']==0 and c['eof'] and json.loads(c['stdout'])['signal_count']==1 for c in child_results) and len(child_results)==2)
 except BaseException as e:result['error']=repr(e);result['pass']=False
 # No repeat signal/KILL. Independent20s cooperative fallback, then original
 # child wait and every partial handle accounted for; failure remains failure.
 for p in children:
  if p.pid not in eof_by_pid:
   try:
    o,e=p.communicate(timeout=max(1,26-(time.monotonic()-start)))
    eof_by_pid[p.pid]=dict(pid=p.pid,exit=p.returncode,stdout=o.decode(),stderr=e.decode(),eof=True)
   except BaseException as e:result['retained_error']=repr(e);retain(result)
 if 'ops' in locals() and not close_owned(ops,children,result):retain(result)
 if not close_owned(q,children,result):retain(result)
 if result.get('reader_errors') or result.get('close_errors') or result.get('error'):result['pass']=False
 result['cleanup_children']=list(eof_by_pid.values())
 result['elapsed_seconds']=time.monotonic()-start
 with (out/'result.json').open('x') as f:json.dump(result,f,indent=2);f.flush();os.fsync(f.fileno())
 print(json.dumps(result),flush=True)
 return 0 if result['pass'] else 2
if __name__=='__main__':sys.exit(main())
