"""Check downloaded release files; never downloads, extracts, installs or executes them."""
import argparse,hashlib,json,re
from pathlib import Path

def unique(pairs):
    result={}
    for k,v in pairs:
        if k in result:raise ValueError('duplicate JSON key')
        result[k]=v
    return result

def verify(directory,manifest):
    data=json.loads(manifest.read_text(encoding='utf-8'),object_pairs_hook=unique)
    rows=data['assets'];names=set()
    if not isinstance(rows,list) or not rows:raise ValueError('empty/invalid asset list')
    for row in rows:
        name=row['file'];size=row['bytes'];digest=row['sha256']
        if not isinstance(name,str) or not name or name in ('.','..') or '/' in name or '\\' in name or ':' in name or name in names:raise ValueError('invalid/duplicate asset name')
        names.add(name)
        if type(size) is not int or size<0 or not isinstance(digest,str) or not re.fullmatch('[0-9a-f]{64}',digest):raise ValueError('invalid asset identity')
        p=directory/name
        if p.is_symlink() or not p.is_file() or p.stat().st_size!=size:raise ValueError('missing/linked/wrong-size asset: '+name)
        with p.open('rb') as f:
            h=hashlib.sha256()
            for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
        if h.hexdigest()!=digest:raise ValueError('asset hash mismatch: '+name)
    return {'verified_assets':len(rows),'result':'PASS','executed_or_extracted':False,'privacy_scrubbed':False}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--manifest',type=Path,required=True);a=p.parse_args();print(json.dumps(verify(a.directory,a.manifest)))
