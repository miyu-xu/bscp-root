"""TEST ONLY: real runtime/driver pipes, simulated Darwin APIs, no signals/VM."""
import pathlib,sys,json,os,threading,queue,subprocess,time,types,copy
HERE=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/'scripts'/'microdroid_lifecycle'))
import runtime,native,mac_owner,audit_owner
from test_mac import World
DISK={k:dict(dev=1,inode=i+1,bytes=8,sha256='a'*64) for i,k in enumerate(('instance.img','storage.img','app.idsig'))}
PAYLOAD='''import sys,threading,queue,time
q=queue.Queue()
def reader():
 for line in sys.stdin:q.put(line.strip())
 q.put(None)
threading.Thread(target=reader,daemon=True).start()
n=0;end=time.monotonic()+25
while time.monotonic()<end:
 try:c=q.get(timeout=1)
 except queue.Empty:continue
 if c is None:break
 if c=='INFO':r='HEADLESS_COUNTER_V1 GUEST_ENCRYPTED_STORAGE'
 elif c=='GET':r='VALUE '+str(n)
 elif c.startswith('SET '):n=int(c[4:]);r='VALUE '+str(n)
 elif c=='ADD 1':n+=1;break
 else:r='ERROR'
 sys.stdout.buffer.write((r+'\\n').encode());sys.stdout.buffer.flush()
else:sys.exit(2)
'''
class Pipe:
 def __init__(self):
  self.p=subprocess.Popen([sys.executable,'-u','-c',PAYLOAD],stdin=subprocess.PIPE,stdout=subprocess.PIPE)
  self.q=queue.Queue();self.timeout=3
  def read():
   while True:
    b=self.p.stdout.readline();self.q.put(b)
    if not b:return
  self.reader=threading.Thread(target=read);self.reader.start()
 def __enter__(self):return self
 def __exit__(self,*a):pass
 def settimeout(self,n):self.timeout=n
 def sendall(self,b):self.p.stdin.write(b);self.p.stdin.flush()
 def recv(self,n):return self.q.get(timeout=self.timeout)
 def shutdown(self,*a):pass
 def close(self):
  self.p.stdin.close();self.p.wait(timeout=28);self.reader.join(3)
  if self.reader.is_alive():raise RuntimeError('fixture reader not EOF')
  self.p.stdout.close()

def main():
 pkg=pathlib.Path(sys.argv[0]).resolve().parent;key=pkg.name;root=pkg.parent
 scenario=json.loads((root/'scenario.json').read_text())['scenario']
 action=sys.argv[sys.argv.index('--platform')+2]
 cfg=dict(name=key,cid=64320 if key=='t' else 64321,port=45971 if key=='t' else 45972,gpu_mode='off',timeout=60)
 p=dict(platform='macos',instance_root=str(root/'i'),runtime_profile_reviewed=True,
        request_fault_test=dict(enabled=True,cleanup_authorized=True,instance=key,mode='timeout' if key=='t' else 'disconnect'))
 prepared=pkg/'prepared'/key
 if action=='check':print('{}');return
 if action=='create':
  prepared.mkdir(parents=True)
  (prepared/'record.json').write_text(json.dumps(dict(schema=2,config=cfg,state='PREPARED',profile_sha256=native.sha(pkg/'profiles/macos.json'))))
  (prepared/'launch-plan.json').write_text(json.dumps(dict(gpu_mode='off',config_binding=cfg)));print('{}');return
 owner=None
 real_bind=audit_owner.BoundInstance
 class Owner(mac_owner.MacOwner):
  def __init__(self,p,cfg,plan,run,state):
   native.NativeOwner.__init__(self,p,cfg,plan,run,state);self.w=World();self.tick=0
   self.audit_query=types.SimpleNamespace(retained=[],evidence=lambda:[]);self.log=(run/'fixture.log').open('w')
   self.child=types.SimpleNamespace(pid=99,poll=lambda:0)
   self.owned={pid:{**r,'kind':'crosvm' if r['role']=='guest' else r['role']} for pid,r in self.w.rows.items()}
  def start(self):self.transport=Pipe();self.started=True
  def ready(self):return True
  def observe(self):pass
  def guest_present(self):return True
  def guest_done(self):return False
  def table(self,all_members=False):return self.w.inventory()
  def session_empty(self):return not self.w.rows and self.transport.p.poll()==0
  def snapshot(self):return copy.deepcopy(DISK)
  def bind_linux_fault_members(self):
   if scenario=='null':raise PermissionError('prebind injected EPERM')
   return super().bind_linux_fault_members()
  def finalize_if_terminal(self):
   self.tick+=1
   if self.tick>=1:self.w.rows={}
   if self.tick>=2:self.w.dead={11,12}
   if scenario=='hard':self.w.error=PermissionError('injected EPERM')
   if scenario=='pending':self.w.dead=set()
   if scenario in ('old','null'):return False
   return super().finalize_if_terminal()
 def evidence(w):
  return [dict(pid=pid,token=r['audit_token'],registration_verified=True,exited=pid in w.dead,
      signal_attempted=pid in w.sent,signal_return=0 if pid in w.sent else None,
      kq_closed=pid in w.closed,port_closed=pid in w.closed,
      events=[dict(ident=pid,filter=-5,flags=32817,fflags=0x80000000)] if pid in w.dead else []) for pid,r in w.expected.items()]
 def factory(*a):
  nonlocal owner
  owner=Owner(*a);owner.w.evidence=lambda:evidence(owner.w)
  return owner
 mac_owner.MacOwner=factory
 mac_owner.AuditOps=lambda expected:owner.w
 mac_owner.BoundInstance=lambda expected,inventory,pid,ops:real_bind(expected,inventory,pid,ops,lambda:owner.w.now,owner.w.sleep)
 native.socket=types.SimpleNamespace(create_connection=lambda *a,**k:owner.transport,SHUT_RDWR=2)
 runtime.__file__=str(pkg/'runtime.py');runtime.await_port=lambda *a:None;runtime.probe_port=lambda *a:None
 result={};errors=[]
 def run():
  try:result.update(runtime.run(p,cfg,json.loads((prepared/'record.json').read_text()),prepared,False,lambda p:None,lambda *a:None,lambda *a:{},sys.stdin))
  except BaseException as e:errors.append(repr(e))
 if scenario in ('old','hard','pending','null'):
  t=threading.Thread(target=run,daemon=True);t.start();end=time.monotonic()+18
  while time.monotonic()<end:
   if owner and owner.tick>=3:break
   time.sleep(.05)
  if owner and hasattr(owner,'transport'):owner.transport.close();owner.log.close()
  (prepared/'retention-witness.json').write_text(json.dumps(dict(returned=not t.is_alive(),signals=owner.w.sent,payload_exit=owner.transport.p.returncode,reader_alive=owner.transport.reader.is_alive(),errors=errors)))
  return # injected unresolved kernel, no OS resources discarded
 try:
  worker=threading.Thread(target=run,daemon=True);worker.start();worker.join(23)
  if worker.is_alive():
   (prepared/'fixture-deadline.json').write_text(json.dumps(dict(returned=False,signals=owner.w.sent)))
   raise RuntimeError('local fixture runtime deadline; not acceptance PASS')
  if errors:raise RuntimeError(errors)
 finally:
  if owner:owner.transport.close();owner.log.close()
 (prepared/'proof.json').write_text(json.dumps(dict(signals=owner.w.sent,budget=owner.w.now,payload_exit=owner.transport.p.returncode,reader_alive=owner.transport.reader.is_alive())))
 print(json.dumps(result,indent=2),flush=True)
if __name__=='__main__':main()
