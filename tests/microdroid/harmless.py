"""Fresh owned-child audit-token/kqueue test. No VM, PID kill, or retry."""
import os,sys,pathlib,json,hashlib,subprocess,select,time
ROOT=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'t'))
from mac_audit import AuditOps
def retain(result):
 try:print(json.dumps(result),flush=True)
 except BaseException:pass
 while True:time.sleep(1)

def close_owned(ops,children,result):
 # Includes partial acquire/open resources, not just successfully registered
 # handles. Direct Popen exit may justify disposal but never NOTE_EXIT PASS.
 by_pid={p.pid:p for p in children}
 for h in ops.retained:
  try:
   p=by_pid.get(h['pid'])
   if p is None or p.poll() is None:raise RuntimeError('original child not reaped')
   if h.get('kq') is not None and not h.get('kq_closed'):
    if h.get('kq_close_attempted'):raise RuntimeError('kqueue close uncertain')
    if h.get('registration_verified'):
     ops.close(h)
    else:
     h['kq_close_attempted']=True;h['kq'].close();h['kq_closed']=True
   if not h.get('port_closed'):ops.release_port(h)
  except BaseException as e:result.setdefault('close_errors',[]).append(repr(e))
 unresolved=any(not h.get('port_closed') or (h.get('kq') is not None and not h.get('kq_closed')) for h in ops.retained)
 return not unresolved

def main():
 if sys.argv[1:]!=['--approved-own-child']:raise RuntimeError('explicit probe invocation required')
 m=hashlib.sha256((ROOT/'transfer-manifest.json').read_bytes()).hexdigest()
 ops=AuditOps();children=[];handles=[];result={'manifest_sha256':m,'pass':False,'signals':0,'children':[]}
 expected_exe=ops.identity(os.getpid())['exe']
 result['expected_python_executable']=expected_exe
 start=time.monotonic()
 try:
  for i in range(2):
   p=subprocess.Popen([sys.executable,'-u','-c','import time;print("READY",flush=True);time.sleep(20)'],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
   children.append(p)
   if not select.select([p.stdout],[],[],3)[0] or p.stdout.readline()!=b'READY\n':raise RuntimeError('READY missing')
   identity=ops.identity(p.pid)
   if identity['exe']!=expected_exe:raise RuntimeError('child executable differs from original Python executable')
   ops.expected[p.pid]=identity
   h=ops.open(p.pid);handles.append(h)
  for h in handles:
   result['signals']+=1;ops.term(h)
  end=time.monotonic()+5
  while time.monotonic()<end and not all(ops.exited(h) for h in handles):time.sleep(.01)
  if not all(ops.exited(h) for h in handles):raise RuntimeError('original NOTE_EXIT missing')
 except BaseException as e:result['error']=repr(e)
 # Every child has an independent20s cooperative exit; no signal replay.
 for p in children:
  try:
   out,err=p.communicate(timeout=max(1,26-(time.monotonic()-start)))
   result['children'].append(dict(pid=p.pid,exit=p.returncode,eof=True,stdout=out.decode(),stderr=err.decode()))
  except BaseException as e:
   result['error']=repr(e);result['retained']=True
   retain(result) # reporting failure cannot discard original child/pipes
 if not close_owned(ops,children,result):
  result['retained']=True;result['handles']=ops.evidence();retain(result)
 result['handles']=ops.evidence()
 result['pass']=('error' not in result and not result.get('close_errors') and len(children)==len(handles)==2 and all(r['exit']==-15 and r['eof'] for r in result['children']) and all(h.get('registration_verified') and h.get('exited') and h.get('port_closed') and h.get('kq_closed') for h in handles))
 result['elapsed_seconds']=time.monotonic()-start
 with (ROOT/'harmless-result.json').open('x') as f:json.dump(result,f,indent=2);f.flush();os.fsync(f.fileno())
 print(json.dumps(result),flush=True)
 return 0 if result['pass'] else 2
if __name__=='__main__':sys.exit(main())
