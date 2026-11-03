"""Read-only check of a private, already admitted deployment; never installs or starts a VM."""
from pathlib import Path
import argparse,hashlib,json,platform,subprocess,sys

def load(path):
    def unique(pairs):
        out={}
        for k,v in pairs:
            if k in out:raise ValueError('duplicate JSON key')
            out[k]=v
        return out
    return json.loads(path.read_bytes(),object_pairs_hook=unique)

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--platform',choices=['windows','linux','macos'],required=True)
    p.add_argument('--deployment-config',type=Path,required=True,help='Private directory containing the original admitted BINDING.json, profile.json and closure.json; not a template or new admission')
    a=p.parse_args();actual={'Windows':'windows','Linux':'linux','Darwin':'macos'}.get(platform.system())
    if actual!=a.platform:raise ValueError('must run on matching platform')
    here=Path(__file__).resolve().parent;d=a.deployment_config
    if d.is_symlink() or not d.is_dir():raise ValueError('private configuration must be a real directory')
    if (d/'BINDING.json').is_symlink():raise ValueError('unexpected binding link')
    b=load(d/'BINDING.json')
    if b.get('platform')!=a.platform:raise ValueError('binding platform mismatch')
    state=Path(b['admitted_state_root'])
    if not state.is_absolute():raise ValueError('admitted state root must be absolute')
    for name,key in [('profile','profile_path'),('closure','closure_path')]:
        local=d/(name+'.json');installed=Path(b[key])
        if not installed.is_absolute() or local.is_symlink() or installed.is_symlink():raise ValueError('unexpected profile path/link')
        expected=b[name+'_sha256']
        if any(hashlib.sha256(x.read_bytes()).hexdigest()!=expected for x in [local,installed]):raise ValueError(name+' binding mismatch')
    profile=load(d/'profile.json');closure=load(d/'closure.json')
    if Path(profile['instance_root'])!=state/'instances':raise ValueError('state binding mismatch')
    if b['expected_host_sha256'] not in [v['sha256'] for v in closure['files'].values()]:raise ValueError('Host identity missing')
    entry=here/'windows-v1830/scripts/microdroid.py' if a.platform=='windows' else here/'controllers'/a.platform/'microdroid.py'
    results=[]
    for gpu in ['on','off']:
        cmd=[sys.executable,'-B',str(entry),'--platform',a.platform,'--state-root',str(state),'check','--gpu',gpu]
        r=subprocess.run(cmd,capture_output=True)
        if r.returncode:raise RuntimeError('production check failed: '+r.stderr.decode(errors='replace'))
        value=json.loads(r.stdout)
        if value.get('asset_hashes')!='PASS' or value.get('vm_started') is not False:raise ValueError('unexpected production check receipt')
        results.append({'gpu':gpu,'exit':r.returncode,'stdout_stderr_eof':True,'asset_hashes':value['asset_hashes']})
    print(json.dumps({'platform':a.platform,'checks':results,'vm_started':False,'security_settings_changed':False,'scope':'Existing admitted deployment only; private paths deliberately omitted'},indent=2))

if __name__=='__main__':main()
