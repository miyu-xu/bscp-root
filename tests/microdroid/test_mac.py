import sys,pathlib,unittest,copy,types,errno
from unittest.mock import patch,Mock
P=pathlib.Path(__file__).resolve().parents[2]/'scripts'/'microdroid_lifecycle';sys.path.insert(0,str(P))
from audit_owner import BoundInstance
from mac_owner import MacOwner
from mac_audit import AuditOps
from native import PosixOwner

class World:
 def __init__(self):
  self.rows={p:dict(start='start',sid=99,exe='/test',cmd='test',audit_token=[0,0,0,0,0,p,1,100+p],role=r) for p,r in ((11,'virtmgr'),(12,'guest'))}
  self.expected=copy.deepcopy(self.rows);self.dead=set();self.sent=[];self.closed=[];self.now=0;self.error=None;self.opened=[]
 def inventory(self):
  if self.error:raise self.error
  return copy.deepcopy(self.rows)
 def open(self,p):self.opened.append(p);return p
 def exited(self,p):return p in self.dead
 def term(self,p):self.sent.append(p)
 def close(self,p):self.closed.append(p)
 def sleep(self,t):self.now+=t
 def owner(self):return BoundInstance(self.expected,self.inventory,1,self,lambda:self.now,self.sleep)

class Tests(unittest.TestCase):
 def test_delayed_original_handles(self):
  w=World();o=w.owner();o.bind();r=o.terminate_once(authorized=True,logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
  self.assertEqual(w.sent,[11,12]);self.assertTrue(r['observation_expired']);self.assertAlmostEqual(w.now,5)
  w.rows={};r=o.observe_once(logs_released=lambda:True,storage_snapshot=lambda:{'disk':1});self.assertEqual(r['resource_state'],'UNKNOWN')
  w.dead={11,12};r=o.observe_once(logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
  self.assertEqual(r['resource_state'],'REAPED');self.assertEqual(r['task_result'],'UNKNOWN');self.assertTrue(r['needsRecovery']);self.assertEqual(w.closed,[11,12]);self.assertEqual(w.sent,[11,12])
  with self.assertRaises(RuntimeError):o.terminate_once(authorized=True,logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
 def test_generation_and_new_member(self):
  for change in ('token','new'):
   w=World();o=w.owner();o.bind()
   if change=='token':w.rows[11]['audit_token'][-1]+=1
   else:w.rows[13]=dict(w.rows[12])
   r=o.terminate_once(authorized=True,logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
   self.assertEqual(w.sent,[]);self.assertTrue(r['identity_observation_blocked'])
 def test_hard_error_latched(self):
  for e in (PermissionError(errno.EPERM,'deny'),OSError(errno.EIO,'io')):
   w=World();o=w.owner();o.bind();w.error=e
   o.terminate_once(authorized=True,logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
   w.error=None;w.rows={};w.dead={11,12}
   self.assertEqual(o.observe_once(logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})['resource_state'],'UNKNOWN');self.assertEqual(w.sent,[])
 def test_partial_bind_retains(self):
  w=World();o=w.owner();real=w.open
  def opening(p):
   if p==12:raise PermissionError('denied')
   return real(p)
  w.open=opening
  with self.assertRaises(PermissionError):o.bind()
  self.assertEqual(o.fds,{11:11});self.assertFalse(o.armed);self.assertEqual(w.sent,[])
 def test_signal_failure_no_retry(self):
  w=World();o=w.owner();o.bind()
  def fail(p):w.sent.append(p);raise PermissionError('denied')
  w.term=fail;r=o.terminate_once(authorized=True,logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
  self.assertEqual(w.sent,[11]);self.assertTrue(r['identity_observation_blocked'])
 def test_logs_and_disk_not_substitutes(self):
  for logs,disk in ((False,{'disk':1}),(True,{})):
   w=World();o=w.owner();o.bind();o.cleanup_attempted=True;w.rows={};w.dead={11,12}
   r=o.observe_once(logs_released=lambda:logs,storage_snapshot=lambda:disk)
   self.assertEqual(r['resource_state'],'UNKNOWN');self.assertEqual(w.closed,[])
 def test_start_preflight(self):
  o=MacOwner.__new__(MacOwner)
  with patch.object(PosixOwner,'table',return_value={'test':'preflight'}) as m:self.assertEqual(o.table(),{'test':'preflight'});m.assert_called_once()
 def test_parent_not_identity(self):
  o=MacOwner.__new__(MacOwner);a=dict(start=1,sid=2,exe='e',cmd='c',audit_token=[1],ppid=3);b={**a,'ppid':1}
  self.assertTrue(o.same_identity(a,b));b['audit_token']=[2];self.assertFalse(o.same_identity(a,b))
 def test_partial_query_blocks_terminal(self):
  o=MacOwner.__new__(MacOwner);o.child=Mock();o.child.poll.return_value=0;o.ps_rows=lambda:{};o.audit_query=types.SimpleNamespace(retained=[{'port_closed':False}])
  self.assertFalse(o.session_empty());o.audit_query.retained[0]['port_closed']=True;self.assertTrue(o.session_empty())
 def test_no_ordinary_service_term(self):
  o=MacOwner.__new__(MacOwner);o.fault_owner=object()
  with self.assertRaises(RuntimeError):o.cleanup_services()
 def test_retention_observe_bound_no_signal(self):
  o=MacOwner.__new__(MacOwner);o.owner_terminal=False;o.p={'platform':'macos'};o.fault_owner=Mock(armed=True,cleanup_attempted=True);o.fault_owner.observe_once.return_value={'resource_state':'UNKNOWN'};o.fault_owner.ops.evidence.return_value=[]
  with patch('native.time.monotonic',return_value=10):
   for _ in range(40):self.assertFalse(o.finalize_if_terminal())
  self.assertEqual(o.fault_owner.observe_once.call_count,30);o.fault_owner.terminate_once.assert_not_called()
 def test_audit_signal_once_and_token_change(self):
  o=AuditOps.__new__(AuditOps);o.u=__import__('ctypes').c_uint32;o.proc=Mock();o.proc.proc_signal_with_audittoken.return_value=0
  h=dict(pid=1,token=[0]*8,exe='x',signal_attempted=False,registration_verified=True)
  o.exited=lambda h:False;o.token=lambda h:h['token'];o.path=lambda t:'x'
  o.term(h)
  with self.assertRaises(RuntimeError):o.term(h)
  self.assertEqual(o.proc.proc_signal_with_audittoken.call_count,1)
  h['signal_attempted']=False;o.token=lambda h:[1]*8
  with self.assertRaises(RuntimeError):o.term(h)
  self.assertFalse(h['signal_attempted'])
 def test_partial_handle_evidence(self):
  o=AuditOps.__new__(AuditOps);o.retained=[dict(pid=1,port=9,kq=None,port_closed=False)]
  self.assertEqual(o.evidence(),[dict(pid=1,port=9,port_closed=False)])
 def test_query_errors_never_erased_by_later_exit(self):
  for err in (PermissionError(errno.EPERM,'denied'),OSError(errno.EIO,'io'),RuntimeError('Mach result 5')):
   o=MacOwner.__new__(MacOwner);o.child=types.SimpleNamespace(pid=99);o.run=pathlib.Path('/run')
   row=dict(ppid=1,sid=99,start='s',cmd='/crosvm --cid 3')
   o.ps_rows=Mock(return_value={12:row});o.audit_query=types.SimpleNamespace(identity=Mock(side_effect=err))
   with self.assertRaises(type(err)):o.table(all_members=True)
 def test_acquire_failure_releases_only_known_port(self):
  import ctypes
  o=AuditOps.__new__(AuditOps);o.u=ctypes.c_uint32;o.own=1;o.retained=[];o.lib=Mock()
  def acquire(a,p,ptr):ptr._obj.value=123;return 0
  o.lib.task_name_for_pid.side_effect=acquire;o.lib.mach_port_deallocate.return_value=0
  o.token=Mock(side_effect=RuntimeError('Mach info failed'))
  with self.assertRaises(RuntimeError):o.acquire(12)
  self.assertEqual(o.retained,[]);o.lib.mach_port_deallocate.assert_called_once_with(1,123)
  o.lib.mach_port_deallocate.return_value=5
  with self.assertRaises(RuntimeError):o.acquire(12)
  self.assertEqual(len(o.retained),1);self.assertTrue(o.retained[0]['port_close_attempted']);self.assertNotIn('port_closed',o.retained[0])
 def test_actual_open_registration_and_generation(self):
  import mac_audit
  for changed in (False,True):
   o=AuditOps.__new__(AuditOps);h=dict(pid=12,port=3,token=[0]*8,exe='/x',kq=None)
   o.expected={12:dict(audit_token=[0]*8,exe='/x')};o.acquire=lambda p:h
   o.token=lambda h:([1]*8 if changed else [0]*8);o.path=lambda t:'/x'
   q=Mock();constants=dict(KQ_FILTER_PROC=-5,KQ_EV_ADD=1,KQ_EV_ONESHOT=16,KQ_NOTE_EXIT=0x80000000)
   with patch.multiple(mac_audit.select,create=True,**constants),patch.object(mac_audit.select,'kqueue',return_value=q,create=True),patch.object(mac_audit.select,'kevent',return_value='registration',create=True):
    if changed:
     with self.assertRaises(RuntimeError):o.open(12)
     self.assertNotIn('registration_verified',h)
    else:self.assertIs(o.open(12),h);self.assertTrue(h['registration_verified'])
    q.control.assert_called_once_with(['registration'],0,0)
 def test_actual_event_latch_and_close_errors(self):
  import mac_audit
  for event,valid in ((dict(ident=12,filter=-5,flags=16,fflags=0x80000000),True),(dict(ident=99,filter=-5,flags=16,fflags=0x80000000),False),(dict(ident=12,filter=-5,flags=0x4000,fflags=0),False)):
   o=AuditOps.__new__(AuditOps);q=Mock();q.control.return_value=[types.SimpleNamespace(**event)]
   h=dict(pid=12,kq=q,exited=False,events=[],kq_close_attempted=False)
   with patch.multiple(mac_audit.select,create=True,KQ_FILTER_PROC=-5,KQ_EV_ERROR=0x4000,KQ_NOTE_EXIT=0x80000000):
    if not valid:
     with self.assertRaises(RuntimeError):o.exited(h)
     self.assertFalse(h['exited']);continue
    self.assertTrue(o.exited(h));self.assertTrue(o.exited(h));self.assertEqual(q.control.call_count,1)
    o.release_port=Mock(side_effect=RuntimeError('port close uncertain'))
    with self.assertRaises(RuntimeError):o.close(h)
    with self.assertRaises(RuntimeError):o.close(h)
    self.assertEqual(q.close.call_count,1)

if __name__=='__main__':unittest.main()
