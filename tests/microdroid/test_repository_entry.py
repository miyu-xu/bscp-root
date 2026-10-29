import pathlib,sys,tempfile,json,subprocess,unittest,platform,hashlib,shutil
R=pathlib.Path(__file__).resolve().parents[2];sys.path.insert(0,str(R/'scripts/microdroid_lifecycle'))
import entry
from unittest.mock import patch
class Integration(unittest.TestCase):
 def fixture(self,root):
  host={'Windows':'windows','Linux':'linux','Darwin':'macos'}[platform.system()]
  (root/'profiles').mkdir();(root/'closure').mkdir()
  p=dict(schema=1,platform=host,debug='none',gpu=True,payload_port=5558,hashes={},instance_root=str(root/'instances'),closure_manifest='closure/fixed.json',environment={},vm='not-executed',apk='not-executed',launcher='not-executed',dist='not-executed',apex='not-executed',ownership='test')
  (root/'closure/fixed.json').write_text('{}');(root/'profiles'/f'{host}.json').write_text(json.dumps(p));return host,p
 def call(self,root,host,*args):
  return subprocess.run([sys.executable,'-B',str(R/'scripts/microdroid.py'),'--state-root',str(root),'--platform',host,*args],capture_output=True,text=True,timeout=8)
 def test_real_entry_create_mode_old_record_rejection(self):
  with tempfile.TemporaryDirectory(prefix='mi',dir='/tmp' if platform.system()!='Windows' else None) as tmp:
   root=pathlib.Path(tmp).resolve();host,p=self.fixture(root)
   for mode in ('on','off'):
    c=self.call(root,host,'create','--name',mode,'--cid','2048','--port','46001','--gpu',mode);self.assertEqual(c.returncode,0,c.stderr)
    record=root/'prepared'/mode/'record.json';r=json.loads(record.read_text());self.assertEqual(r['candidate_version'],'microdroid-repository-integration-v1');self.assertFalse((root/'instances').exists())
    self.assertEqual(self.call(root,host,'status','--name',mode,'--gpu','off' if mode=='on' else 'on').returncode,2)
    before=record.read_bytes();self.assertEqual(self.call(root,host,'create','--name',mode,'--cid','2048','--port','46001','--gpu',mode).returncode,2);self.assertEqual(before,record.read_bytes())
    r['candidate_version']='microdroid-unified-candidate-v1';record.write_text(json.dumps(r));self.assertEqual(self.call(root,host,'status','--name',mode).returncode,2)
 def test_root_and_closure_escape_rejected(self):
  with tempfile.TemporaryDirectory(prefix='mi',dir='/tmp' if platform.system()!='Windows' else None) as tmp:
   root=pathlib.Path(tmp).resolve();host,p=self.fixture(root);profile=root/'profiles'/f'{host}.json'
   for key,bad in [('instance_root',str(root.parent/'old-instances')),('closure_manifest','../else.json'),('closure_manifest','/else.json'),('closure_manifest','C:/else.json'),('closure_manifest','closure/..\\..\\outside.json')]:
    q=dict(p);q[key]=bad;profile.write_text(json.dumps(q));c=self.call(root,host,'create','--name','a','--cid','2048','--port','46001','--gpu','off');self.assertEqual(c.returncode,2);self.assertFalse((root/'prepared').exists())
 def test_dispatch_help_and_exit(self):
  for host in ('linux','macos'):
   script=R/f'scripts/vm_{host}.sh';s=script.read_text(encoding='utf-8');self.assertIn('exec python3 -B',s);self.assertIn('"$@"',s)
   if platform.system()!='Windows':
    c=subprocess.run(['bash',str(script),'--lifecycle','--help'],capture_output=True,text=True,timeout=8);self.assertEqual(c.returncode,0,c.stderr);self.assertIn('--state-root',c.stdout)
  c=subprocess.run([sys.executable,'-B',str(R/'scripts/microdroid.py'),'--platform','linux','status','--name','a'],capture_output=True,text=True,timeout=8);self.assertEqual(c.returncode,2)
 @unittest.skipUnless(platform.system()=='Windows','actual PowerShell only on Windows')
 def test_powershell_native_error_preference_exit2(self):
  with tempfile.TemporaryDirectory(prefix='mi space ') as tmp:
   script=str(R/'scripts/vm_windows.ps1').replace("'","''");root=str(pathlib.Path(tmp).resolve()).replace("'","''")
   command="$PSNativeCommandUseErrorActionPreference=$true; & '"+script+"' -Command lifecycle -WorkDir '"+root+"' -VmArgs @('status','--name','a'); exit $LASTEXITCODE"
   shells=['powershell']+(['pwsh'] if shutil.which('pwsh') else [])
   for shell in shells:
    with self.subTest(shell=shell):
     c=subprocess.run([shell,'-NoProfile','-Command',command],capture_output=True,text=True,timeout=10)
     self.assertEqual(c.returncode,2,c.stdout+c.stderr);self.assertIn('BLOCKED_OR_FAILED',c.stderr)
if __name__=='__main__':unittest.main()
