"""Fail-closed build gate for a content-addressed CMake bridge output."""
import argparse, hashlib, json, subprocess, sys
from pathlib import Path

def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser()
 for n in ('manifest','stamp','builder','source','out','sdk-root','vc-root'):p.add_argument('--'+n,required=True,type=Path)
 p.add_argument('--sdk-version',required=True);a=p.parse_args()
 # Only this generated success marker is removed, never DLLs/evidence/user data.
 a.stamp.unlink(missing_ok=True);status=a.stamp.parent/'build-status.json'
 def report(state,**extra):status.write_text(json.dumps({'state':state,**extra},indent=2))
 def verify_inputs():
  for line in a.manifest.read_text(encoding='utf-8-sig').splitlines():
   sha,name=line.split('  ',1)
   if digest(Path(name))!=sha:raise RuntimeError('Input changed since CMake configuration: '+name)
 try:
  report('RUNNING');verify_inputs()
  cmd=[sys.executable,'-X','utf8',str(a.builder),'--source',str(a.source),'--out',str(a.out),'--mode','association','--sdk-root',str(a.sdk_root),'--sdk-version',a.sdk_version,'--vc-root',str(a.vc_root)]
  r=subprocess.run(cmd)
  if r.returncode:raise RuntimeError('Bridge builder failed with exit '+str(r.returncode))
  verify_inputs();result=json.loads((a.out/'build-result.json').read_text())
  if len(result)!=3 or any(x.get('exit')!=0 for x in result):raise RuntimeError('Incomplete build result')
  declared=json.loads((a.out/'artifacts.json').read_text());expected=['WindowsLocalRpcBridge.dll','WindowsLocalRpcBridge.lib','WindowsLocalRpc.h','WindowsLocalRpc_c.c','WindowsLocalRpc_s.c','BootstrapCallback.h','BootstrapCallback_c.c','BootstrapCallback_s.c']
  if set(declared)!=set(expected):raise RuntimeError('Unexpected artifact manifest')
  for n in expected:
   if not (a.out/n).is_file() or not (a.out/n).stat().st_size or digest(a.out/n)!=declared[n]:raise RuntimeError('Missing/mismatched artifact: '+n)
  report('SUCCESS',artifacts=declared)
  a.stamp.write_text(json.dumps({'state':'SUCCESS','inputs_manifest_sha256':digest(a.manifest),'artifacts':declared},indent=2));return 0
 except Exception as exc:
  report('FAILED',error=str(exc));print(str(exc),file=sys.stderr);return 1
if __name__=='__main__':raise SystemExit(main())
