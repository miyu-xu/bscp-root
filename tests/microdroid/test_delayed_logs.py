import ast,pathlib,unittest,subprocess,types
from unittest.mock import Mock
P=pathlib.Path(__file__).parent/'delayed_mac.py'
class Logs(unittest.TestCase):
 def function(self,children,result,eof):
  tree=ast.parse(P.read_text());node=next(n for n in ast.walk(tree) if isinstance(n,ast.FunctionDef) and n.name=='logs')
  ns=dict(children=children,result=result,eof_by_pid=eof)
  exec(compile(ast.fix_missing_locations(ast.Module(body=[node],type_ignores=[])),str(P),'exec'),ns)
  return ns['logs']
 def test_partial_eof_not_all(self):
  a=types.SimpleNamespace(pid=1,poll=lambda:0,returncode=0,communicate=Mock(return_value=(b'a',b'')))
  b=types.SimpleNamespace(pid=2,poll=lambda:0,returncode=0,communicate=Mock(side_effect=subprocess.TimeoutExpired('fixture',1)))
  result={};eof={};logs=self.function([a,b],result,eof)
  with self.assertRaises(subprocess.TimeoutExpired):logs()
  self.assertEqual(set(eof),{1})
  with self.assertRaises(RuntimeError):logs()
  self.assertEqual(a.communicate.call_count,1);self.assertEqual(b.communicate.call_count,1)
 def test_each_original_pipe_completed(self):
  children=[types.SimpleNamespace(pid=i,poll=lambda:0,returncode=0,communicate=Mock(return_value=(b'{}',b''))) for i in (1,2)]
  eof={};logs=self.function(children,{},eof)
  self.assertTrue(logs());self.assertTrue(logs());self.assertEqual(set(eof),{1,2})
  self.assertTrue(all(c.communicate.call_count==1 for c in children))
if __name__=='__main__':unittest.main()
