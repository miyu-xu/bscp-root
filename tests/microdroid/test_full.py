import pathlib,sys,uuid,json,unittest,importlib.util
from unittest.mock import patch
HERE=pathlib.Path(__file__).resolve().parent;sys.path.insert(0,str(HERE.parents[1]/'scripts'/'microdroid_lifecycle'))
import native
from full_fixture import DISK
s=importlib.util.spec_from_file_location('driver',HERE/'accept.py');driver=importlib.util.module_from_spec(s);s.loader.exec_module(driver)
class Full(unittest.TestCase):
 def one(self,scenario,missing=False):
  root=HERE/'full-evidence'/uuid.uuid4().hex;root.mkdir(parents=True)
  (root/'scenario.json').write_text(json.dumps(dict(scenario=scenario)))
  for key in ('t','d'):
   pkg=root/key;(pkg/'profiles').mkdir(parents=True);(pkg/'profiles/macos.json').write_text('{}')
   (pkg/'entry.py').write_text('import sys\nsys.path.insert(0,'+repr(str(HERE))+')\nfrom full_fixture import main\nmain()\n')
  original=driver.Reader
  class Reader(original):
   def receipt(self):
    r=super().receipt()
    if missing:r['eof']=False
    return r
  done=[];error=None
  with patch.object(driver,'ROOT',root),patch.object(driver,'Reader',Reader),patch.object(native.PosixOwner,'snapshot',return_value=DISK):
   try:
    if driver.case('t',64320,45971):done.append('t')
    if driver.case('d',64321,45972):done.append('d')
   except RuntimeError as e:error=str(e)
  (root/'result.json').write_text(json.dumps(dict(done=done,error=error,missing_outer_eof=missing)))
  return root,done,error
 def test_full_timeout_disconnect_delayed(self):
  root,done,error=self.one('late');self.assertEqual(done,['t','d']);self.assertIsNone(error)
  for key in done:
   p=json.loads((root/key/'prepared'/key/'proof.json').read_text());self.assertEqual(p,dict(signals=[11,12],budget=5.0,payload_exit=0,reader_alive=False))
 def test_unresolved_no_second(self):
  for scenario in ('old','hard','pending','null'):
   with self.subTest(scenario=scenario):
    root,done,error=self.one(scenario);self.assertEqual(done,[]);self.assertIsNotNone(error);self.assertFalse((root/'evidence/d').exists())
    p=json.loads((root/'t/prepared/t/retention-witness.json').read_text());self.assertFalse(p['returned']);self.assertEqual(p['payload_exit'],0);self.assertFalse(p['reader_alive'])
 def test_missing_outer_eof_no_second(self):
  root,done,error=self.one('late',True);self.assertEqual(done,[]);self.assertIsNotNone(error);self.assertFalse((root/'evidence/d').exists())
if __name__=='__main__':unittest.main()
