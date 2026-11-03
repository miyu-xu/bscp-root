import unittest,pathlib,json,hashlib,importlib.util,contextlib,uuid
P=pathlib.Path(__file__).resolve().parent
@contextlib.contextmanager
def fixture():
 d=P/('verifier-fixture-'+uuid.uuid4().hex);d.mkdir();yield d
spec=importlib.util.spec_from_file_location('verify_assets',P/'verify_release_assets.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
spec2=importlib.util.spec_from_file_location('verify_installed',P/'verify_installed.py');v=importlib.util.module_from_spec(spec2);spec2.loader.exec_module(v)
class Tests(unittest.TestCase):
 def test_success_and_corruption(self):
  with fixture() as d:
   r=pathlib.Path(d);(r/'asset').write_bytes(b'good');manifest=r/'index.json';manifest.write_text(json.dumps({'assets':[{'file':'asset','bytes':4,'sha256':hashlib.sha256(b'good').hexdigest()}]}));self.assertEqual(m.verify(r,manifest)['result'],'PASS');(r/'asset').write_bytes(b'evil')
   with self.assertRaisesRegex(ValueError,'hash mismatch'):m.verify(r,manifest)
 def test_paths_duplicates_types(self):
  for name,size in [('../escape',1),('C:escape',1),('a\\b',1),('x',True)]:
   with fixture() as d:
    r=pathlib.Path(d);p=r/'index.json';p.write_text(json.dumps({'assets':[{'file':name,'bytes':size,'sha256':'0'*64}]}))
    with self.assertRaises(ValueError):m.verify(r,p)
 def test_duplicate_json_rejected(self):
  with fixture() as d:
   p=pathlib.Path(d)/'config.json';p.write_text('{"platform":"linux","platform":"windows"}')
   with self.assertRaisesRegex(ValueError,'duplicate'):v.load(p)
if __name__=='__main__':unittest.main()
