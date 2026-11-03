from pathlib import Path
import subprocess,json,os,hashlib
p=Path(__file__).resolve().parent;tools=p/'slirp-rebuild-tools/python'
out=Path('E:/bscp/out/aosp17-publication-v1827/libslirp-known-source-v2/build')
receipt=p/'SLIRP-SUPERBUILD-BUILD.json';assert not receipt.exists()
assert json.loads((p/'SLIRP-SUPERBUILD-CONFIGURE-v4.json').read_text())['exit']==0
targets=json.loads((out/'meson-info/intro-targets.json').read_text());target=next(x for x in targets if x['name']=='slirp' and x['type']=='shared library')
dll=Path(target['filename'][0]);assert dll.is_relative_to(out)
cmd=[str(tools/'bin/ninja.exe'),'-C',str(out),'-j2',dll.relative_to(out).as_posix()]
env=dict(os.environ);env['PATH']='C:/workspace/mingw64/bin;'+str(tools/'bin')+';C:/Program Files/Git/usr/bin;'+env['PATH'];env['PYTHONPATH']=str(tools)
record={'argv':cmd,'state':'RUNNING','pid':os.getpid(),'scope':'known-source library candidate only; no accepted runtime replacement/VM'}
receipt.write_text(json.dumps(record,indent=2))
with (p/'SLIRP-SUPERBUILD-BUILD.stdout').open('xb') as stdout,(p/'SLIRP-SUPERBUILD-BUILD.stderr').open('xb') as stderr:
 r=subprocess.run(cmd,env=env,stdout=stdout,stderr=stderr,timeout=900)
record.update(state='EXITED',exit=r.returncode)
if r.returncode==0:record['artifact']={'path':str(dll),'sha256':hashlib.sha256(dll.read_bytes()).hexdigest(),'bytes':dll.stat().st_size}
receipt.write_text(json.dumps(record,indent=2));print(json.dumps(record));raise SystemExit(r.returncode)
