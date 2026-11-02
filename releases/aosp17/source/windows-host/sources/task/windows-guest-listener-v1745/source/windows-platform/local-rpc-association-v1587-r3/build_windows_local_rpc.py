#!/usr/bin/env python3
"""Build the Windows-local C ABI bridge with installed official MIDL/MSVC tools.
No package installation, generated-source edits, C++ ABI crossing or service start.
Association mode builds the complete reviewed facade with no test hooks.
Isolated mode remains for legacy source sets only; no implicit mode upgrade.
Use a dedicated output directory. Existing output must match this exact recipe.
"""
import argparse, hashlib, json, os, subprocess, shutil
from pathlib import Path

def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 p=argparse.ArgumentParser();p.add_argument('--source',required=True,type=Path);p.add_argument('--out',required=True,type=Path);p.add_argument('--mode',required=True,choices=['isolated','association']);p.add_argument('--sdk-root',type=Path,default=Path('C:/Program Files (x86)/Windows Kits/10'));p.add_argument('--sdk-version',default='10.0.26100.0');p.add_argument('--vc-root',type=Path,default=Path('C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.44.35207'));a=p.parse_args()
 source=a.source.resolve(strict=True);out=a.out.resolve();sdk=a.sdk_root.resolve(strict=True);vc=a.vc_root.resolve(strict=True)
 names=['WindowsLocalRpc.idl','WindowsLocalRpc.acf','WindowsLocalRpcBridge.c','WindowsLocalRpcBridge.h'];
 if a.mode=='association':names+=['InitialBootstrap.c','InitialBootstrap.h','BootstrapCallback.idl']
 elif (source/'InitialBootstrap.c').exists():raise SystemExit('Association source requires explicit --mode association')
 inputs={n:digest(source/n) for n in names};midl=sdk/'bin'/a.sdk_version/'x64/midl.exe';cl=vc/'bin/Hostx64/x64/cl.exe'
 recipe={'mode':a.mode,'source':str(source),'inputs':inputs,'sdk':str(sdk),'sdk_version':a.sdk_version,'vc':str(vc),'tools':{str(midl):digest(midl),str(cl):digest(cl)},'C_ABI_only':True,'recipe_version':3,'clean_compile_source_snapshot':True,'builder_sha256':digest(Path(__file__).resolve()),'test_hooks':False}
 if out==source or source in out.parents:raise SystemExit('Output must be independent of source')
 if out.exists() and any(out.iterdir()):
  existing=out/'build-recipe.json'
  if not existing.is_file() or json.loads(existing.read_text())!=recipe:raise SystemExit('Output contains a different/unknown recipe; use a new dedicated directory')
 out.mkdir(parents=True,exist_ok=True)
 # Preserve previous attempts; a failed rerun cannot inherit their success manifest.
 previous=[x for x in out.iterdir() if x.is_file() and (x.suffix in ('.stdout','.stderr') or x.name in ('build-result.json','artifacts.json'))]
 if previous:
  attempt=1
  while (out/('previous-attempt-'+str(attempt))).exists():attempt+=1
  archive=out/('previous-attempt-'+str(attempt));archive.mkdir()
  for x in previous:shutil.copy2(x,archive/x.name)
 (out/'artifacts.json').unlink(missing_ok=True)
 (out/'build-result.json').write_text('[]')
 (out/'build-recipe.json').write_text(json.dumps(recipe,indent=2))
 env=os.environ.copy();env['PATH']=';'.join([str(vc/'bin/Hostx64/x64'),str(sdk/'bin'/a.sdk_version/'x64'),env.get('PATH','')]);env['INCLUDE']=';'.join(str(x) for x in [vc/'include',sdk/'Include'/a.sdk_version/'ucrt',sdk/'Include'/a.sdk_version/'shared',sdk/'Include'/a.sdk_version/'um']);env['LIB']=';'.join(str(x) for x in [vc/'lib/x64',sdk/'Lib'/a.sdk_version/'ucrt/x64',sdk/'Lib'/a.sdk_version/'um/x64']);env.update(TEMP=str(out),TMP=str(out));
 for injected in ('CL','_CL_','LINK','LIBPATH'):env.pop(injected,None)
 commands=[('midl',[str(midl),'/nologo','/env','x64','/target','NT100','/prefix','client','Client_','/prefix','server','Server_','/acf',str(source/'WindowsLocalRpc.acf'),'/out',str(out),str(source/'WindowsLocalRpc.idl')])]
 generated=[out/'WindowsLocalRpc_c.c',out/'WindowsLocalRpc_s.c']
 if a.mode=='association':
  commands.append(('midl-callback',[str(midl),'/nologo','/env','x64','/target','NT100','/prefix','client','CbClient_','/prefix','server','CbServer_','/out',str(out),str(source/'BootstrapCallback.idl')]))
  generated += [out/'BootstrapCallback_c.c',out/'BootstrapCallback_s.c']
 # Quoted includes search the TU directory before /I. Never consume stale
 # generated headers that may be present in a manually built source directory.
 compile_source=out/'source-inputs';compile_source.mkdir(exist_ok=True)
 if any(x.name not in names or not x.is_file() for x in compile_source.iterdir()):raise SystemExit('Unknown file in clean compile source snapshot')
 for name in names:shutil.copy2(source/name,compile_source/name)
 translation=compile_source/('InitialBootstrap.c' if a.mode=='association' else 'WindowsLocalRpcBridge.c')
 define='/DBSCP_WLR_ASSOCIATION_CORE' if a.mode=='association' else '/DBSCP_WLR_ISOLATED_COMPONENT'
 commands.append(('bridge',[str(cl),'/nologo','/TC','/LD','/MD','/W4','/WX',define,'/I'+str(out),'/I'+str(compile_source),str(translation)]+list(map(str,generated))+['/Fe:'+str(out/'WindowsLocalRpcBridge.dll'),'/link','rpcrt4.lib','advapi32.lib','bcrypt.lib','/IMPLIB:'+str(out/'WindowsLocalRpcBridge.lib')]))

 result=[]
 for name,cmd in commands:
  with (out/(name+'.stdout')).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:r=subprocess.run(cmd,cwd=out,env=env,stdout=stdout,stderr=stderr)
  result.append({'stage':name,'command':cmd,'exit':r.returncode});(out/'build-result.json').write_text(json.dumps(result,indent=2));print(name,r.returncode,flush=True)
  if r.returncode:return r.returncode
 outputs=['WindowsLocalRpcBridge.dll','WindowsLocalRpcBridge.lib','WindowsLocalRpc.h','WindowsLocalRpc_c.c','WindowsLocalRpc_s.c']
 if a.mode=='association':outputs+=['BootstrapCallback.h','BootstrapCallback_c.c','BootstrapCallback_s.c']
 (out/'artifacts.json').write_text(json.dumps({n:digest(out/n) for n in outputs},indent=2));return 0
if __name__=='__main__':raise SystemExit(main())
