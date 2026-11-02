"""Validate publication snapshot bytes, without executing downloaded source."""
from pathlib import Path
import argparse,hashlib,json

def pairs(index):
    if 'source_files' in index:
        return [('sources/'+r['path'],r) for r in index['source_files']]
    rows=index['files']
    return [(r['path'],r) for r in rows] if isinstance(rows,list) else list(rows.items())

def verify(root):
    count=0;closures=[]
    for directory in sorted(root.iterdir()):
        if not directory.is_dir():continue
        index=json.loads((directory/'SOURCE-CLOSURE.json').read_bytes())
        seen=set()
        for name,row in pairs(index):
            rel=Path(name)
            if rel.is_absolute() or '..' in rel.parts or name in seen:raise ValueError('invalid or duplicate snapshot path')
            seen.add(name);path=directory/rel
            if path.is_symlink() or not path.resolve().is_relative_to(directory.resolve()):raise ValueError('source escaped snapshot')
            data=path.read_bytes()
            if len(data)!=row['bytes'] or hashlib.sha256(data).hexdigest()!=row['sha256']:raise ValueError('snapshot byte mismatch: '+name)
            count+=1
        closures.append({'name':directory.name,'files':len(seen),'scope':index['scope']})
    if not closures:raise ValueError('no source closures')
    return {'files':count,'closures':closures,'source_byte_verification':'PASS','rebuild_or_runtime_verification':False}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--source-root',type=Path,default=Path(__file__).parent/'source')
    print(json.dumps(verify(p.parse_args().source_root),indent=2))
