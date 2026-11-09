"""Verify exact inputs and create a new partial source overlay; never edit input trees."""
import argparse,hashlib,json,pathlib,subprocess,shutil
B=pathlib.Path(__file__).resolve().parent

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 a=argparse.ArgumentParser();a.add_argument('--host',type=pathlib.Path,required=True);a.add_argument('--guest',type=pathlib.Path,required=True);a.add_argument('--output',type=pathlib.Path,required=True);a=a.parse_args()
 m=json.loads((B/'MANIFEST.json').read_text());out=a.output.absolute()
 if out.exists():raise RuntimeError('Output already exists; no overwrite/replay')
 inputs=[]
 for kind in ['host','guest']:
  root=getattr(a,kind).resolve(strict=True)
  for r in m['files'][kind]:
   rel=pathlib.PurePosixPath(r['path'])
   if rel.is_absolute() or '..' in rel.parts:raise RuntimeError('Unsafe manifest path')
   f=(root/pathlib.Path(rel)).resolve(strict=True)
   if not f.is_relative_to(root) or sha(f)!=r['before_sha256']:raise RuntimeError('Baseline mismatch: '+kind+'/'+str(rel))
   inputs.append((kind,rel,f))
 out.mkdir(parents=False)
 for kind,rel,f in inputs:
  dest=out/kind/pathlib.Path(rel);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(f,dest)
 for kind in ['host','guest']:
  patch=B/'patches'/(kind+'-combined.patch')
  for mode in [['--check'],[]]:subprocess.run(['git','apply',*mode,str(patch)],cwd=out/kind,check=True,capture_output=True)
  for r in m['files'][kind]:
   target=out/kind/r['path']
   if r['after_line_endings']=='CRLF':
    target.write_bytes(target.read_bytes().replace(b'\r\n',b'\n').replace(b'\n',b'\r\n'))
   elif r['after_line_endings']=='LF':
    target.write_bytes(target.read_bytes().replace(b'\r\n',b'\n'))
   else:
    raise RuntimeError('Unsupported output line-ending contract')
   if sha(out/kind/r['path'])!=r['after_sha256']:raise RuntimeError('Output mismatch: '+r['path'])
 (out/'APPLIED.json').write_text(json.dumps({'all_hashes_match':True,'scope':'partial overlay only; inputs unchanged','format':1},indent=2)+'\n')
 print('PASS: exact baseline -> verified partial overlays; inputs unchanged')
if __name__=='__main__':main()
