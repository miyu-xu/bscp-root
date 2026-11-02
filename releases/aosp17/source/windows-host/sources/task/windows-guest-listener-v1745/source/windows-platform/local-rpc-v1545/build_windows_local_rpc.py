#!/usr/bin/env python3
"""Build the Windows-local C ABI bridge with installed official MIDL/MSVC tools.
No package installation, generated-source edits, C++ ABI crossing or service start.
Only isolated mode exists until production bootstrap is independently integrated.
Use a dedicated output directory. Existing output must match this exact recipe.
"""
import argparse, hashlib, json, os, subprocess
from pathlib import Path

def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--source',required=True,type=Path);p.add_argument('--out',required=True,type=Path);p.add_argument('--mode',required=True,choices=['isolated']);p.add_argument('--sdk-root',type=Path,default=Path('C:/Program Files (x86)/Windows Kits/10'));p.add_argument('--sdk-version',default='10.0.26100.0');p.add_argument('--vc-root',type=Path,default=Path('C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.44.35207'));a=p.parse_args()
 source=a.source.resolve(strict=True);out=a.out.resolve();sdk=a.sdk_root.resolve(strict=True);vc=a.vc_root.resolve(strict=True)
 names=['WindowsLocalRpc.idl','WindowsLocalRpc.acf','WindowsLocalRpcBridge.c','WindowsLocalRpcBridge.h'];inputs={n:digest(source/n) for n in names};midl=sdk/'bin'/a.sdk_version/'x64/midl.exe';cl=vc/'bin/Hostx64/x64/cl.exe'
 recipe={'mode':a.mode,'source':str(source),'inputs':inputs,'sdk':str(sdk),'sdk_version':a.sdk_version,'vc':str(vc),'tools':{str(midl):digest(midl),str(cl):digest(cl)},'C_ABI_only':True}
 if out==source or source in out.parents:raise SystemExit('Output must be independent of source')
 if out.exists() and any(out.iterdir()):
  existing=out/'build-recipe.json'
  if not existing.is_file() or json.loads(existing.read_text())!=recipe:raise SystemExit('Output contains a different/unknown recipe; use a new dedicated directory')
 out.mkdir(parents=True,exist_ok=True);(out/'build-recipe.json').write_text(json.dumps(recipe,indent=2))
 env=os.environ.copy();env['PATH']=';'.join([str(vc/'bin/Hostx64/x64'),str(sdk/'bin'/a.sdk_version/'x64'),env.get('PATH','')]);env['INCLUDE']=';'.join(str(x) for x in [vc/'include',sdk/'Include'/a.sdk_version/'ucrt',sdk/'Include'/a.sdk_version/'shared',sdk/'Include'/a.sdk_version/'um']);env['LIB']=';'.join(str(x) for x in [vc/'lib/x64',sdk/'Lib'/a.sdk_version/'ucrt/x64',sdk/'Lib'/a.sdk_version/'um/x64']);env.update(TEMP=str(out),TMP=str(out))
 commands=[('midl',[str(midl),'/nologo','/env','x64','/target','NT100','/prefix','client','Client_','/prefix','server','Server_','/acf',str(source/'WindowsLocalRpc.acf'),'/out',str(out),str(source/'WindowsLocalRpc.idl')]),('bridge',[str(cl),'/nologo','/TC','/LD','/MD','/W4','/WX','/DBSCP_WLR_ISOLATED_COMPONENT','/I'+str(out),'/I'+str(source),str(source/'WindowsLocalRpcBridge.c'),str(out/'WindowsLocalRpc_c.c'),str(out/'WindowsLocalRpc_s.c'),'/Fe:'+str(out/'WindowsLocalRpcBridge.dll'),'/link','rpcrt4.lib','advapi32.lib','bcrypt.lib','/IMPLIB:'+str(out/'WindowsLocalRpcBridge.lib')])]
 result=[]
 for name,cmd in commands:
  with (out/(name+'.stdout')).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:r=subprocess.run(cmd,cwd=out,env=env,stdout=stdout,stderr=stderr)
  result.append({'stage':name,'command':cmd,'exit':r.returncode});(out/'build-result.json').write_text(json.dumps(result,indent=2));print(name,r.returncode,flush=True)
  if r.returncode:return r.returncode
 outputs=['WindowsLocalRpcBridge.dll','WindowsLocalRpcBridge.lib','WindowsLocalRpc.h','WindowsLocalRpc_c.c','WindowsLocalRpc_s.c'];(out/'artifacts.json').write_text(json.dumps({n:digest(out/n) for n in outputs},indent=2));return 0
if __name__=='__main__':raise SystemExit(main())
