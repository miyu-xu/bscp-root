import unittest,sys,pathlib,types
from unittest.mock import Mock,patch
HERE=pathlib.Path(__file__).parent;sys.path.insert(0,str(HERE.parents[1]/'scripts'/'microdroid_lifecycle'));import harmless
class Helper(unittest.TestCase):
 def test_report_failure_keeps_retention(self):
  with patch('builtins.print',side_effect=BrokenPipeError()),patch.object(harmless.time,'sleep',side_effect=RuntimeError('retention reached')) as sleep:
   with self.assertRaisesRegex(RuntimeError,'retention reached'):harmless.retain({'pass':False})
   sleep.assert_called_once()
 def test_partial_known_port_disposal_after_child_exit(self):
  h=dict(pid=3,kq=None,port_closed=False);ops=types.SimpleNamespace(retained=[h])
  ops.release_port=lambda h:h.update(port_closed=True)
  self.assertTrue(harmless.close_owned(ops,[types.SimpleNamespace(pid=3,poll=lambda:0)],{}))
 def test_uncertain_close_retains_no_replay(self):
  h=dict(pid=3,kq=Mock(),kq_closed=False,kq_close_attempted=True,port_closed=False)
  ops=types.SimpleNamespace(retained=[h],release_port=Mock());r={}
  self.assertFalse(harmless.close_owned(ops,[types.SimpleNamespace(pid=3,poll=lambda:0)],r))
  h['kq'].close.assert_not_called();ops.release_port.assert_not_called();self.assertTrue(r['close_errors'])
if __name__=='__main__':unittest.main()
