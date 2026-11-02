from pathlib import Path
import subprocess,json,os,sys
p=Path(__file__).resolve().parent;tools=p/'slirp-rebuild-tools/python'
source=p/'slirp-rebuild-sources/bscp-slirp-superbuild-v1'
out=Path('E:/bscp/out/aosp17-publication-v1827/libslirp-known-source-v2/build')
assert sys.argv[1:] in ([],['resume-v2'],['resume-v3'],['resume-v4'])
suffix='-'+sys.argv[1].removeprefix('resume-') if sys.argv[1:] else ''
previous={'-v2':'','-v3':'-v2','-v4':'-v3'}.get(suffix,'')
if suffix:assert json.loads((p/('SLIRP-SUPERBUILD-CONFIGURE'+previous+'.json')).read_text())['exit']==1
else:assert not out.exists()
out.parent.mkdir(parents=True,exist_ok=True)
env=dict(os.environ);env['PYTHONPATH']=str(tools)
env['PATH']='C:/workspace/mingw64/bin;'+str(tools/'bin')+';C:/Program Files/Git/usr/bin;'+env['PATH']
env['CC']='C:/workspace/mingw64/bin/gcc.exe';env['CXX']='C:/workspace/mingw64/bin/g++.exe';env['NINJA']=str(tools/'bin/ninja.exe')
cmd=[sys.executable,'-m','mesonbuild.mesonmain','setup',str(out),str(source),'--buildtype=release','--wrap-mode=nodownload','--prefix',str(out.parent/'install')]
record={'argv':cmd,'state':'RUNNING','pid':os.getpid(),'scope':'official-source superbuild configure only; no VM/runtime replacement'}
receipt=p/('SLIRP-SUPERBUILD-CONFIGURE'+suffix+'.json');assert not receipt.exists();receipt.write_text(json.dumps(record,indent=2))
with (p/('SLIRP-SUPERBUILD-CONFIGURE'+suffix+'.stdout')).open('xb') as stdout,(p/('SLIRP-SUPERBUILD-CONFIGURE'+suffix+'.stderr')).open('xb') as stderr:
 r=subprocess.run(cmd,env=env,stdout=stdout,stderr=stderr,timeout=600)
record.update(state='EXITED',exit=r.returncode);receipt.write_text(json.dumps(record,indent=2));print(json.dumps(record));raise SystemExit(r.returncode)
