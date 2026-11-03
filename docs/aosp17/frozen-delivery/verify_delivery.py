"""Read-only handoff byte/manifest validation; never runs any product or VM."""
from pathlib import Path
import hashlib
import json
import xml.etree.ElementTree as ET

ROOT=Path(__file__).resolve().parent

def verify_source_index(expected):
    path=ROOT/'SOURCE-INDEX.json'
    if not path.exists():
        return {'distributed_fixed_source_complete':False}
    index=json.loads(path.read_bytes())
    if (index['state']!='ALL_FIXED_PROJECT_SNAPSHOTS_AVAILABLE_DISTRIBUTED'
            or index['projects']!=1087 or index['existing_matching_snapshots']!=901
            or index['isolated_clean_checkouts']!=186 or index['original_tree_switched']
            or index['single_repo_managed_clean_tree']):
        raise ValueError('source index completion/boundary mismatch')
    allowed={ (ROOT.parent/name).resolve() for name in [
        'UPSTREAM-LOCK.json','SOURCE-SNAPSHOT-v1807.stdout',
        'SOURCE-BATCH-VERIFY-v1800.stdout','SOURCE-BATCH-VERIFY-v1808.stdout',
        'SOURCE-BATCH-VERIFY-v1812.stdout','SOURCE-BATCH-VERIFY-v1814.stdout',
        'SOURCE-BATCH-VERIFY-v1819.stdout'] }
    allowed.add(Path('E:/bscp/out/aosp17-source-recovery-v1811/result.json').resolve())
    allowed.add(Path('E:/bscp/out/aosp17-source-recovery-v1811/result.next.json').resolve())
    allowed.add(Path('E:/bscp/out/aosp17-source-recovery-v1823/result.json').resolve())
    allowed.add(Path('E:/bscp/out/aosp17-source-recovery-v1825/result.json').resolve())
    seen=set();hashes=set()
    for proof in index['evidence']:
        evidence=Path(proof['path']).resolve(strict=True)
        if evidence not in allowed or evidence in seen:
            raise ValueError('unapproved/duplicate source evidence path')
        seen.add(evidence)
        data=evidence.read_bytes()
        if len(data)!=proof['bytes'] or hashlib.sha256(data).hexdigest()!=proof['sha256']:
            raise ValueError('source evidence changed')
        hashes.add(proof['sha256'])
    if seen!=allowed:raise ValueError('incomplete source evidence set')
    actual={};kinds={}
    for row in index['rows']:
        if row['path'] in actual or row['proof'] not in hashes:
            raise ValueError('duplicate source project or unknown proof')
        actual[row['path']]=(row['name'],row['sha'])
        kinds[row['kind']]=kinds.get(row['kind'],0)+1
    if actual!=expected or kinds!={'existing_matching_fixed_snapshot':901,'isolated_clean_fixed_checkout':186}:
        raise ValueError('source index does not cover exact lock')
    return {'distributed_fixed_source_complete':True,'fixed_source_snapshots_available':len(actual)}

def verify(records, boundary):
    seen=set()
    for record in records:
        path=(ROOT/record['path']).resolve(strict=True)
        if not path.is_relative_to(boundary.resolve()) or path in seen:
            raise ValueError('duplicate or out-of-scope path: '+record['path'])
        seen.add(path)
        data=path.read_bytes()
        if len(data)!=record['bytes'] or hashlib.sha256(data).hexdigest()!=record['sha256']:
            raise ValueError('byte identity mismatch: '+record['path'])
    return len(seen)

def main():
    files=verify(json.loads((ROOT/'DELIVERY-MANIFEST.json').read_bytes())['files'],ROOT)
    refs=verify(json.loads((ROOT/'versions.json').read_bytes())['records'],ROOT.parent)
    lock=json.loads((ROOT/'upstream-lock.json').read_bytes())
    expected={r['path']:(r['name'],r['sha']) for r in lock['projects']}
    actual={}
    for element in ET.fromstring((ROOT/'android17-locked.xml').read_bytes()).findall('project'):
        name=element.attrib['name'];path=element.attrib.get('path',name)
        if path in actual:raise ValueError('duplicate manifest project')
        actual[path]=(name,element.attrib['revision'])
    if actual!=expected or len(actual)!=1087:raise ValueError('manifest lock mismatch')
    print(json.dumps({'delivery_files':files,'original_evidence_references':refs,
                      'pinned_projects':len(actual),'byte_validation':'PASS',
                      'source_checkout_complete':False,'new_runtime_test':False,
                      'publication_performed':False,**verify_source_index(expected)}))

if __name__=='__main__':main()
