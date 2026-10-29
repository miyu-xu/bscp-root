import pathlib,sys,unittest,types,tempfile,copy
from unittest.mock import Mock,patch
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[2]/'scripts'/'microdroid_lifecycle'))
import strict_json,runtime
from mac_owner import MacOwner
from native import WindowsOwner
from native import PosixOwner
from test_mac import World

class Unified(unittest.TestCase):
 def test_linux_normal_retains_original_fd_on_deadline(self):
  o=PosixOwner.__new__(PosixOwner);o.p={'platform':'linux'};o.guest_done=lambda:True
  o.owned={11:{'kind':'virtmgr'}};o.live=lambda *a:True;o.observation_evidence={'state':'LIVE'}
  o.child=Mock();o.child.poll.return_value=0;o.session_empty=lambda:True;o.log=Mock();o.run=pathlib.Path('unused-test-only')
  with patch('native.os.pidfd_open',return_value=91,create=True) as opening,patch('native.signal.pidfd_send_signal',create=True) as term,patch('native.select.select',return_value=([],[],[])),patch('native.os.close') as close:
   with self.assertRaises(RuntimeError):o.cleanup_services()
   with self.assertRaises(RuntimeError):o.cleanup_services()
   self.assertEqual(opening.call_count,1);self.assertEqual(term.call_count,1);close.assert_not_called();self.assertEqual(o.service_fds,{11:91})
  with patch('native.select.select',return_value=([91],[],[])),patch('native.os.close') as close:
   self.assertTrue(o.release_service_handles());close.assert_called_once_with(91)
  self.assertFalse(o.service_fds)
 def test_linux_close_error_stays_unknown_no_replay(self):
  o=PosixOwner.__new__(PosixOwner);o.service_fds={11:91}
  with patch('native.select.select',return_value=([91],[],[])),patch('native.os.close',side_effect=OSError('unknown')) as close:
   with self.assertRaises(OSError):o.release_service_handles()
   with self.assertRaises(RuntimeError):o.release_service_handles()
   self.assertEqual(close.call_count,1);self.assertEqual(o.service_fds,{11:91})
 def test_normal_postopen_identity_failure_retains(self):
  o=MacOwner.__new__(MacOwner);o.p={};o.owned={11:{'kind':'virtmgr'}};o.normal_handles={}
  o.normal_ops=Mock();o.normal_ops.expected={};h={'port':7};o.normal_ops.open.return_value=h;o.table=lambda:{}
  with patch.object(PosixOwner,'observe'):
   with self.assertRaises(RuntimeError):o.observe()
   with self.assertRaises(RuntimeError):o.observe()
  self.assertIs(o.normal_handles[11],h);self.assertEqual(o.normal_ops.open.call_count,1)
  self.assertFalse(o.finalize_if_terminal())
 def test_empty_session_partial_handles_not_terminal(self):
  o=MacOwner.__new__(MacOwner);o.p={'request_fault_test':{}};o.owner_terminal=False
  o.session_empty=lambda:True;o.normal_ops=types.SimpleNamespace(retained=[{'port_closed':True,'kq_closed':False}]);o.log=Mock()
  self.assertFalse(o.finalize_if_terminal());o.log.close.assert_not_called()
 def test_real_close_and_receipt_failures_latch(self):
  for stage in ('close','receipt'):
   o=MacOwner.__new__(MacOwner);o.owner_terminal=False;o.child=Mock();o.child.poll.return_value=0
   o.guest_done=lambda:True;o.session_empty=lambda:True;o.log=Mock();o.run=pathlib.Path('unused-test-only')
   o.owned={11:{'kind':'virtmgr'}};h={'pid':11,'port_closed':False};o.normal_handles={11:h}
   def close(handle):
    if stage=='close':raise OSError('close unknown')
    handle['port_closed']=True
   o.normal_ops=types.SimpleNamespace(exited=lambda h:True,close=Mock(side_effect=close),evidence=lambda:[])
   with patch.object(pathlib.Path,'write_text',side_effect=OSError('receipt unknown')):
    with self.assertRaises(OSError):o.cleanup_services()
    with self.assertRaises(RuntimeError):o.cleanup_services()
   self.assertFalse(o.owner_terminal);self.assertFalse(o.finalize_if_terminal());self.assertEqual(o.normal_ops.close.call_count,1)
 def test_partial_normal_bind_never_reopens(self):
  o=MacOwner.__new__(MacOwner);o.p={};o.owned={11:{'kind':'virtmgr'}};o.normal_handles={}
  o.normal_ops=Mock();o.normal_ops.expected={};o.normal_ops.open.side_effect=PermissionError('partial port held')
  with patch.object(PosixOwner,'observe'):
   with self.assertRaises(PermissionError):o.observe()
   with self.assertRaises(RuntimeError):o.observe()
  self.assertEqual(o.normal_ops.open.call_count,1);self.assertFalse(o.finalize_if_terminal())
 def test_partial_normal_close_never_replays(self):
  o=MacOwner.__new__(MacOwner);o.normal_close_blocked=True
  self.assertFalse(o.finalize_if_terminal())
  with self.assertRaises(RuntimeError):o.cleanup_services()
 def test_strict_inputs(self):
  for raw in ('{"a":1,"a":2}','{"a":{"b":1,"b":2}}','{"a":NaN}','{"a":Infinity}'):
   with self.assertRaises(ValueError):strict_json.loads(raw)
  self.assertEqual(strict_json.loads('{"a":false,"b":0}'),{'a':False,'b':0})
 def test_signal_snapshot_race_only_observes(self):
  w=World();o=w.owner();o.bind();w.rows.pop(11)
  r=o.terminate_once(authorized=True,logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
  self.assertEqual(w.sent,[]);self.assertNotIn('identity_observation_blocked',r)
  w.rows={};w.dead={11,12}
  r=o.observe_once(logs_released=lambda:True,storage_snapshot=lambda:{'disk':1})
  self.assertEqual(r['resource_state'],'REAPED');self.assertEqual(w.sent,[])
 def test_mac_normal_service_original_handle(self):
  with tempfile.TemporaryDirectory() as tmp:
   o=MacOwner.__new__(MacOwner);o.run=pathlib.Path(tmp);o.child=Mock();o.child.poll.return_value=0
   o.guest_done=lambda:True;o.session_empty=lambda:True;o.log=Mock();o.owner_terminal=False
   o.owned={11:dict(kind='virtmgr',start=1,sid=9,exe='e',cmd='c',audit_token=[1]),12:dict(kind='crosvm')}
   o.table=lambda:{11:copy.deepcopy(o.owned[11])}
   o.normal_handles={11:dict(pid=11,signal_attempted=False),12:dict(pid=12,signal_attempted=False)}
   dead={12};sent=[];closed=[]
   def term(h):h['signal_attempted']=True;sent.append(h['pid']);dead.add(h['pid'])
   def close(h):closed.append(h['pid']);h['port_closed']=True
   o.normal_ops=types.SimpleNamespace(exited=lambda h:h['pid'] in dead,term=term,close=close,evidence=lambda:[])
   o.cleanup_services();o.cleanup_services()
   self.assertEqual(sent,[11]);self.assertEqual(closed,[11,12]);self.assertTrue(o.owner_terminal)
 def test_mac_normal_identity_rejects_without_signal(self):
  o=MacOwner.__new__(MacOwner);o.guest_done=lambda:True;o.child=Mock();o.child.poll.return_value=0
  o.owned={11:dict(kind='virtmgr',start=1,sid=9,exe='e',cmd='c',audit_token=[1])}
  o.normal_handles={11:dict(signal_attempted=False)};o.table=lambda:{11:{**o.owned[11],'audit_token':[2]}}
  o.normal_ops=Mock();o.normal_ops.exited.return_value=False
  with self.assertRaises(RuntimeError):o.cleanup_services()
  o.normal_ops.term.assert_not_called()
 def test_windows_late_resource_requires_eof_and_storage(self):
  o=WindowsOwner.__new__(WindowsOwner);o.p={'platform':'windows'};o.owner_terminal=True;o.family=types.SimpleNamespace(closed=True);o.cleanup={}
  o.snapshot=Mock(return_value={'disk':1});o.log_receipt={'eof':False,'finished':True,'reader_alive':False}
  r={'task_result':'UNKNOWN','needsRecovery':True,'resource_state':'UNKNOWN'}
  runtime.refresh_terminal_receipts(r,o);self.assertEqual(r['resource_state'],'UNKNOWN');o.snapshot.assert_not_called()
  o.log_receipt['eof']=True;runtime.refresh_terminal_receipts(r,o)
  self.assertEqual(r['resource_state'],'REAPED');self.assertEqual(r['task_result'],'UNKNOWN');self.assertTrue(r['needsRecovery'])
  o.snapshot.side_effect=PermissionError('unverified')
  with self.assertRaises(PermissionError):o.terminal_resource_receipt()

if __name__=='__main__':unittest.main()
